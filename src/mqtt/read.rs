use itertools::Itertools;
use rumqttc::{EventLoop, Publish};
use serde::Deserialize;
use strum::EnumIs;

pub struct State {
	event_loop: EventLoop
}

impl State {
	pub async fn next_power_state(&mut self) -> (String, bool) {
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

fn parse(publish: &Publish) -> Option<(String, bool)> {	
	let Some(["tele", machine_name, "MARGINS"]) =
		publish.topic.split('/').collect_array()
	else {
		log::error!(publish:?; "unknown topic");
		return None;
	};
	
	let Ok(Payload { margins }) =
		serde_json::from_reader(&publish.payload[..])
	else {
		log::error!(publish:?; "cannot parse payload");
		return None;
	};
	
	let new_power_state = margins.power_high;
	log::info!(machine_name, new_power_state:?; "observed machine activity");
	
	Some(
		(machine_name.to_owned(), new_power_state.is_on())
	)
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