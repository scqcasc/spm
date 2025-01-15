// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;
mod password;
use std::error::Error;
use std::path::PathBuf;
use std::fs;
use dirs::home_dir;
use slint::ToSharedString;


slint::include_modules!();

pub fn set_pass(ui: AppWindow)  {
    let p = password::Password {
        password_type:password::PasswordType::Complex,
        password_length: 15,
    };
    let passphrase = p.get_a_password();
    ui.set_passphrase(passphrase.to_shared_string());
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(home) = home_dir() {
        println!("User's home directory: {}", home.display());
        let mut config_path = PathBuf::from(home);
        config_path.push(".local");
        config_path.push("share");
        config_path.push("spm");

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
    set_pass(ui.clone_strong());
    
    ui.on_new_pass_clicked ({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            set_pass(ui);
        }
    });

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.on_exit(move || {
        std::process::exit(0);
    });
        
    ui.run()?;

    Ok(())
}
