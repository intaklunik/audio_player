use std::time::Duration;
use std::thread;
use std::sync::mpsc::{Sender};
use crate::app::{AppEvent, FinderEvent};
use crate::player::{Song};

pub struct Finder {
    tx: Sender<AppEvent>,
}

impl Finder {
    pub fn new(tx: Sender<AppEvent>) -> Self {
        Self { tx }
    }

    pub fn lookup_songs(&mut self) {
        thread::sleep(Duration::from_secs(2));
        self.send_event(FinderEvent::NewPlaylist(
            vec![
                Song{ title: "Song 1".to_string(), duration: 4 },
                Song{ title: "Song 2".to_string(), duration: 123 },
                Song{ title: "Song 3".to_string(), duration: 44 },
                Song{ title: "Song 4".to_string(), duration: 399 },
                Song{ title: "Song 5".to_string(), duration: 47 },
                Song{ title: "Song 6".to_string(), duration: 3 },
            ]
        ));
    }

    fn send_event(&self, event: FinderEvent) {
        let _ = self.tx.send(AppEvent::Finder(event));
    }
}