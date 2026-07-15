use std::{
    fs, io,
    sync::mpsc::{Sender},
    path::{PathBuf, Path},
    fs::DirEntry,
};
use crate::app::{AppEvent, FinderEvent, SenderExt};
use crate::player::AudioPlayer;

pub struct Finder {
    tx: Sender<AppEvent>,
}

impl Finder {
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self { tx }
    }

    pub fn lookup_playlist(&mut self, dir: &Path) {
        let event = match Self::lookup_files(dir) {
            Ok(files) => { 
                if files.is_empty() { FinderEvent::NewEmptyPlaylist } 
                else { FinderEvent::NewPlaylist(files) }
            },
            Err(_) => FinderEvent::Error,
        };

        self.tx.send_finder_event(event);
    }

    fn lookup_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let is_extension_supported = |path: PathBuf| 
            path.extension().map_or(false, |s| 
                AudioPlayer::supported_formats().contains(&s.to_string_lossy().as_ref()))
        ;

        let is_file_supported = |entry: &DirEntry| 
            entry.path().is_file() && is_extension_supported(entry.path())
        ;

        let files: Vec<PathBuf> = fs::read_dir(dir)?.into_iter()
            .filter_map(Result::ok)
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
        assert_eq!(Finder::lookup_files(Path::new("./tests/data/empty_dir"))?, Vec::<PathBuf>::new());
        Ok(())
    }

    #[test]
    fn test_supported_files() -> io::Result<()> {
        let files = vec![
            PathBuf::from("./tests/data/song1.mp3")
        ];
        assert_eq!(Finder::lookup_files(Path::new("./tests/data"))?, files);
        Ok(())
    }
}