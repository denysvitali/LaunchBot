extern crate reqwest;
extern crate serde_json;
extern crate chrono;

use std::io::Read;
use serde_json::{Value};
use chrono::{Utc, TimeZone};
const NEXT_LAUNCHES : &str = "https://launchlibrary.net/1.2/launch/next/5";

pub fn get_launches() -> String {

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


            if launchInSeconds > 0 && launchInSeconds < (60 * 60 * 48) {
                launchesUpcoming = true;
                // Launch in less than 24 hours
                res.push_str(&format!("*{}*\n\n", launch["name"].as_str().unwrap()));

                let hours = ((launchInSeconds as f64) / (60.0 * 60.0)).floor() as i64;
                let minutes = (((launchInSeconds - hours * 60 * 60) as f64) / 60.0).floor() as i64;
                let seconds = launchInSeconds - hours * 60 * 60 - minutes * 60 as i64;

                res.push_str(&format!("Next launch is in {} hours, {} minutes and {} seconds\n\n", hours, minutes, seconds));


                match launch["vidURLs"][0].as_str() {
                    Some(vid) => {res.push_str(&format!("Streaming available [here]({})", vid)); },
                    None => { res.push_str(&format!("No streaming available yet :("))}
                }
            }

            if !launchesUpcoming {
                res = String::from("No launches planned for the next 48 hours");
            }
        }
    } else {
        res = String::from("Cannot get launches. Please try again later");
    }

    res
}
