#![allow(clippy::field_scoped_visibility_modifiers, reason = "explicit opt-in")]

use rumqttc::*;

use crate::*;

pub mod read;
pub mod write;

pub async fn create_split(config: &cfg::Main) -> (read::Service, write::Service) {
	let (client, event_loop) = AsyncClient::new(options(config), 10);

	client
	.subscribe("tele/+/MARGINS", QoS::AtMostOnce)
	.await
	.expect("failed to subscribe");

	(
		read::Service { event_loop },
		write::Service::new(config, client)
	)
}

fn options(config: &cfg::Main) -> MqttOptions {
	let broker = &config.mqtt_broker;

	let mut mqttoptions = MqttOptions::new("spacermake", &broker.host, 1883);
	mqttoptions.set_keep_alive(Duration::from_secs(5));	

	if let (Some(username), Some(password)) = (&broker.username, &broker.password) {
		mqttoptions.set_credentials(username, password);
	}

	mqttoptions
}