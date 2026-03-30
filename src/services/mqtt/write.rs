use std::collections::HashMap;

use rumqttc::{AsyncClient, QoS};
use serde::{Deserialize, Serialize};

use crate::cfg::Supporter;
use crate::*;

pub struct Service {
	client: AsyncClient,
	configs: HashMap<SupporterId, SupporterConfig>
}

impl Service {
	pub(super) fn new(config: &cfg::Main, client: AsyncClient) -> Self {
		Self {
			client,
			configs: SupporterConfig::get_all(config)
		}
	}
	
	pub async fn set_power_state(&self, supporter: &SupporterId, new_state: bool) {
		log::trace!(new_state; "set power state");

		let props = &self.configs[supporter];
		let topic = &props.topic;
		let payload =
			if new_state { &props.payload_startup }
			else         { &props.payload_shutdown };

		log::trace!(topic:%, payload:%; "publishing");

		self.send(topic, payload).await;
	}
	
	pub async fn set_screen_text(&self, leader_id: &LeaderId, runtime: Duration) {
		let formatted_time = format_time(runtime);
		
		let messages = [
            ("title", "Dauer"),
            ("info", &formatted_time),
        ];

        for (route, payload) in messages {
			let topic = format!("fabreader/{}/display/{route}", leader_id.0);
            self.send(&topic, payload).await;
        }
	}
	
	async fn send(&self, topic: &str, payload: &str) {
		self
    	.client
		.publish(topic, QoS::AtMostOnce, false, payload.as_bytes())
		.await
		.expect("failed to publish");
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupporterConfig {
	topic: String,
	payload_startup: String,
	payload_shutdown: String
}

impl SupporterConfig {
	fn get_all(config: &cfg::Main) -> HashMap<SupporterId, Self> {
		config
		.supporters
		.iter()
		.map(|(id, properties)|
			(id.clone(), Self::of(properties))
		)
		.collect()
	}
	
	fn of(properties: &Supporter) -> Self {
		Self {
			topic           : properties.mqtt_topic.clone(),
			payload_startup : properties.mqtt_payload_startup.clone(),
			payload_shutdown: properties.mqtt_payload_shutdown.clone(),
		}
	}
}

fn format_time(runtime: Duration) -> String {
    let mut total_minutes = runtime.as_secs() / 60;
    if !runtime.is_zero() {
        total_minutes += 1; //workaround so partial minutes get rounded up instead of down
    }

    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    format!("{hours:.0}:{minutes:0>2.0}")
}