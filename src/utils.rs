use std::fs;
use std::time::Duration;

use serde::de::DeserializeOwned;

pub mod logs;
pub mod booking;
pub mod index;

pub fn parse_toml_file<T: DeserializeOwned>(path: &str) -> T {
    let file_content = fs::read_to_string(path).expect("failed to read .toml file");
    toml::from_str(&file_content).expect("failed to parse toml")
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

pub fn create_display_update_message(runtime: Duration) -> String {
    let hours = runtime.as_secs() / 3600;
    let minutes = runtime.as_secs() / 60 % 60;

    json::object! {
        Cmd: "message",
        MssgID: 12,
        ClrTxt: "Nutzungsdauer",
        AddnTxt: format!("{hours:.0}:{minutes:0>2.0}")
    }.to_string()
}