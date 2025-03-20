use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::time::Duration;

use serde::de::DeserializeOwned;
use tap::Pipe;

pub mod logs;
pub mod booking;
pub mod index;

pub fn parse_toml_file<T: DeserializeOwned>(path: &str) -> T {
    match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            File::create(path).expect("failed to create file");
            String::new()
        },
        _ => panic!("error reading {path}")
    }
        .pipe_as_ref(toml::from_str)
        .expect("failed to parse toml")
}

pub fn get_power_state(payload: &str) -> Result<String, &'static str> {
    //todo: there gotta be an easier way to do this
    json::parse(payload)
        .map_err(|_| "payload is not a valid json string")?
        .entries()
        .find(|(key, _value)| *key == "MARGINS")
        .ok_or("no MARGINS data present in payload")?
        .1
        .entries()
        .find(|(key, _value)| *key == "PowerHigh")
        .ok_or("no powerHigh information")?
        .1
        .as_str()
        .ok_or("powerHigh state was not a string")
        .map(str::to_string)
}

///whether this duration crossed a minute boundary within the last second
pub fn minute_mark(duration: Duration) -> bool {
    duration.as_secs() % 60 == 0
}

pub fn create_display_time_string(runtime: Duration) -> String {
    let mut total_minutes = runtime.as_secs() / 60;
    if !runtime.is_zero() {
        total_minutes += 1; //workaround so partial minutes get rounded up instead of down
    }

    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    format!("{hours:.0}:{minutes:0>2.0}")
}