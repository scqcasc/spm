use anyhow::{anyhow, Result};

use crate::db;
use crate::db::PassEntry;
use crate::password::{self, Password, PasswordType};
use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct MainApp {
    pub app_state: AppState,
    pub background_event_sender: Sender<Event>,
    pub event_receiver: Receiver<Event>,
    pub db_con: Arc<Mutex<sqlite::Connection>>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub selected_entry: Option<PassEntry>,
    pub entries: Vec<PassEntry>,
    pub add_form: AddForm,
}

#[derive(Debug, Clone)]
pub struct AddForm {
    pub show: bool,
    pub show_pass: bool,
    pub auto_pass: bool,
    pub name: String,
    pub url: String,
    pub passphrase: String,
    pub notes: String,
}

pub enum Event {
    DeleteEntryFromDB(egui::Context, Arc<Mutex<sqlite::Connection>>, i64),
    GetEntryFromDB(egui::Context, Arc<Mutex<sqlite::Connection>>, i64),
    GetPassword(egui::Context, Password),
    InsertEntryToDB(egui::Context, Arc<Mutex<sqlite::Connection>>, PassEntry),
    SetEntries(Vec<PassEntry>),
    SetSelectedEntry(Option<PassEntry>),
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

impl MainApp {
    pub fn new(
        background_event_sender: Sender<Event>,
        event_receiver: Receiver<Event>,
        db_con: sqlite::Connection,
    ) -> Result<Box<Self>> {
        let db_con = Arc::new(Mutex::new(db_con));
        let entries = db::get_entries_from_db(db_con.clone())?;
        let pass = Password {
            password_type: PasswordType::Complex,
            password_length: 25,
        };
        Ok(Box::new(Self {
            app_state: AppState {
                selected_entry: None,
                entries,
                add_form: AddForm {
                    show: false,
                    show_pass: false,
                    auto_pass: true,
                    name: String::default(),
                    url: String::default(),
                    passphrase: pass.get_a_password(),
                    notes: String::default(),
                },
            },
            background_event_sender,
            event_receiver,
            db_con,
        }))
    }

    fn handle_gui_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                Event::SetSelectedEntry(entry) => self.app_state.selected_entry = entry,
                Event::SetEntries(entries) => {
                    if let Some(ref selected_entry) = self.app_state.selected_entry {
                        if !entries.iter().any(|p| p.id == selected_entry.id) {
                            self.app_state.selected_entry = None;
                        }
                    }
                    self.app_state.entries = entries;
                }
                _ => (),
            };
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.handle_gui_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("left panel")
                .resizable(true)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Passwords");
                        ui.separator();
                        if ui.button("Hide/Show Entry Form").clicked() {
                            self.app_state.add_form.show = !self.app_state.add_form.show;
                        }
                        if ui.button("Exit").clicked() {
                            std::process::exit(0);
                        }
                        if self.app_state.add_form.show {
                            ui.separator();

                            ui.vertical_centered(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("name:");
                                        ui.label("url");
                                        ui.label("passphrase");
                                        ui.label("notes");
                                    });
                                    ui.end_row();
                                    ui.vertical(|ui| {
                                        ui.text_edit_singleline(&mut self.app_state.add_form.name);
                                        ui.text_edit_singleline(&mut self.app_state.add_form.url);
                                        ui.text_edit_singleline(
                                            &mut self.app_state.add_form.passphrase,
                                        );

                                        ui.text_edit_singleline(&mut self.app_state.add_form.notes);
                                    });
                                });

                                if ui.button("Submit").clicked() {
                                    let add_form = &mut self.app_state.add_form;
                                    let username = add_form.name.to_owned();
                                    let url = add_form.url.to_owned();
                                    let passphrase = add_form.passphrase.to_owned();
                                    let notes = add_form.notes.to_owned();
                                    if !username.is_empty() && !passphrase.is_empty() {
                                        let _ = self.background_event_sender.send(
                                            Event::InsertEntryToDB(
                                                ctx.clone(),
                                                self.db_con.clone(),
                                                PassEntry {
                                                    id: -1,
                                                    username,
                                                    url,
                                                    passphrase,
                                                    notes,
                                                },
                                            ),
                                        );

                                        add_form.name = String::default();
                                        add_form.url = String::default();
                                        add_form.passphrase = String::default();
                                        add_form.notes = String::default();
                                    }
                                }
                            });
                        }

                        ui.separator();
                        self.app_state.entries.iter().for_each(|entry| {
                            if ui
                                .selectable_value(
                                    &mut self.app_state.selected_entry,
                                    Some(entry.to_owned()),
                                    entry.username.clone(),
                                )
                                .changed()
                            {
                                let _ = self.background_event_sender.send(Event::GetEntryFromDB(
                                    ctx.clone(),
                                    self.db_con.clone(),
                                    entry.id,
                                ));
                            }
                        });
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Details");
                    if let Some(entry) = &self.app_state.selected_entry {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Delete").clicked() {
                                    let _ = self.background_event_sender.send(
                                        Event::DeleteEntryFromDB(
                                            ctx.clone(),
                                            self.db_con.clone(),
                                            entry.id,
                                        ),
                                    );
                                }
                            });
                            ui.separator();
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("id:");
                                        ui.label("name:");
                                        ui.label("url");
                                        ui.label("passphrase");
                                        ui.label("notes");
                                    });
                                    ui.end_row();
                                    ui.vertical(|ui| {
                                        ui.label(entry.id.to_string());
                                        ui.label(entry.username.to_string());
                                        ui.label(entry.url.to_string());
                                        ui.label(entry.passphrase.to_string());
                                        ui.label(entry.notes.to_string());
                                    });
                                });
                                ui.separator();
                            });
                        });
                    } else {
                        ui.label("No entry selected.");
                    }
                });
            });
        });
    }
}
