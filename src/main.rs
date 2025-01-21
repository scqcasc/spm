// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;
mod password;
mod mk_ui;

use std::error::Error;
use std::path::PathBuf;
use std::fs;
use std::fmt;
use dirs::home_dir;
use slint::{ComponentHandle, SharedString};


slint::include_modules!();



#[derive(Debug, Clone)]
pub struct TableRow {
    pub id: i32,
    pub username: SharedString,
    pub url: SharedString,
    pub passphrase: SharedString,
    pub notes: SharedString,
}


#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for MyError {}

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(home) = home_dir() {
        let mut config_path: PathBuf = mk_ui::get_config_dir(home);
        // make sure the config_dir exists
        if let Ok(_) = fs::create_dir_all(config_path.clone()) {
            config_path.push("spm.db");
            match db::create_database(&config_path.as_path()) {
                Ok(_) => {
                    println!("{}", "Database Created");
                    let ui_c = mk_ui::create_ui() ;
                    ui_c.run()?;

                    Ok(())
                },
                Err(_) => {
                    panic!("{}", "Database creation failed");
                }
            }   
                
        } else {
            return Err(Box::new(MyError("Could not create config dir".into())));
        }
        
    } else {
        return Err(Box::new(MyError("Could not determine home dir".into())));
    }
    
    
}
