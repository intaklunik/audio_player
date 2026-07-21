use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use cursive::{Cursive, CursiveExt};
use audio_player::app::app::{Application};
use audio_player::app::types::AppEvent;
use audio_player::view::{CursiveAppView, CursiveViewExt};
use audio_player::app::types::AppResult;

fn main() -> AppResult<()> {
    let mut ui = Cursive::new();
    let view = CursiveAppView::new(ui.cb_sink().clone());
    let (ui_tx, ui_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();

    ui.set_user_data(ui_tx.clone());
    ui.create();

    let mut app = Application::try_new(view, ui_rx, ui_tx)?;

    let t_app = thread::spawn(move || { 
            app.run();
    });

    ui.run();
    t_app.join().unwrap();
    Ok(())
}
