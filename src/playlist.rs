use lofty::read_from_path;
use lofty::prelude::{Accessor, TaggedFileExt};
use lofty::error::LoftyError;
use lofty::file::AudioFile;
use std::path::PathBuf;
use std::fmt;

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
pub struct CurrentTrack {
    pub id: TrackId,
    pub progress: TrackProgress,
}

#[derive(Default, Clone, Debug)]
pub struct Playlist {
    playlist: Vec<Track>,
}

impl fmt::Display for TrackProgress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

impl Track {
    pub fn from_path(path: PathBuf) -> Result<Self, LoftyError> {
        let metadata = TrackMetadata::from_path(&path)?;
        Ok(Self{ path, metadata })
    }

    pub fn metadata(&self) -> &TrackMetadata {
        &self.metadata
    }
}

impl TrackMetadata {
    pub fn from_path(path: &PathBuf) -> Result<Self, LoftyError> {
        let file = read_from_path(path)?;
        let duration = file.properties().duration();
        let (title, author) = match file.primary_tag() {
            Some(tag) => (tag.title().unwrap_or_default().into_owned(), tag.artist().unwrap_or_default().into_owned()),
            None => ("No Title".to_string(), "No Author".to_string()),
        };
    
        Ok(Self {
            fullname: path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
            title,
            author,
            duration: duration.as_secs()
        })
    }
}

impl CurrentTrack {
    pub fn new(id: TrackId) -> Self {
        Self { id, progress: TrackProgress::default() }
    }
}

impl Playlist {
    pub fn get(&self, id: TrackMaybeId) -> TrackId {
        if id < 0 {
            self.playlist.len() - 1
        } else if id >= self.playlist.len() as TrackMaybeId {
            0
        } else {
            id as TrackId
        }
    }

    pub fn get_playlist(&self) -> &Vec<Track> {
        &self.playlist
    }

    pub fn empty(&self) -> bool {
        self.playlist.is_empty()
    }
}

impl From<Vec<PathBuf>> for Playlist {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self { playlist: paths
            .into_iter()
            .filter_map(|path: PathBuf|Track::from_path(path).ok())
            .collect()
        }
    }
}
