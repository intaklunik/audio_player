use crate::player::playlist::{TrackId, TrackMaybeId, TrackMetadata, TrackProgress};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use thiserror::Error;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Lofty error: {0}")]
    Lofty(#[from] lofty::error::LoftyError),

    #[error("Decoder error: {0}")]
    Rodio(#[from] rodio::decoder::DecoderError),

    #[error("Device sink error: {0}")]
    RodioStream(#[from] rodio::stream::DeviceSinkError),

    #[error("Empty playlist")]
    EmptyPlaylist,
}

#[derive(PartialEq)]
pub enum ViewEvent {
    NewPlaylist(Vec<TrackMetadata>),
    NewCurrentTrack(TrackId),
    Progress(TrackProgress),
    Error(String),
    PlayPause,
    Quit,
}

#[derive(PartialEq)]
pub enum UIEvent {
    Next,
    Prev,
    PlayPause(TrackMaybeId), // play button + from playlist
}

#[derive(PartialEq)]
pub enum FinderEvent {
    NewFiles(Vec<PathBuf>),
}

#[derive(PartialEq)]
pub enum AudioPlayerEvent {
    Progress(TrackProgress),
}

pub enum AppEvent {
    UI(UIEvent),
    Player(AudioPlayerEvent),
    Finder(FinderEvent),
    Error(AppError),
    Quit,
}

impl PartialEq for AppEvent {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (AppEvent::UI(_), AppEvent::UI(_))
                | (AppEvent::Player(_), AppEvent::Player(_))
                | (AppEvent::Finder(_), AppEvent::Finder(_))
                | (AppEvent::Error(_), AppEvent::Error(_))
                | (AppEvent::Quit, AppEvent::Quit)
        )
    }
}

pub trait SenderExt {
    fn send_ui_event(&self, event: UIEvent);
    fn send_player_event(&self, event: AudioPlayerEvent);
    fn send_finder_event(&self, event: FinderEvent);
    fn send_error_event(&self, err: AppError);
    fn send_quit_event(&self);
}

impl SenderExt for Sender<AppEvent> {
    fn send_ui_event(&self, event: UIEvent) {
        self.send(AppEvent::UI(event)).unwrap();
    }
    fn send_player_event(&self, event: AudioPlayerEvent) {
        self.send(AppEvent::Player(event)).unwrap();
    }
    fn send_finder_event(&self, event: FinderEvent) {
        self.send(AppEvent::Finder(event)).unwrap();
    }
    fn send_error_event(&self, err: AppError) {
        self.send(AppEvent::Error(err)).unwrap();
    }
    fn send_quit_event(&self) {
        self.send(AppEvent::Quit).unwrap();
    }
}
