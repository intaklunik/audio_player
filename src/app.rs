use std::path::{PathBuf, Path};
use std::sync::mpsc::{Receiver, Sender};
use crate::player::{AudioPlayer, Song, SongId, SongProgress};
use crate::finder::{Finder};

#[derive(PartialEq)]
pub enum ViewEvent {
    NewPlaylist(Vec<Song>),
    NewCurrentSong(SongId),
    Progress(SongProgress),
    Error(String),
    PlayPause,
    Quit,
}

#[derive(PartialEq)]
pub enum UIEvent {
    Next,
    Prev,
    PlayPause(SongId), // play button + from playlist
}

#[derive(PartialEq)]
pub enum FinderEvent {
    NewPlaylist(Vec<PathBuf>),
    NewEmptyPlaylist,
    Error,
}

#[derive(PartialEq)]
pub enum AudioPlayerEvent {
    Progress(SongProgress),
}

#[derive(PartialEq)]
pub enum AppEvent {
    UI(UIEvent),
    Player(AudioPlayerEvent),
    Finder(FinderEvent),
    Quit,
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
         match event {
            FinderEvent::NewPlaylist(playlist) => { 
                self.player.new_playlist(playlist);
                self.view.update(ViewEvent::NewPlaylist(self.player.playlist()));
            },
            FinderEvent::NewEmptyPlaylist => {
                self.player.new_empty_playlist();
                self.view.update(ViewEvent::Error("Empty Playlist".to_string()));
            },
            FinderEvent::Error => {
                self.view.update(ViewEvent::Error("Error occurred during dir search".to_string()));
            }
        }
    }

    fn player_event_handler(&mut self, event: AudioPlayerEvent) {
        match event {
            AudioPlayerEvent::Progress(progress) => self.view.update(ViewEvent::Progress(progress)),
        }
    }

    fn ui_event_handler(&mut self, event: UIEvent) {        
        let result: Result<ViewEvent, String>  = match event {
            UIEvent::PlayPause(id) => {
                if self.player.is_current_song(id) { self.player.play_pause().map(|_| ViewEvent::PlayPause) }
                else { self.player.play(id).map(ViewEvent::NewCurrentSong) }
            },
            UIEvent::Next => self.player.next().map(ViewEvent::NewCurrentSong),
            UIEvent::Prev => self.player.prev().map(ViewEvent::NewCurrentSong),
            
        }; 
        match result {
            Ok(event) => self.view.update(event),
            Err(e) => self.view.update(ViewEvent::Error(e)),
        }
    }

    fn stop(&mut self) {
      //  self.player_state.stop(&mut self.player);
        self.view.update(ViewEvent::Quit);
    }
}


