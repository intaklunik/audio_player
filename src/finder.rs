use crate::app::types::{AppError, AppEvent, FinderEvent, SenderExt};
use crate::player::player::AudioPlayer;
use std::{
    fs,
    fs::DirEntry,
    io,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

pub struct Finder {
    tx: Sender<AppEvent>,
}

impl Finder {
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self { tx }
    }

    pub fn lookup_playlist(&mut self, dir: &Path) {
        match Self::lookup_files(dir) {
            Ok(files) if files.is_empty() => self.tx.send_error_event(AppError::EmptyPlaylist),
            Ok(files) => self.tx.send_finder_event(FinderEvent::NewFiles(files)),
            Err(e) => self.tx.send_error_event(AppError::IO(e)),
        };
    }

    fn lookup_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let is_extension_supported = |path: &PathBuf| {
            path.extension().is_some_and(|s| {
                AudioPlayer::supported_formats().contains(&s.to_string_lossy().as_ref())
            })
        };

        let is_file_supported =
            |entry: &DirEntry| entry.path().is_file() && is_extension_supported(&entry.path());

        let files: Vec<PathBuf> = fs::read_dir(dir)?
            .flatten()
            .filter(|entry| is_file_supported(entry))
            .map(|entry| entry.path())
            .collect();

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dir() -> io::Result<()> {
        assert_eq!(
            Finder::lookup_files(Path::new("./tests/data/empty_dir"))?,
            Vec::<PathBuf>::new()
        );
        Ok(())
    }

    #[test]
    fn test_supported_files() -> io::Result<()> {
        let files = vec![
            PathBuf::from("./tests/data/song1.mp3"),
            PathBuf::from("./tests/data/silence-5sec-32kbps.mp3"),
            PathBuf::from("./tests/data/sfx-compilation.mp3"),
            PathBuf::from("./tests/data/tone-test.mp3"),
        ];
        assert!({
            let recv_files = Finder::lookup_files(Path::new("./tests/data"))?;
            recv_files.len() == files.len() && files.iter().all(|file| recv_files.contains(file))
        });
        Ok(())
    }
}
