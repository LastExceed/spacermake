use itertools::Itertools;
use rumqttc::{EventLoop, Publish};
use serde::Deserialize;
use strum::EnumIs;

use crate::LeaderId;

pub struct Service {
	pub(super) event_loop: EventLoop
}

impl Service {
	pub async fn next_power_state(&mut self) -> (LeaderId, bool) {
		loop {
			let publish = self.next_publish().await;
			log::trace!(publish:?; "publish received");
			
			let Some(data) = parse(&publish)
			else { continue };

			break data;
		}
	}

	async fn next_publish(&mut self) -> Publish {
		loop {
			use rumqttc::Event::*;
			use rumqttc::Packet::*;

			match self.event_loop.poll().await {
				Ok(Incoming(Publish(publish))) => break publish,
				Ok(event) => log::debug!(event:?; "non-publish event observed"),
				Err(error) => log::error!(error:?; "event_loop.poll() failed")
			}
		}
	}
}

fn parse(publish: &Publish) -> Option<(LeaderId, bool)> {	
	let Some(["tele", machine_name, "MARGINS"]) =
		publish.topic.split('/').collect_array()
	else {
		log::error!(publish:?; "unknown topic");
		return None;
	};
	
	let Ok(Payload { margins }) =
		serde_json::from_reader(&*publish.payload)
	else {
		log::error!(publish:?; "cannot parse payload");
		return None;
	};
	
	log::info!(machine_name, new_power_state:? = margins.power_high; "observed machine activity");

	let leader_id = LeaderId(machine_name.to_owned());
	let new_state = margins.power_high.is_on();
	
	Some((leader_id, new_state))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all="UPPERCASE")]
struct Payload {
	margins: Margins
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all="PascalCase")]
struct Margins {
	power_high: PowerHigh
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, EnumIs)]
#[serde(rename_all="UPPERCASE")]
enum PowerHigh {
	On,
	Off
}