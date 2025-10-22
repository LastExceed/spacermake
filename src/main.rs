use std::time::Duration;

use colour::{dark_grey_ln, magenta_ln};
use futures::future::join3;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use state::{Announcer, Listener, State};

use self::my_config::MyConfig;

pub mod my_config;
mod state;
mod utils;
mod web;
mod schema;

pub const BOOKING_TOPIC: &str = "fabaccess/log";

#[tokio::main]
async fn main() {
	lastexceed::start!();

	magenta_ln!("===== spacermake =====");
	
	let my_config = MyConfig::load();
	dark_grey_ln!("{my_config:#?}");

	let (client, event_loop) = create_client(&my_config).await;
	magenta_ln!("start");
	let listener = State::new(Listener, client, my_config);
	let announcer = listener.duplicate_as(Announcer);

	join3(
		web::start(),
		announcer.run(),
		listener.run(event_loop)
	).await;
}

async fn create_client(my_config: &MyConfig) -> (AsyncClient, EventLoop) {
	let mut mqttoptions = MqttOptions::new("spacermake", &my_config.mqtt_host, 1883);
	mqttoptions.set_keep_alive(Duration::from_secs(5));
	if let (Some(username), Some(password)) = (&my_config.mqtt_username, &my_config.mqtt_password) {
		mqttoptions.set_credentials(username, password);
	}

	let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
	client.subscribe("tele/+/MARGINS", QoS::AtMostOnce).await.expect("failed to subscribe");
	client.subscribe(BOOKING_TOPIC,    QoS::AtMostOnce).await.expect("failed to subscribe");

	(client, event_loop)
}