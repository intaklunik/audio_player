use std::path::{PathBuf, Path};
use std::sync::mpsc::{Receiver, Sender};
use crate::player::{AudioPlayer};
use crate::playlist::{TrackId, TrackMaybeId, TrackMetadata, TrackProgress};
use crate::finder::{Finder};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum  AppError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("Lofty error: {0}")]
    Lofty(#[from] lofty::error::LoftyError),

    #[error("Empty playlist")]
    EmptyPlaylist,
}

#[derive(PartialEq)]
pub enum ViewEvent {
    NewPlaylist(Vec<TrackMetadata>),
    NewCurrentSong(TrackId),
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
    NewPlaylist(Vec<PathBuf>),
    NewEmptyPlaylist,
    Error,
}

#[derive(PartialEq)]
pub enum AudioPlayerEvent {
    NewTrack(TrackId),
    Progress(TrackProgress),
}

#[derive(PartialEq)]
pub enum AppEvent {
    UI(UIEvent),
    Player(AudioPlayerEvent),
    Finder(FinderEvent),
    Quit,
}

pub trait SenderExt {
    fn send_ui_event(&self, event: UIEvent);
    fn send_player_event(&self, event: AudioPlayerEvent);
    fn send_finder_event(&self, event: FinderEvent);
    fn send_quit_event(&self);
}

impl SenderExt for Sender<AppEvent> {
    fn send_ui_event(&self, event: UIEvent) { self.send(AppEvent::UI(event)).unwrap(); }
    fn send_player_event(&self, event: AudioPlayerEvent) { self.send(AppEvent::Player(event)).unwrap(); }
    fn send_finder_event(&self, event: FinderEvent) { self.send(AppEvent::Finder(event)).unwrap(); }
    fn send_quit_event(&self) { self.send(AppEvent::Quit).unwrap(); }
}

pub trait AppView {
    fn update(&mut self, event: ViewEvent);
}
pub struct Application<View: AppView> {
    player: AudioPlayer,
    view: View,
    finder: Finder,
    rx: Receiver<AppEvent>,
}

impl<View: AppView> Application<View> {
    pub fn new(view: View, rx: Receiver<AppEvent>, tx: Sender<AppEvent>) -> Self {
        let player = AudioPlayer::new(tx.clone());
        let finder = Finder::new(tx);

        Self {player, view, finder, rx}
    }

    pub fn run(&mut self) {
        self.finder.lookup_playlist(Path::new("./tests/data"));
        
        while let Ok(event) = self.rx.recv() {
            match event {
                AppEvent::UI(ev) => self.ui_event_handler(ev), 
                AppEvent::Player(ev) => self.player_event_handler(ev),
                AppEvent::Finder(ev) => self.finder_event_handler(ev),
                AppEvent::Quit => { self.stop(); break; }
            }
        }
    }

    fn finder_event_handler(&mut self, event: FinderEvent) {
        let view_event: ViewEvent = match event {
            FinderEvent::NewPlaylist(playlist) => { 
                self.player.new_playlist(playlist);
                match self.player.playlist() {
                    Some(playlist) => ViewEvent::NewPlaylist(playlist),
                    None => ViewEvent::Error(AppError::EmptyPlaylist.to_string()),
                }
            },
            FinderEvent::NewEmptyPlaylist => {
                self.player.new_empty_playlist();
                ViewEvent::Error(AppError::EmptyPlaylist.to_string())
            },
            FinderEvent::Error => ViewEvent::Error(AppError::EmptyPlaylist.to_string()),
        };

        self.view.update(view_event);
    }

    fn player_event_handler(&mut self, event: AudioPlayerEvent) {
        match event {
            AudioPlayerEvent::Progress(progress) => self.view.update(ViewEvent::Progress(progress)),
            AudioPlayerEvent::NewTrack(id) => self.view.update(ViewEvent::NewCurrentSong(id)),
        }
    }

    fn ui_event_handler(&mut self, event: UIEvent) {        
        let result: AppResult<()> = match event {
            UIEvent::PlayPause(id) => self.player.play(id),
            UIEvent::Next => self.player.next(),
            UIEvent::Prev => self.player.prev(),
        }; 

        result.map_err(|e|self.view.update(ViewEvent::Error(e.to_string())));
    }

    fn stop(&mut self) {
      //  self.player_state.stop(&mut self.player);
        self.view.update(ViewEvent::Quit);
    }
}


