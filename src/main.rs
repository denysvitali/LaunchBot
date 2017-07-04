extern crate reqwest;
extern crate serde_json;
extern crate chrono;
extern crate regex;
extern crate rusqlite;
extern crate yaml_rust;

use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::{thread,time};
use serde_json::{Value,Error};
use chrono::prelude::*;
use chrono::{TimeZone};
use regex::Regex;
use rusqlite::Connection;
use yaml_rust::{YamlLoader};

mod bot;
mod launches;

use bot::Bot;

fn init_db() -> Connection{
    let path = Path::new("database.sqlite");
    let mut is_new_db = false;
    if !path.exists() {
        is_new_db = true;
        File::create(path).expect("Unable to create path");
    }

    let conn = Connection::open(path).expect("Unable to connect to DB");

    if is_new_db {
        conn.execute("CREATE TABLE settings (
            key         TEXT PRIMARY KEY,
            value       TEXT
        )", &[]).expect("Error creating posts table");
    }

    conn
}

fn get_telegram_key() -> String {
    let mut file = File::open(Path::new("./credentials.yml")).expect("File credentials.yml does not exist");
    let mut content = String::new();
    file.read_to_string(&mut content);

    let credentials = YamlLoader::load_from_str(&content).expect("Invalid credentials.yml");
    if credentials.len() == 0 {
        panic!("Credentials file is empty");
    }
    let tgkey = &credentials[0]["telegram_key"].as_str().expect("Telegram Key not found");
    tgkey.to_string()
}

fn main() {
    //send_message(-235707208, 0, String::from("*Test* 1234"));
    let conn = init_db();
    let key = get_telegram_key();

    let mut last_id = -1;
    let mut rows = conn.query_row("SELECT value FROM settings WHERE key = ?", &[&"last_update"], |row| {
        let result : String = row.get(0);
        println!("{}", result);
        last_id = result.parse::<i64>().unwrap();
    });


    let bot : Bot = Bot::new(&key);
    loop {
        let sec = time::Duration::from_millis(1000);
        thread::sleep(sec);
        last_id = bot.get_updates(last_id, &conn);
    }
}
