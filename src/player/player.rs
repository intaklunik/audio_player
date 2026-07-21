use super::playlist::{Playlist, Track, TrackId, TrackMaybeId, TrackMetadata};
use crate::app::types::{AppError, AppEvent, AppResult};
use rodio::{Decoder, MixerDeviceSink, Player};
use std::{fs::File, io::BufReader, path::PathBuf, sync::mpsc::Sender};

pub enum AudioPlayerMode {
    Default,
    Random,
}

#[derive(PartialEq)]
enum CurrentTrack {
    Uninitialized,
    Initialized,
    Set(TrackId),
}

impl CurrentTrack {
    pub fn get(&self) -> TrackMaybeId {
        match *self {
            CurrentTrack::Uninitialized => -1,
            CurrentTrack::Initialized => 0,
            CurrentTrack::Set(id) => id as TrackMaybeId,
        }
    }
}

pub struct AudioPlayer {
    playlist: Option<Playlist>,
    current_track: CurrentTrack,
    tx: Sender<AppEvent>,
    mode: AudioPlayerMode,
    handle: MixerDeviceSink,
    player: Player,
}

impl AudioPlayer {
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["mp3"]
    }

    pub fn try_new(tx: Sender<AppEvent>) -> AppResult<Self> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
            .map_err(|err| AppError::RodioStream(err))?;
        let player = rodio::Player::connect_new(handle.mixer());

        Ok(Self {
            playlist: None,
            current_track: CurrentTrack::Uninitialized,
            tx,
            mode: AudioPlayerMode::Default,
            handle,
            player,
        })
    }

    pub fn try_new_playlist(&mut self, playlist: Vec<PathBuf>) -> AppResult<Vec<TrackMetadata>> {
        let new_playlist = Playlist::try_from(playlist)?;
        self.player.stop();

        let playlist_metadata: Vec<TrackMetadata> = new_playlist
            .vec()
            .iter()
            .map(|track: &Track| track.metadata().clone())
            .collect();
        self.playlist = Some(new_playlist);

        Ok(playlist_metadata)
    }

    pub fn stop(&mut self) {
        self.player.stop();
    }
}

// AudioPlayerControl panel

impl AudioPlayer {
    pub fn play_pause(&mut self, id: TrackMaybeId) -> AppResult<Option<TrackId>> {
        if matches!(self.current_track, CurrentTrack::Set(_))
            && matches!(self.current_track.get(), id)
        {
            self.play_pause_rodio();
            Ok(None)
        } else {
            Ok(Some(self.play_track(id)?))
        }
    }

    pub fn next(&mut self) -> AppResult<TrackId> {
        self.play_track(self.current_track.get() + 1)
    }

    pub fn prev(&mut self) -> AppResult<TrackId> {
        self.play_track(self.current_track.get() - 1)
    }

    fn play_track(&mut self, id: TrackMaybeId) -> AppResult<TrackId> {
        let playlist = self.playlist.as_ref().ok_or(AppError::EmptyPlaylist)?;
        let track = match self.mode {
            AudioPlayerMode::Default => playlist.get(id),
            AudioPlayerMode::Random => todo!(),
        };

        self.play_track_rodio(track.path())?;

        let new_id = playlist.normalize(id);
        self.current_track = CurrentTrack::Set(new_id);

        Ok(new_id)
    }

    fn play_pause_rodio(&self) {
        if self.player.is_paused() {
            self.player.play();
        } else {
            self.player.pause();
        }
    }

    fn play_track_rodio(&self, path: &PathBuf) -> AppResult<()> {
        self.player.stop();
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::try_from(file)?;

        self.player.append(source);

        Ok(())
    }
}
