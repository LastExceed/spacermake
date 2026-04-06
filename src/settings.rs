use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

use config::Config;
use serde::Deserialize;

use crate::ResourceId;

pub type ResourceSettings = HashMap<ResourceId, ResourceProperties>;

#[expect(clippy::module_name_repetitions, reason = "avoid name collision with `config` crate")]
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub resources    : HashMap<String, ResourceProperties>,
    pub mqtt_host    : String,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub show_unbooked: bool
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceProperties {
    pub dependencies_booktime: Vec<ResourceId>,
    pub dependencies_runtime : Vec<ResourceId>,
    pub shutdown_delay       : f32,
    pub mqtt_topic           : String,
    pub mqtt_payload_startup : String,
    pub mqtt_payload_shutdown: String,
}

fn open_or_create_file(config: &Config, key: &str) -> String {
    let path = config
        .get_string(key)
        .unwrap();
    
    let mut out = String::new();

    File::options()
        .read(true)
        .write(true) // needed for creation
        .create(true)
        .truncate(false)
        .open(&path)
        .unwrap_or_else(|error| panic!("failed to open file {path} ~~ {error}"))
        .read_to_string(&mut out)
        .unwrap();
    
    out
}