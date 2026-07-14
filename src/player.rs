use std::path::PathBuf;
use std::sync::mpsc::{Sender};
use crate::playlist::*;
use crate::app::AppEvent;

pub enum AudioPlayerMode {
    Default,
    Random,
}

pub struct AudioPlayer {
    playlist: Playlist,
    current_song: CurrentTrack,
    tx: Sender<AppEvent>,
    mode: AudioPlayerMode,
}

impl AudioPlayer {
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["mp3"]
    }

    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self { playlist: Playlist::default(), current_song: CurrentTrack::default(), tx, mode: AudioPlayerMode::Default }
    } 

    pub fn playlist(&self) -> Vec<TrackMetadata> {
        self.playlist.get_playlist()
            .iter()
            .map(|track: &Track| track.metadata().clone())
            .collect()
    }

    pub fn new_playlist(&mut self, playlist: Vec<PathBuf>) {
        self.stop();
        self.playlist = Playlist::from(playlist);
    }

    pub fn new_empty_playlist(&mut self) {
        self.stop();
    }

    pub fn is_current_song(&self, id: TrackId) -> bool { self.current_song.id == id }

    pub fn play(&mut self, id: TrackId) -> Result<TrackId, String> {
        if !self.playlist.empty() {
            self.current_song = CurrentTrack::new(id);
            Ok(self.current_song.id)
        } else {
            Err("Empty playlist".to_string())
        }  
    }

    pub fn next(&mut self) -> Result<TrackId, String> {
        if !self.playlist.empty() {
            let id = match self.mode {
                AudioPlayerMode::Default => self.playlist.get((self.current_song.id as TrackMaybeId) + 1),
                AudioPlayerMode::Random => 0,
            };
            self.play(id)
        } else {
            Err("Next song not found".to_string())
        }     
    }

    pub fn prev(&mut self) -> Result<TrackId, String> {
        if !self.playlist.empty() {
            let id = match self.mode {
                AudioPlayerMode::Default => self.playlist.get((self.current_song.id as TrackMaybeId) - 1),
                AudioPlayerMode::Random => 0,
            };
            self.play(id)
        } else {
            Err("Previous song not found".to_string())
        }   
    }

    pub fn play_pause(&mut self) -> Result<TrackId, String> {
        if !self.playlist.empty() {
            Ok(self.current_song.id)
        } else {
            Err("Nothing to play".to_string())
        }   
        
    }

    fn stop(&mut self) {}
}

