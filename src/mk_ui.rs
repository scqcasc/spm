use crate::db;
use crate::password;
use dirs::home_dir;
use std::path::PathBuf;

use slint::{
    ComponentHandle, ModelRc, SharedString, StandardListViewItem, ToSharedString, VecModel,
};
slint::include_modules!();

pub fn set_pass(ui: AppWindow, length: i32) {
    let p = password::Password {
        password_type: password::PasswordType::Complex,
        password_length: length,
    };
    let passphrase = p.get_a_password();
    ui.set_passphrase(passphrase.to_shared_string());
}

/// Converts Vec<PassEntry> to ModelRc<ModelRc<StandardListViewItem>>
fn convert_to_table_rows(entries: Vec<db::PassEntry>) -> ModelRc<ModelRc<StandardListViewItem>> {
    // Convert each PassEntry into a ModelRc<StandardListViewItem>
    let rows: Vec<ModelRc<StandardListViewItem>> = entries
        .into_iter()
        .map(|entry| {
            // Wrap the row in a VecModel and then wrap that in a ModelRc
            ModelRc::new(VecModel::from(vec![
                StandardListViewItem::from(SharedString::from(entry.id.to_string())),
                StandardListViewItem::from(SharedString::from(entry.username)),
                StandardListViewItem::from(SharedString::from(entry.url)),
                StandardListViewItem::from(SharedString::from(entry.passphrase)),
                StandardListViewItem::from(SharedString::from(entry.notes)),
            ]))
        })
        .collect();

    // Wrap all rows in an outer VecModel and then wrap that in a ModelRc
    ModelRc::new(VecModel::from(rows))
}

fn load_table() -> Result<ModelRc<ModelRc<StandardListViewItem>>, Box<dyn std::error::Error>> {
    // Get the home directory
    let home = home_dir().ok_or("Could not locate the home directory")?;

    // Construct the configuration path
    let mut config_path: PathBuf = get_config_dir(home);
    config_path.push("spm.db");

    // Query the database for entries
    let entries =
        db::query_all_sl(&config_path).map_err(|e| format!("Failed to query database: {}", e))?;

    // Convert entries to `ModelRc<ModelRc<StandardListViewItem>>`
    let table_rows = convert_to_table_rows(entries);

    Ok(table_rows)
}

pub fn create_ui() -> AppWindow {
    let ui = AppWindow::new().expect("could not start ui");
    let ui_c = ui.clone_strong();

    match load_table() {
        Ok(table_rows) => {
            ui.set_table_rows(table_rows.into());
        }
        Err(e) => {
            panic!("Problem with db {e}")
        }
    }

    set_pass(ui.clone_strong(), 25);

    ui.on_new_pass_clicked({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let length = ui.get_passlength();
            set_pass(ui, length);
        }
    });

    ui.clone_strong().on_update_db_clicked(move || {
        if let Some(home) = home_dir() {
            let mut config_path: PathBuf = get_config_dir(home);
            config_path.push("spm.db");
            let data = db::PassEntry {
                url: ui.get_url().to_string(),
                id: 0,
                username: ui.get_user_name().to_string(),
                passphrase: ui.get_passphrase().to_string(),
                notes: ui.get_notes().to_string(),
            };
            match db::insert_data(&data, &config_path) {
                Ok(_) => {
                    println!("{}", "ok");
                    // message will display for 10 seconds
                    ui.set_message("Entry added".to_shared_string());

                    match load_table() {
                        Ok(table_rows) => {
                            ui.set_table_rows(table_rows.into());
                        }
                        Err(e) => {
                            panic!("Problem with db {e}")
                        }
                    }
                }
                Err(e) => {
                    let mut msg: String = "Entry failed: ".to_owned();
                    msg.push_str(&e.to_string());
                    ui.set_message(msg.to_shared_string());
                }
            };
        }
    });
    ui_c.on_request_increase_value({
        let ui_handle = ui_c.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui_c.on_exit(move || {
        std::process::exit(0);
    });

    // Define the edit_row callback
    let ui_c_c = ui_c.clone_strong();
    ui_c.on_edit_row(move |row_index| {
        println!("Editing row: {}", row_index);
        println!("User is {}", ui_c_c.get_user_name().to_string());
        // Open a dialog or editor for the specified row
    });

    // Define the copy_field callback
    ui_c.on_copy_field(|row_index, field| {
        println!("Copying field '{}' from row {}", field.title, row_index);
        // Fetch the field value and copy it to the clipboard
        // Example: use a clipboard library like `copypasta`
    });
    ui_c
}
