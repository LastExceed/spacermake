use std::io::Read;
use std::fs::File;
use std::collections::{HashMap, HashSet};

use config::Config;
use tap::Pipe;

use self::slave::Slave;

pub mod slave;

#[expect(clippy::module_name_repetitions, reason = "avoid name collision with `config` crate")]
#[derive(Debug)]
pub struct SpacerConfig {
    pub slaves_by_master: HashMap<String, HashSet<String>>,
    pub slave_properties: HashMap<String, Slave>,
    pub machine_ids     : HashMap<String, String>,
    pub data_user       : HashMap<String, UserData>,
    pub data_machines   : HashMap<String, MachineData>,
    pub billing_log     : String,
    pub machine_log     : String,
    pub debug_log       : String,
    pub mqtt_host       : String,
    pub mqtt_username   : Option<String>,
    pub mqtt_password   : Option<String>,
    pub fabaccess_host  : String,
    pub fabaccess_port  : u16
}

#[derive(Debug)]
pub struct UserData {
    pub id: Option<i32>,
    pub to_be_used: bool
}

#[derive(Debug)]
pub struct MachineData {
    pub id: Option<i32>,
    pub to_be_used: bool,
    pub power_sense: bool, //1 = runtime, 0 = booked time
    pub divider: i32
}

impl SpacerConfig {
    pub fn load() -> Self {
        let config = Config::builder()
            .add_source(config::File::with_name("spacermake"))
            .add_source(config::Environment::default())
            .build()
            .expect("failed to load paths");
        
        let slaves_by_master = open_or_create_file(&config, "SLAVES_BY_MASTER") // master-slave_relations.toml
            .pipe_as_ref(toml::from_str)
            .expect("failed to load SLAVES_BY_MASTER");
        
        let slave_properties = open_or_create_file(&config, "SLAVE_PROPERTIES") // slave_properties.toml
            .pipe_as_ref(toml::from_str)
            .expect("failed to load SLAVE_PROPERTIES");
        
        let machine_ids = open_or_create_file(&config, "MACHINE_IDS") // /root/fabfire/config.toml
            .pipe_as_ref(toml::from_str::<toml::Table>)
            .expect("failed to load MACHINE_IDS")
            ["readers"]
            .as_table()
            .unwrap()
            .iter()
            .map(|(_key, value)| {
                let entry = value.as_table().unwrap();
                (
                    entry["machine"].as_str().unwrap().replace("urn:fabaccess:resource:", ""),
                    entry["id"].as_str().unwrap().into()
                )
            })
            .collect();
        
        let data_user = open_or_create_file(&config, "DATA_USER") // DataUser.csv
            .lines()
            .map(|line| {
                let mut splits = line.split(',');
                
                let name = splits.next().unwrap().to_owned();
    
                let ud = UserData {
                    id        : splits.next().unwrap().parse       ().ok(),
                    to_be_used: splits.next().unwrap().parse::<i32>().unwrap_or(1) == 1,
                };
                
                (name, ud)
            })
            .collect();

        let data_machines = open_or_create_file(&config, "DATA_MACHINES") // DataMachines.csv
            .lines()
            .map(|line| {
                let mut splits = line.split(',');
                
                let name = splits.next().unwrap().to_owned();
                let md = MachineData {
                    id         : splits.next().unwrap().parse       ().ok(),
                    to_be_used : splits.next().unwrap().parse::<i32>().unwrap_or(1) == 1,
                    power_sense: splits.next().unwrap().parse::<i32>().unwrap()     == 1,
                    divider    : splits.next().unwrap().parse       ().unwrap()
                };
                
                (name, md)
            })
            .collect();
        
        Self {
            slaves_by_master,
            slave_properties,
            machine_ids,
            data_user,
            data_machines,
            billing_log   : config.get("BILLING_LOG").unwrap(),
            machine_log   : config.get("MACHINE_LOG").unwrap(),
            debug_log     : config.get("DEBUG_LOG").unwrap(),
            mqtt_host     : config.get("MQTT_HOST").unwrap(),
            mqtt_username : config.get("MQTT_USERNAME").ok(),
            mqtt_password : config.get("MQTT_PASSWORD").ok(),
            fabaccess_host: config.get("FABACCESS_HOST").unwrap(),
            fabaccess_port: config.get("FABACCESS_PORT").unwrap()
        }
    }
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