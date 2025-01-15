// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;
mod password;
use std::error::Error;
use std::path::PathBuf;
use std::fs;
use dirs::home_dir;
use slint::{ComponentHandle, ToSharedString};


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
                    // let md = MessageDialog::new(
                    //     None::<&Window>, 
                    //     DialogFlags::empty(),
                    //      MessageType::Info,
                    //     ButtonsType::Ok, 
                    //     "Entry updated");
                    // md.run();
                    // md.close();
                },
                Err(e) => {
                    let mut msg : String = "Entry failed: ".to_owned();
                    msg.push_str(&e.to_string());
                    panic!("{}", msg);
                    // let md = MessageDialog::new(
                    //     None::<&Window>, 
                    //     DialogFlags::empty(),
                    //      MessageType::Error,
                    //     ButtonsType::Ok, 
                    //     &msg);
                    // md.run();
                    // md.close();
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
