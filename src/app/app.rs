use std::path::{PathBuf, Path};
use std::sync::mpsc::{Receiver, Sender};
use crate::app::types::*;
use crate::player::player::{AudioPlayer};
use crate::player::playlist::{TrackId, TrackMaybeId, TrackMetadata, TrackProgress};
use crate::finder::{Finder};

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
    pub fn try_new(view: View, rx: Receiver<AppEvent>, tx: Sender<AppEvent>) -> AppResult<Self> {
        let player = AudioPlayer::try_new(tx.clone())?;
        let finder = Finder::new(tx);

        Ok(Self { player, view, finder, rx })
    }

    pub fn run(&mut self) {
        self.finder.lookup_playlist(Path::new("./tests/data"));
        
        while let Ok(event) = self.rx.recv() {
            match event {
                AppEvent::UI(ev) => self.ui_event_handler(ev), 
                AppEvent::Player(ev) => self.player_event_handler(ev),
                AppEvent::Finder(ev) => self.finder_event_handler(ev),
                AppEvent::Error(ev) => self.error_event_handler(ev),
                AppEvent::Quit => { self.stop(); break; }
            }
        }
    }

    fn error_event_handler(&mut self, err: AppError) {
        self.view.update(ViewEvent::Error(err.to_string()));
    }

    fn finder_event_handler(&mut self, event: FinderEvent) {
        match event {
            FinderEvent::NewFiles(files) => {
                match self.player.try_new_playlist(files) {
                    Ok(pl) => self.view.update(ViewEvent::NewPlaylist(pl)),
                    Err(err) => self.error_event_handler(err),
                }
            },
        };
    }

    fn player_event_handler(&mut self, event: AudioPlayerEvent) {
        match event {
            AudioPlayerEvent::Progress(progress) => self.view.update(ViewEvent::Progress(progress)),
        }
    }

    fn ui_event_handler(&mut self, event: UIEvent) {        
        match event {
            UIEvent::PlayPause(id) => {
                match self.player.play_pause(id) {
                    Ok(Some(new_id)) => self.view.update(ViewEvent::NewCurrentTrack(new_id)),
                    Ok(None) => {},
                    Err(err) => self.error_event_handler(err),
                };
            },
            UIEvent::Next => {
                match self.player.next() {
                    Ok(new_id) => self.view.update(ViewEvent::NewCurrentTrack(new_id)),
                    Err(err) => self.error_event_handler(err),
                };
            },
            UIEvent::Prev => {
                match self.player.prev() {
                    Ok(new_id) => self.view.update(ViewEvent::NewCurrentTrack(new_id)),
                    Err(err) => self.error_event_handler(err),
                };
            },
        }; 
    }

    fn stop(&mut self) {
      //  self.player_state.stop(&mut self.player);
        self.view.update(ViewEvent::Quit);
    }
}


