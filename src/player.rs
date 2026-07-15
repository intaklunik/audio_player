use rodio::{MixerDeviceSink, Player, Decoder};
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::mpsc::{Sender}
};
use crate::{app::{AudioPlayerEvent, SenderExt}, playlist::*};
use crate::app::{AppError, AppEvent, AppResult};

pub enum AudioPlayerMode {
    Default,
    Random,
}

pub struct AudioPlayer {
    playlist: Option<Playlist>,
    current_track: Option<TrackId>,
    tx: Sender<AppEvent>,
    mode: AudioPlayerMode,
    handle: MixerDeviceSink,
    player: Player,
}

impl AudioPlayer {
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["mp3"]
    }

    fn play_track(&self, path: &PathBuf) {
        self.player.stop();
        let file = BufReader::new(File::open(path).unwrap());
        let source = Decoder::try_from(file).unwrap();

        self.player.append(source);
    }

    pub fn new(tx: Sender<AppEvent>) -> Self {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = rodio::Player::connect_new(&handle.mixer());

        Self {
            playlist: None,
            current_track: None,
            tx,
            mode: AudioPlayerMode::Default,
            handle,
            player,
         }
    } 

    pub fn playlist(&self) -> Option<Vec<TrackMetadata>> {
        let playlist_metadata: Vec<TrackMetadata> = self.playlist.as_ref()?.get_playlist()
            .iter()
            .map(|track: &Track| track.metadata().clone())
            .collect();
        Some(playlist_metadata)
    }

    pub fn new_playlist(&mut self, playlist: Vec<PathBuf>) {
        self.stop();
        self.playlist = Playlist::try_from(playlist).ok();
    }

    pub fn new_empty_playlist(&mut self) {
        self.stop();
    }

    pub fn play(&mut self, id: TrackMaybeId) -> AppResult<()> {
        let playlist = self.playlist.as_ref().ok_or(AppError::EmptyPlaylist)?;
        
        if let Some(current) = self.current_track {
            if id == current as TrackMaybeId {
            if self.player.is_paused() { self.player.play(); }
            else { self.player.pause(); }
            } 
        }
        else {
            self.play_track(playlist.get(id).path());
            let new_id = playlist.normalize(id);
            self.current_track = Some(new_id);
            self.tx.send_player_event(AudioPlayerEvent::NewTrack(new_id));
        }
        
        Ok(()) 
    }

    pub fn next(&mut self) -> AppResult<()> {
        self._play((self.current_track.unwrap_or(0) as TrackMaybeId) + 1)
    }

    pub fn prev(&mut self) -> AppResult<()> {        
        self._play((self.current_track.unwrap_or(0) as TrackMaybeId) - 1)
    }

    fn _play(&mut self, id: TrackMaybeId) -> AppResult<()> {
        let playlist = self.playlist.as_ref().ok_or(AppError::EmptyPlaylist)?;
        let track = match self.mode {
            AudioPlayerMode::Default => playlist.get(id),
            AudioPlayerMode::Random => todo!(),
        };

        self.play_track(track.path());

        let new_id = playlist.normalize(id);

        self.current_track = Some(new_id);
        self.tx.send_player_event(AudioPlayerEvent::NewTrack(new_id));
        
        Ok(())   
    }

    fn stop(&mut self) {}
}

