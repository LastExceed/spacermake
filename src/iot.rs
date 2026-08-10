use std::time::Duration;

use itertools::Itertools;
use rumqttc::Publish;

use crate::{cfg, newtypes::*};

mod model;
mod mqtt;

pub async fn new() -> (Controller, Observer) {
	log::trace!("init iot");
	
	let (writer, reader) = mqtt::new().await;

	(Controller(writer), Observer(reader))
}

#[derive(Debug)]
pub struct Controller(mqtt::Writer);

impl Controller {
	pub async fn set_power_state(&self, supporter: &SupporterId, new_state: bool) {
		log::trace!(new_state; "set power state");

		let device_config = &cfg::INSTANCE.read().await.supporters[supporter];

		let topic = &device_config.topic;
		let payload =
			if new_state { &device_config.payload_start }
			else         { &device_config.payload_stop };

		self.0.send(topic, payload).await;
	}

	pub async fn set_screen_text(&self, leader_id: &LeaderId, runtime: Duration) {
		log::trace!(leader_id:?, runtime:?; "set screen text");

		let formatted_time = format_time(runtime);

		let messages = [
			("title", "Dauer"),
			("info", &formatted_time),
		];

		for (route, payload) in messages {
			let topic = format!("fabreader/{}/display/{route}", leader_id.0);
			self.0.send(&topic, payload).await;
		}
	}
}

pub struct Observer(mqtt::Reader);

impl Observer {
	pub async fn next_power_state(&mut self) -> (LeaderId, bool) {
		log::trace!("get next power state");

		loop {
			let publish = self.0.next_publish().await;
			log::trace!(publish:?; "publish received");

			let Some(data) = parse_as_power_state(&publish)
			else {
				log::trace!(publish:?; "non-powerstate publish observed, ignoring");
				continue;
			};

			break data;
		}
	}
}

fn format_time(runtime: Duration) -> String {
	log::trace!(runtime:?; "format time");
	
	let total_minutes = runtime.as_secs() / 60 + 1; // note the integer division. the +1 is there so partial minutes get rounded up instead of down

	let hours = total_minutes / 60;
	let minutes = total_minutes % 60;

	format!("{hours:.0}:{minutes:0>2.0}")
}

fn parse_as_power_state(publish: &Publish) -> Option<(LeaderId, bool)> {
	log::trace!(publish:?; "parsing powerstate");

	let Some(["tele", device_name, "MARGINS"]) =
		publish.topic.split('/').collect_array()
	else {
		log::error!(publish:?; "unknown topic");
		return None;
	};

	let Ok(model::Payload { margins }) =
		serde_json::from_reader(&*publish.payload)
	else {
		log::error!(publish:?; "cannot parse payload");
		return None;
	};

	log::info!(device_name, new_power_state:? = margins.power_high; "observed machine activity");

	let leader_id = LeaderId(device_name.to_owned());
	let new_state = margins.power_high.is_on();

	Some((leader_id, new_state))
}