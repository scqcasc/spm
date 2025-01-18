// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;
mod password;
use db::PassEntry;
use std::error::Error;
use std::path::PathBuf;
use std::fs;
use dirs::home_dir;
use std::rc::Rc;
use slint::{ComponentHandle, StandardListViewItem, ModelRc, SharedString, ToSharedString, VecModel};


slint::include_modules!();

pub fn set_pass(ui: AppWindow, length: i32)  {
    let p = password::Password {
        password_type:password::PasswordType::Complex,
        password_length: length,
    };
    let passphrase = p.get_a_password();
    ui.set_passphrase(passphrase.to_shared_string());
}

fn get_config_dir(home: PathBuf) -> PathBuf {
    println!("User's home directory: {}", home.display());
    let mut config_path = PathBuf::from(home);
    config_path.push(".local");
    config_path.push("share");
    config_path.push("spm");
    config_path
}

fn convert_to_slint_model(entries: Vec<PassEntry>) -> ModelRc<Vec<SharedString>> {
    let rows: Vec<Vec<SharedString>> = entries
        .into_iter()
        .map(|entry| {
            vec![
                SharedString::from(entry.id.to_string()),
                SharedString::from(entry.username),
                SharedString::from(entry.url),
                SharedString::from(entry.passphrase),
                SharedString::from(entry.notes),
            ]
        })
        .collect();

    ModelRc::from(Rc::new(VecModel::from(rows)))
}


#[derive(Debug, Clone)]
pub struct TableRow {
    pub id: i32,
    pub username: SharedString,
    pub url: SharedString,
    pub passphrase: SharedString,
    pub notes: SharedString,
}


/// Converts Vec<PassEntry> to ModelRc<ModelRc<StandardListViewItem>>
fn convert_to_table_rows(entries: Vec<PassEntry>) -> ModelRc<ModelRc<StandardListViewItem>> {
    // Convert each PassEntry into a ModelRc<StandardListViewItem>
    let rows: Vec<ModelRc<StandardListViewItem>> = entries
        .into_iter()
        .map(|entry| {
            // Wrap the row in a VecModel and then wrap that in a ModelRc
            ModelRc::new(VecModel::from(vec![
                StandardListViewItem::from(SharedString::from(entry.id.to_string())),
                StandardListViewItem::from(SharedString::from(entry.username)),
                StandardListViewItem::from(SharedString::from(entry.url)),
                // StandardListViewItem::from(SharedString::from(entry.passphrase)),
                StandardListViewItem::from(SharedString::from(entry.notes)),
            ]))
        })
        .collect();

    // Wrap all rows in an outer VecModel and then wrap that in a ModelRc
    ModelRc::new(VecModel::from(rows))
}


fn main() -> Result<(), Box<dyn Error>> {
    if let Some(home) = home_dir() {
        let mut config_path: PathBuf = get_config_dir(home);
        // make sure the config_dir exists
        if let Ok(_) = fs::create_dir_all(config_path.clone()) {
            config_path.push("spm.db");
            match db::create_database(&config_path.as_path()) {
                Ok(_) => {
                    println!("{}", "Database Created");
                },
                Err(_) => {
                    panic!("{}", "Database creation failed");
                }
            }   
                
        } else {
            println!("{}", "Unable to create config directory")
        }
        
    } else {
        println!("Could not determine the home directory.");
    }
    let ui = AppWindow::new()?;
    let ui_c = ui.clone_strong();
    

    // load the database
    if let Some(home) = home_dir() {
        let mut config_path: PathBuf = get_config_dir(home);
        config_path.push("spm.db");

        // let entries = db::query_all_sl(&config_path).expect("db query failed");
         // Query the database for entries
    match db::query_all_sl(&config_path) {
        Ok(entries) => {
            // Convert entries to [[StandardListViewItem]]
            let table_rows = convert_to_table_rows(entries);

            // Set the rows property in the Slint UI
            ui.set_table_rows(table_rows.into());
        }
        Err(e) => {
            eprintln!("Error querying database: {}", e);
        }
    }
    
        
    }

    set_pass(ui.clone_strong(), 25);
    
    ui.on_new_pass_clicked ({
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
                                                      
                },
                Err(e) => {
                    let mut msg : String = "Entry failed: ".to_owned();
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
        
    ui_c.run()?;

    Ok(())
}
