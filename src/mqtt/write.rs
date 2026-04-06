use rumqttc::{AsyncClient, QoS};

use crate::app_settings;

pub struct State {
	client: AsyncClient
}

impl State {
	pub async fn set_power_state(&self, resource: &str, new_state: bool) {
		log::trace!(resource, new_state; "set power state");
		let props = &app_settings().resources[resource];
		
		let topic = &props.mqtt_topic;
		let payload = if new_state { &props.mqtt_payload_startup } else { &props.mqtt_payload_shutdown };

		log::trace!(topic:%, payload:%; "publishing");

		self
    	.client
		.publish(topic, QoS::AtMostOnce, false, payload.as_bytes())
		.await
		.expect("failed to publish");
	}
}