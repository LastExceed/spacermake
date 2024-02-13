use std::time::Duration;
use std::collections::{HashMap, HashSet};

use colour::{dark_grey_ln, magenta_ln};
use lazy_static::*;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use state::{Announcer, Listener, State};

use utils::parse_toml_file;

mod state;
mod utils;

pub const BOOKING_TOPIC: &str = "fabaccess/log";

lazy_static! {
    static ref SLAVES_BY_MASTER: HashMap<String, HashSet<String>> = parse_toml_file("master-slave_relations.toml");
	static ref SLAVE_PROPERTIES: HashMap<String, [bool; 3]> = parse_toml_file("slave_properties.toml");
	static ref MACHINE_IDS: HashMap<String, String> = parse_toml_file::<toml::Table>("/root/fabfire/config.toml")
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
}

#[tokio::main]
async fn main() {
	magenta_ln!("===== spacermake =====");
	print_config();
	let (client, event_loop) = create_client().await;
	magenta_ln!("start");
	let listener = State::new(Listener, client);
	let announcer = listener.duplicate_as(Announcer);

	tokio::spawn(announcer.run());
	listener.run(event_loop).await;
}

fn print_config() {
	let slaves_by_master: &HashMap<_, _> = &SLAVES_BY_MASTER;
	let slave_properties: &HashMap<_, _> = &SLAVE_PROPERTIES;
	let machine_ids: &HashMap<_, _> = &MACHINE_IDS;
	dark_grey_ln!("{slaves_by_master:#?}{slave_properties:#?}{machine_ids:#?}");
}

async fn create_client() -> (AsyncClient, EventLoop) {
	let mut mqttoptions = MqttOptions::new("spacermake", "mqtt.makerspace-bocholt.local", 1883);
	mqttoptions.set_keep_alive(Duration::from_secs(5));

	let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
	client.subscribe("tele/+/MARGINS", QoS::AtMostOnce).await.expect("failed to subscribe");
	client.subscribe(BOOKING_TOPIC,    QoS::AtMostOnce).await.expect("failed to subscribe");

	(client, event_loop)
}