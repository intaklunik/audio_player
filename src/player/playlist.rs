use crate::app::types::{AppError, AppResult};
use lofty::{
    file::AudioFile,
    prelude::{Accessor, TaggedFileExt},
    read_from_path,
};
use std::{ffi::OsStr, fmt, path::PathBuf};

pub type TrackId = usize;
pub type TrackMaybeId = i128;
pub type TrackDuration = u64;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct TrackProgress(u64);

#[derive(Clone, Debug)]
pub struct Track {
    path: PathBuf,
    metadata: TrackMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackMetadata {
    pub fullname: String,
    pub title: String,
    pub author: String,
    pub duration: TrackDuration,
}

#[derive(Default, Clone, Debug)]
pub(super) struct Playlist {
    playlist: Vec<Track>,
}

impl fmt::Display for TrackProgress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

impl Track {
    pub fn try_from_path(path: PathBuf) -> AppResult<Self> {
        let metadata = TrackMetadata::try_from_path(&path)?;
        Ok(Self { path, metadata })
    }

    pub fn metadata(&self) -> &TrackMetadata {
        &self.metadata
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl TrackMetadata {
    pub fn try_from_path(path: &PathBuf) -> AppResult<Self> {
        let file = read_from_path(path).map_err(|e| AppError::Lofty(e))?;
        let duration = file.properties().duration();

        let title = file
            .primary_tag()
            .and_then(|tag| tag.title().map(|s| s.into_owned()))
            .unwrap_or("No Title".to_string());

        let author = file
            .primary_tag()
            .and_then(|tag| tag.artist().map(|s| s.into_owned()))
            .unwrap_or("No Author".to_string());

        Ok(Self {
            fullname: path
                .file_name()
                .unwrap_or(OsStr::new("No Fullname"))
                .to_string_lossy()
                .into_owned(),
            title,
            author,
            duration: duration.as_secs(),
        })
    }
}

impl Playlist {
    pub fn normalize(&self, id: TrackMaybeId) -> TrackId {
        if id < 0 {
            self.playlist.len() - 1
        } else if id >= self.playlist.len() as TrackMaybeId {
            0
        } else {
            id as TrackId
        }
    }

    pub fn get(&self, id: TrackMaybeId) -> &Track {
        &self.playlist[self.normalize(id)]
    }

    pub fn vec(&self) -> &Vec<Track> {
        &self.playlist
    }
}

impl TryFrom<Vec<PathBuf>> for Playlist {
    type Error = AppError;

    fn try_from(paths: Vec<PathBuf>) -> Result<Self, Self::Error> {
        let playlist: Vec<Track> = paths
            .into_iter()
            .flat_map(|path: PathBuf| Track::try_from_path(path))
            .collect();

        if playlist.is_empty() {
            Err(AppError::EmptyPlaylist)
        } else {
            Ok(Self { playlist })
        }
    }
}
