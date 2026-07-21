use std::sync::mpsc::{Sender};
use cursive::{
    Cursive, CbSink,
    view::{Nameable, Resizable},
    views::{TextView, LinearLayout, Dialog, SelectView, ViewRef, HideableView },
};
use std::fmt::Write;
use crate::app::types::{SenderExt, UIEvent, AppEvent, ViewEvent,};
use crate::player::playlist::{TrackId, TrackMaybeId, TrackMetadata, TrackProgress};
use crate::app::app::{AppView};


const PLAYPAUSE_ICON: &str = "\u{23EF}";
const PREVTRACK_ICON: &str = "<<";
const NEXTTRACK_ICON: &str = ">>";

const PLAYLIST_VIEW_NAME: &str = "PlaylistView";
const TRACK_VIEW_NAME: &str = "TrackView";
const TRACK_PROGRESS_VIEW_NAME: &str = "TrackProgressView";
const WAIT_VIEW_NAME: &str = "WaitView";
const WAIT_VIEW_MSG_NAME: &str = "WaitViewMsg";
const ERROR_VIEW_NAME: &str = "ErrorView";

pub trait CursiveViewExt {
    fn create(&mut self);
}
trait CursiveExt {
    fn show_error_message(&mut self, err: &str);
    fn update_playlist(&mut self, playlist: Vec<TrackMetadata>);
    fn set_current_track(&mut self, id: TrackId);
    fn progress_current_track(&mut self, progress: TrackProgress);

    fn playlist_view(&mut self) -> ViewRef<SelectView::<TrackMetadata>>;
    fn track_view(&mut self) -> ViewRef<TextView>;
    fn track_progress_view(&mut self) -> ViewRef<TextView>;

    fn show_update_message(&mut self, msg: &str);
    fn hide_update_message(&mut self);
}

trait CursiveInputExt {
    fn play_next(&mut self);
    fn play_prev(&mut self);
    fn play_pause(&mut self);
    fn quit_app(&mut self);
}

impl CursiveViewExt for Cursive {
    fn create(&mut self) {
        let mut select_view = SelectView::<TrackMetadata>::new();
    
        select_view = select_view.on_submit(|s: &mut Cursive, _track: &TrackMetadata| s.play_pause());

        let layout = LinearLayout::horizontal()
            .child(select_view.with_name(PLAYLIST_VIEW_NAME).min_width(40).max_width(60))
            .child(TextView::new("").with_name(TRACK_VIEW_NAME).min_width(30).max_width(40))
            .child(TextView::new("").with_name(TRACK_PROGRESS_VIEW_NAME));

        self.add_layer(layout);

        self.menubar()
            .add_leaf(PREVTRACK_ICON, |s| s.play_prev())
            .add_leaf(PLAYPAUSE_ICON, |s| s.play_pause())
            .add_leaf(NEXTTRACK_ICON, |s| s.play_next())
        ;

        self.add_global_callback('q', |s| s.quit_app());
        self.set_autohide_menu(false);

        self.add_layer(HideableView::new(Dialog::around(
            TextView::new("Loading...".to_string()).with_name(WAIT_VIEW_MSG_NAME)
        )).with_name(WAIT_VIEW_NAME));

    }   
}

impl CursiveExt for Cursive {
    fn playlist_view(&mut self) -> ViewRef<SelectView::<TrackMetadata>> {
        self.find_name::<SelectView<TrackMetadata>>(PLAYLIST_VIEW_NAME).unwrap()
    }

    fn track_view(&mut self) -> ViewRef<TextView> {
        self.find_name::<TextView>(TRACK_VIEW_NAME).unwrap()
    }

    fn track_progress_view(&mut self) -> ViewRef<TextView> {
        self.find_name::<TextView>(TRACK_PROGRESS_VIEW_NAME).unwrap()
    }

    fn progress_current_track(&mut self, progress: TrackProgress) {
        let mut track_progress_view = self.track_progress_view();
        track_progress_view.set_content(progress.to_string());
    }

