extern crate reqwest;
extern crate serde_json;
extern crate chrono;
extern crate regex;
extern crate rusqlite;

use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::{thread,time};
use serde_json::{Value,Error};
use chrono::prelude::*;
use chrono::{TimeZone};
use regex::Regex;
use rusqlite::Connection;

const TELEGRAM_API : &str = "https://api.telegram.org/bot";
const TELEGRAM_KEY : &str = "YOUR_TG_KEY";
const NEXT_LAUNCHES : &str = "https://launchlibrary.net/1.2/launch/next/5";

fn get_updates(last_id : i64, conn : &Connection) -> i64 {
    let mut resp = reqwest::get(&format!("{}{}{}?offset={}", TELEGRAM_API, TELEGRAM_KEY, "/getUpdates", last_id.to_string() )).unwrap();
    if resp.status().is_success() {
        let mut content = String::new();
        resp.read_to_string(&mut content);

        let v : Value = serde_json::from_str(&content).unwrap();
        if v["ok"].as_bool().unwrap() == true {
            let max = v["result"].as_array().unwrap().len();
            let lastupdateid = v["result"][max-1]["update_id"].as_i64().unwrap();

            if lastupdateid != last_id {
                for update in v["result"].as_array().unwrap().iter() {
                    if update["update_id"].as_i64().unwrap() != last_id {
                        parse_update(update);
                        println!("{}", update["update_id"]);
                    }
                }
                conn.execute("UPDATE settings SET value=? WHERE key='last_update'", &[&lastupdateid]).expect("Unable to update");
            }

            return lastupdateid;
        }
    }
    else {
        println!("Failed, {}", resp.status());
    }
    0
}

fn parse_update(update : &Value){
    match update.get("message") {
        Some(v) => {
            parse_message(v);
        },
        None => {}
    };
}

fn parse_message(msg : &Value){
    let msgtext = msg["text"].as_str().unwrap();
    println!("[{}] {}: {}", msg["chat"]["title"].as_str().unwrap(), msg["from"]["first_name"].as_str().unwrap(), msgtext);
    let re = Regex::new(r"^/nextlaunch$").unwrap();
    if re.is_match(msgtext) {
        // Send message
        println!("Next launch...");
        send_message(msg["chat"]["id"].as_i64().unwrap(), msg["message_id"].as_i64().unwrap(), get_launches());

    }
}

fn send_message(chatid : i64, replyto : i64, text : String){
    let params = [
        ("chat_id",chatid.to_string()),
        ("text", text),
        ("parse_mode", String::from("Markdown")),
        ("reply_to_message_id", replyto.to_string())
    ];

    let client = reqwest::Client::new().unwrap();
    let res = client.post(&format!("{}{}{}",TELEGRAM_API, TELEGRAM_KEY, "/sendMessage"))
        .form(&params)
        .send();
}

fn main() {
    //send_message(-235707208, 0, String::from("*Test* 1234"));
    let conn = init_db();

    let mut last_id = -1;
    let mut rows = conn.query_row("SELECT value FROM settings WHERE key = ?", &[&"last_update"], |row| {
        let result : String = row.get(0);
        println!("{}", result);
        last_id = result.parse::<i64>().unwrap();
    });

    while true {
        last_id = get_updates(last_id, &conn);
        let sec = time::Duration::from_millis(1000);
        thread::sleep(sec);
    }
}

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

fn get_launches() -> String {

    let mut res = String::new();

    let mut resp = reqwest::get(NEXT_LAUNCHES).unwrap();
    if resp.status().is_success() {
        let mut content = String::new();
        resp.read_to_string(&mut content);
        let v : Value = serde_json::from_str(&content).unwrap();
        let launches : &Value = &v["launches"];

        let mut launchesUpcoming = false;
        let mut first = true;

        for launch in launches.as_array().unwrap().iter() {
            if first {
                first = false;
            }
            else {
                res.push_str("\n\n");
            }

            let utctime = Utc.timestamp(launch["netstamp"].as_i64().unwrap(), 0);
            let launchTimestamp = launch["netstamp"].as_i64().unwrap();
            let nowTimeStamp = Utc::now().timestamp();
            let launchInSeconds = launchTimestamp - nowTimeStamp;


            if launchInSeconds > 0 && launchInSeconds < (60 * 60 * 24) {
                launchesUpcoming = true;
                // Launch in less than 24 hours
                res.push_str(&format!("*{}*\n\n", launch["name"].as_str().unwrap()));

                let hours = ((launchInSeconds as f64) / (60.0 * 60.0)).floor() as i64;
                let minutes = (((launchInSeconds - hours * 60 * 60) as f64) / 60.0).floor() as i64;
                let seconds = launchInSeconds - hours * 60 * 60 - minutes * 60 as i64;

                res.push_str(&format!("Next launch is in {} hours, {} minutes and {} seconds\n\n", hours, minutes, seconds));


                match launch["vidURLs"][0].as_str() {
                    Some(vid) => {res.push_str(&format!("Streaming available [here]({})", vid)); },
                    None => { res.push_str(&format!("No streaming available :("))}
                }
            }

            if !launchesUpcoming {
                res = String::from("No launches planned for the next 24 hours");
            }
        }
    } else {
        res = String::from("Cannot get launches. Please try again later");
    }

    res
}
