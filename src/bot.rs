extern crate reqwest;
extern crate rusqlite;
extern crate serde_json;
extern crate regex;

use std::io::Read;
use rusqlite::Connection;
use serde_json::Value;
use regex::Regex;

const TELEGRAM_API : &str = "https://api.telegram.org/bot";

use super::launches;

pub struct Bot {
    key: String
}

impl Bot {

    pub fn new(api_key: &str) -> Bot {
        Bot {
            key: api_key.to_string()
        }
    }

    pub fn get_updates(&self, last_id : i64, conn : &Connection) -> i64 {
        let mut resp = reqwest::get(&format!("{}{}{}?offset={}", TELEGRAM_API, self.key, "/getUpdates", last_id.to_string() )).unwrap();
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
                            self.parse_update(update);
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

    fn parse_update(&self, update : &Value){
        match update.get("message") {
            Some(v) => {
                self.parse_message(v);
            },
            None => {}
        };
    }

    fn parse_message(&self, msg : &Value){
        let msgtext = match msg["text"].as_str() {
            Some(v) => { v },
            None => {return;}
        };

        println!("[{}] {}: {}", msg["chat"]["title"].as_str().unwrap(), msg["from"]["first_name"].as_str().unwrap(), msgtext);
        let re = Regex::new(r"^/nextlaunch$").unwrap();
        if re.is_match(msgtext) {
            // Send message
            println!("Next launch...");
            self.send_message(msg["chat"]["id"].as_i64().unwrap(), msg["message_id"].as_i64().unwrap(), launches::get_launches());
        }
    }

    fn send_message(&self, chatid : i64, replyto : i64, text : String){
        let params = [
            ("chat_id",chatid.to_string()),
            ("text", text),
            ("parse_mode", String::from("Markdown")),
            ("reply_to_message_id", replyto.to_string())
        ];

        let client = reqwest::Client::new().unwrap();
        let res = client.post(&format!("{}{}{}",TELEGRAM_API, self.key, "/sendMessage"))
            .form(&params)
            .send();
    }
}
