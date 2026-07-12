use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use cursive::{Cursive, CursiveExt};
use audio_player::app::{Application, AppEvent};
use audio_player::view::{CursiveAppView, CursiveViewExt};


fn main() {
    let mut ui = Cursive::new();
    let view = CursiveAppView::new(ui.cb_sink().clone());
    let (ui_tx, ui_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();

    ui.set_user_data(ui_tx.clone());
    ui.create();

    let t_app = thread::spawn(move || { 
        let mut app = Application::new(view, ui_rx, ui_tx);
        app.run();
    });

    ui.run();
    t_app.join().unwrap();
}
