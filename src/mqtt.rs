use std::time::Duration;
use rumqttc::*;

use crate::app_settings;

pub mod read;
pub mod write;

pub async fn create_client() -> (AsyncClient, EventLoop) {
	let options = build_options();
	let (client, event_loop) = AsyncClient::new(options, 10);

	client
	.subscribe("tele/+/MARGINS", QoS::AtMostOnce)
	.await
	.expect("failed to subscribe");

	(client, event_loop)
}

fn build_options() -> MqttOptions {
	let config = app_settings();
	
	let mut mqttoptions = MqttOptions::new("spacermake", &config.mqtt_host, 1883);
	mqttoptions.set_keep_alive(Duration::from_secs(5));	

	if let (Some(username), Some(password)) = (&config.mqtt_username, &config.mqtt_password) {
		mqttoptions.set_credentials(username, password);
	}

	mqttoptions
}