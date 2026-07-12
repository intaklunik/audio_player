use std::sync::mpsc::{Sender};
use cursive::{
    Cursive, CbSink,
    view::{Nameable, Resizable},
    views::{TextView, LinearLayout, Dialog, SelectView, ViewRef, HideableView },
};
use crate::player::{Song, SongId, SongProgress};
use crate::app::{UIEvent, AppEvent, ViewEvent, AppView};


const PLAYPAUSE_ICON: &str = "\u{23EF}";
const PREVSONG_ICON: &str = "<<";
const NEXTSONG_ICON: &str = ">>";

const PLAYLIST_VIEW_NAME: &str = "PlaylistView";
const SONG_VIEW_NAME: &str = "SongView";
const SONG_PROGRESS_VIEW_NAME: &str = "SongProgressView";
const WAIT_VIEW_NAME: &str = "WaitView";
const WAIT_VIEW_MSG_NAME: &str = "WaitViewMsg";
const ERROR_VIEW_NAME: &str = "ErrorView";

pub trait CursiveViewExt {
    fn create(&mut self);
}
trait CursiveExt {
    fn show_error_message(&mut self, err: &str);
    fn update_playlist(&mut self, playlist: Vec<Song>);
    fn set_current_song(&mut self, id: SongId);
    fn progress_current_song(&mut self, progress: SongProgress);

    fn playlist_view(&mut self) -> ViewRef<SelectView::<Song>>;
    fn song_view(&mut self) -> ViewRef<TextView>;
    fn song_progress_view(&mut self) -> ViewRef<TextView>;

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
        let mut select_view = SelectView::<Song>::new();
    
        select_view = select_view.on_submit(|s: &mut Cursive, _song: &Song| s.play_pause());

        let layout = LinearLayout::horizontal()
            .child(select_view.with_name(PLAYLIST_VIEW_NAME).min_width(40).max_width(60))
            .child(TextView::new("").with_name(SONG_VIEW_NAME).min_width(30).max_width(40))
            .child(TextView::new("").with_name(SONG_PROGRESS_VIEW_NAME));

        self.add_layer(layout);

        self.menubar()
            .add_leaf(PREVSONG_ICON, |s| s.play_prev())
            .add_leaf(PLAYPAUSE_ICON, |s| s.play_pause())
            .add_leaf(NEXTSONG_ICON, |s| s.play_next())
        ;

        self.add_global_callback('q', |s| s.quit_app());
        self.set_autohide_menu(false);

        self.add_layer(HideableView::new(Dialog::around(
            TextView::new("Loading...".to_string()).with_name(WAIT_VIEW_MSG_NAME)
        )).with_name(WAIT_VIEW_NAME));

    }   
}

impl CursiveExt for Cursive {
    fn playlist_view(&mut self) -> ViewRef<SelectView::<Song>> {
        self.find_name::<SelectView<Song>>(PLAYLIST_VIEW_NAME).unwrap()
    }

    fn song_view(&mut self) -> ViewRef<TextView> {
        self.find_name::<TextView>(SONG_VIEW_NAME).unwrap()
    }

    fn song_progress_view(&mut self) -> ViewRef<TextView> {
        self.find_name::<TextView>(SONG_PROGRESS_VIEW_NAME).unwrap()
    }

    fn progress_current_song(&mut self, progress: SongProgress) {
        let mut song_progress_view = self.song_progress_view();
        song_progress_view.set_content(progress.to_string());
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

    fn update_playlist(&mut self, playlist: Vec<Song>) {
        let mut playlist_view = self.playlist_view();
        
        playlist_view.add_all(playlist.into_iter().map(|song|(song.to_string(), song)));
    }

    fn set_current_song(&mut self, id: SongId) {
        let mut playlist_view = self.playlist_view();
        
        playlist_view.set_selection(id);

        if let Some((_label, song)) = playlist_view.get_item(id) {
            self.song_view().set_content(song.to_string());
            self.song_progress_view().set_content(song.duration.to_string());
        }
    }
}

impl CursiveInputExt for Cursive {
    fn play_next(&mut self) {
        self.show_update_message("Loading next song");
        send_ui_event(self, UIEvent::Next);
    }

    fn play_prev(&mut self) {
        self.show_update_message("Loading previous song");
         send_ui_event(self, UIEvent::Prev);
    }

    fn play_pause(&mut self) {
        self.show_update_message("Loading current song");
        send_playpause(self);
    }

    fn quit_app(&mut self) {
        self.show_update_message("Quitting");
        send_quit(self);
    }
}

fn send_ui_event(s: &mut Cursive, event: UIEvent) {
    s.with_user_data(|tx: &mut Sender<AppEvent>| { tx.send(AppEvent::UI(event)).unwrap(); });
}

fn send_quit(s: &mut Cursive) {
    s.with_user_data(|tx: &mut Sender<AppEvent>| { tx.send(AppEvent::Quit).unwrap(); });
}

fn send_playpause(s: &mut Cursive) {
    if let Some(song_id) = s.playlist_view().selected_id() {
        send_ui_event(s, UIEvent::PlayPause(song_id));
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
            ViewEvent::NewCurrentSong(id) => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.set_current_song(id); 
                    s.hide_update_message();
                })).unwrap(),
            ViewEvent::Progress(progress) => 
                self.sink.send(Box::new(move | s: &mut Cursive| {
                    s.progress_current_song(progress); 
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