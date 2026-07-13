use std::path::PathBuf;
use std::sync::mpsc::{Sender};
use std::fmt;
use crate::app::AppEvent;

pub type SongId = usize;
type SongMaybeId = i128;
pub type SongDuration = u64;


#[derive(Default, Clone, Debug, PartialEq)]
pub struct SongProgress(u64);

#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    pub title: String,
    pub duration: SongDuration,
}

#[derive(Default, Clone, Debug)]
pub struct CurrentSong {
    pub id: SongId,
    pub progress: SongProgress,
}

#[derive(Default, Clone, Debug)]
pub struct Playlist {
    playlist: Vec<Song>,
}

pub enum AudioPlayerMode {
    Default,
    Random,
}

pub struct AudioPlayer {
    playlist: Playlist,
    current_song: CurrentSong,
    tx: Sender<AppEvent>,
    mode: AudioPlayerMode,
}

impl fmt::Display for SongProgress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl CurrentSong {
    pub fn new(id: SongId) -> Self {
        Self { id, progress: SongProgress::default() }
    }
}

impl Playlist {
    pub fn get(&self, id: SongMaybeId) -> SongId {
        if id < 0 {
            self.playlist.len() - 1
        } else if id >= self.playlist.len() as SongMaybeId {
            0
        } else {
            id as SongId
        }
    }

    pub fn empty(&self) -> bool {
        self.playlist.is_empty()
    }
}

impl From<Vec<PathBuf>> for Playlist {
    fn from(paths: Vec<PathBuf>) -> Self {
        let playlist = paths.into_iter().map(|path| {
            Song {
                title: path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
                duration: 0
            }
        }).collect();
        
        Self {playlist}
    }
}

impl AudioPlayer {
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["mp3"]
    }

    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self { playlist: Playlist::default(), current_song: CurrentSong::default(), tx, mode: AudioPlayerMode::Default }
    } 

    pub fn playlist(&self) -> Vec<Song> {
        self.playlist.playlist.clone()
    }

    pub fn new_playlist(&mut self, playlist: Vec<PathBuf>) {
        self.stop();
        self.playlist = Playlist::from(playlist);
     
    }

    pub fn new_empty_playlist(&mut self) {
        self.stop();
    }

    pub fn is_current_song(&self, id: SongId) -> bool { self.current_song.id == id }

    pub fn play(&mut self, id: SongId) -> Result<SongId, String> {
        if !self.playlist.empty() {
            self.current_song = CurrentSong::new(id);
            Ok(self.current_song.id)
        } else {
            Err("Empty playlist".to_string())
        }  
    }

    pub fn next(&mut self) -> Result<SongId, String> {
        if !self.playlist.empty() {
            let id = match self.mode {
                AudioPlayerMode::Default => self.playlist.get((self.current_song.id as SongMaybeId) + 1),
                AudioPlayerMode::Random => 0,
            };
            self.play(id)
        } else {
            Err("Next song not found".to_string())
        }     
    }

    pub fn prev(&mut self) -> Result<SongId, String> {
        if !self.playlist.empty() {
            let id = match self.mode {
                AudioPlayerMode::Default => self.playlist.get((self.current_song.id as SongMaybeId) - 1),
                AudioPlayerMode::Random => 0,
            };
            self.play(id)
        } else {
            Err("Previous song not found".to_string())
        }   
    }

    pub fn play_pause(&mut self) -> Result<SongId, String> {
        if !self.playlist.empty() {
            Ok(self.current_song.id)
        } else {
            Err("Nothing to play".to_string())
        }   
        
    }

    fn stop(&mut self) {}
}

