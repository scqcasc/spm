mod db;
mod gui;
mod password;
use eframe::egui;
use std::error::Error;

use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;

fn get_config_dir() -> PathBuf {
    let mut config_dir = dirs::config_local_dir().unwrap();
    config_dir.push("spm");
    config_dir
}

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for MyError {}

fn main() {
    env_logger::init();

    let (background_event_sender, background_event_receiver) = channel::<gui::Event>();
    let (event_sender, event_receiver) = channel::<gui::Event>();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    std::thread::spawn(move || {
        while let Ok(event) = background_event_receiver.recv() {
            let sender = event_sender.clone();
            gui::handle_events(event, sender);
        }
    });

    let mut config_path: PathBuf = get_config_dir();
    let _ = fs::create_dir_all(config_path.clone());
    config_path.push("spm.db");

    let db_con = sqlite::open(config_path.as_path()).expect("can create sqlite db");
    db_con
        .execute(crate::db::CREATE_DB)
        .expect("can initialize sqlite db");
}
