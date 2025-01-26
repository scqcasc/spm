use crate::db;
use crate::db::PassEntry;
use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub enum Event {
    SetEntries(Vec<PassEntry>),
    GetEntryFromDB(egui::Context, Arc<Mutex<sqlite::Connection>>, i64),
    SetSelectedEntry(Option<PassEntry>),
    InsertEntryToDB(egui::Context, Arc<Mutex<sqlite::Connection>>, PassEntry),
    DeleteEntryFromDB(egui::Context, Arc<Mutex<sqlite::Connection>>, i64),
}

pub fn handle_events(event: Event, sender: Sender<Event>) {
    match event {
        Event::GetEntryFromDB(ctx, db_con, entry_id) => {
            if let Ok(Some(entry)) = db::get_entry_from_db(db_con, entry_id) {
                let _ = sender.send(Event::SetSelectedEntry(Some(entry)));
                ctx.request_repaint();
            }
        }
        Event::DeleteEntryFromDB(ctx, db_con, entry_id) => {
            if db::delete_entry_from_db(db_con.clone(), entry_id).is_ok() {
                if let Ok(entries) = db::get_entries_from_db(db_con) {
                    let _ = sender.send(Event::SetEntries(entries));
                    ctx.request_repaint();
                }
            }
        }
        Event::InsertEntryToDB(ctx, db_con, entry) => {
            if let Ok(new_entry) = db::insert_entry_to_db(db_con.clone(), entry) {
                if let Ok(entries) = db::get_entries_from_db(db_con) {
                    let _ = sender.send(Event::SetEntries(entries));
                    let _ = sender.send(Event::SetSelectedEntry(Some(new_entry)));
                    ctx.request_repaint();
                }
            }
        }
        _ => (),
    }
}
