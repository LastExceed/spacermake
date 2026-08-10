use std::time::Duration;
use rumqttc::*;
use tap::prelude::Pipe;

use crate::{APP_NAME, cfg};

pub async fn new() -> (Writer, Reader) {
	log::trace!("create mqtt reader/writer");

	let options =
		cfg::INSTANCE
		.read()
		.await
		.mqtt_broker
		.pipe_ref(make_options);

	let (client, event_loop) = AsyncClient::new(options, 10);
	configure_subscriptions(&client).await;

	(Writer(client), Reader(event_loop))
}

pub fn make_options(broker: &cfg::MqttBroker) -> MqttOptions {
	log::trace!("make mqtt options");

	let mut options = MqttOptions::new(APP_NAME, &broker.host, 1883);
	options.set_keep_alive(Duration::from_secs(5));

	if let (Some(username), Some(password)) = (&broker.username, &broker.password) {
		log::trace!("set mqtt credentials");
		options.set_credentials(username, password);
	} else {
		log::trace!("no mqtt credentials");
	}

	options
}

pub async fn configure_subscriptions(client: &AsyncClient) {
	log::trace!("subscribe mqtt topics");

	client
	.subscribe("tele/+/MARGINS", QoS::AtMostOnce)
	.await
	.expect("failed to subscribe");
}

pub struct Reader(EventLoop);

impl Reader {
	pub async fn next_publish(&mut self) -> Publish {
		use rumqttc::Event::*;
		use rumqttc::Packet::*;

		log::trace!("reading next publish");

		loop {
			match self.0.poll().await {
				Ok(Incoming(Publish(publish))) => break publish,
				Ok(event)                      => log::trace!(event:?; "non-publish event received, ignoring"),
				Err(error)                     => log::error!(error:?; "event_loop.poll() failed")
			}
		}
	}
}

#[derive(Debug)]
pub struct Writer(AsyncClient);

impl Writer {
	pub async fn send(&self, topic: &str, payload: &str) {
		log::trace!(topic:%, payload:%; "publishing");

		_ = self
		.0
		.publish(topic, QoS::AtMostOnce, false, payload.as_bytes())
		.await
		.inspect_err(|error| log::error!(topic:%, payload:%, error:%; "failed to publish"));
	}
}