    fn show_error_message(&mut self, err: &str) {
        let view = self.find_name::<TextView>(ERROR_VIEW_NAME);
        if let Some(mut v) = view {
            v.set_content(err.to_string());
        } else {
            self.add_layer(
            Dialog::around(
                TextView::new(err.to_string()).with_name(ERROR_VIEW_NAME))
                .button("Ok", |s| { s.pop_layer(); })
            );
        }
    }

    fn show_update_message(&mut self, msg: &str) {
        let mut view = self.find_name::<TextView>(WAIT_VIEW_MSG_NAME).unwrap();
        view.set_content(msg.to_string());
        
        let mut view = self.find_name::<HideableView<Dialog>>(WAIT_VIEW_NAME).unwrap();
        view.unhide();
    }

    fn hide_update_message(&mut self) {
        let mut view = self.find_name::<HideableView<Dialog>>(WAIT_VIEW_NAME).unwrap();
        view.hide();
    }

    fn update_playlist(&mut self, playlist: Vec<TrackMetadata>) {
        let mut playlist_view = self.playlist_view();
        
        playlist_view.add_all(playlist.into_iter().map(|track|(track.fullname.clone(), track)));
    }

    fn set_current_track(&mut self, id: TrackId) {
        let mut playlist_view = self.playlist_view();
        
        playlist_view.set_selection(id);

        if let Some((_label, track)) = playlist_view.get_item(id) {
            self.track_view().set_content(track_content(track));
            self.track_progress_view().set_content(track.duration.to_string());
        }
    }
}

fn track_content(track: &TrackMetadata) -> String {
    let mut s = String::new();
    writeln!(s, "{}", track.title).unwrap();
    writeln!(s, "{}", track.author).unwrap();

    s
}

impl CursiveInputExt for Cursive {
    fn play_next(&mut self) {
        self.show_update_message("Loading next track");
        send_ui_event(self, UIEvent::Next);
    }

    fn play_prev(&mut self) {
        self.show_update_message("Loading previous track");
         send_ui_event(self, UIEvent::Prev);
    }

    fn play_pause(&mut self) {
        self.show_update_message("Loading current track");
        send_playpause(self);
    }

    fn quit_app(&mut self) {
        self.show_update_message("Quitting");
        send_quit(self);
    }
}

fn send_ui_event(s: &mut Cursive, event: UIEvent) {
    s.with_user_data(|tx: &mut Sender<AppEvent>| { tx.send_ui_event(event); });
}

fn send_quit(s: &mut Cursive) {
    s.with_user_data(|tx: &mut Sender<AppEvent>| { tx.send_quit_event(); });
}

fn send_playpause(s: &mut Cursive) {
    if let Some(track_id) = s.playlist_view().selected_id() {
        send_ui_event(s, UIEvent::PlayPause(track_id as TrackMaybeId));
    }
}

pub struct CursiveAppView {
    sink: CbSink,
}

impl CursiveAppView {
    pub fn new(sink: CbSink) -> Self {
        Self { sink }
    }
}

impl AppView for CursiveAppView {
    fn update(&mut self, event: ViewEvent) {
       match event {
            ViewEvent::NewPlaylist(playlist) => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.update_playlist(playlist); 
                    s.hide_update_message();
                })).unwrap(),
            ViewEvent::NewCurrentTrack(id) => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.set_current_track(id); 
                    s.hide_update_message();
                })).unwrap(),
            ViewEvent::Progress(progress) => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.progress_current_track(progress); 
                })).unwrap(),
            ViewEvent::Error(err) => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.show_error_message(&err); 
                    s.hide_update_message();
                })).unwrap(),
            ViewEvent::Quit => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.hide_update_message();
                    s.quit(); 
                })).unwrap(),
            ViewEvent::PlayPause => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.hide_update_message(); 
                })).unwrap(),
       }
    }

}