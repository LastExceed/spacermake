use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::prelude::future::join_all;
use tap::prelude::Pipe;
use tokio::sync::RwLock;

use crate::cfg;
use crate::SupporterId;

use super::mqtt;

pub struct Service {
	mqtt_write: Arc<mqtt::write::Service>,

	trailing_times: HashMap<SupporterId, Duration>,
	shutdown_schedule: RwLock<HashMap<SupporterId, Instant>>
}

impl Service {
	pub(super) fn new(config: &cfg::Main, mqtt_write: Arc<mqtt::write::Service>) -> Self {
		Self {
			mqtt_write,
			trailing_times: get_trailing_times(config),
			shutdown_schedule: RwLock::default()
		}
	}

	pub async fn update_machine(&self, supporter_id: &SupporterId, new_state: bool) {
		if new_state {
			self.start(supporter_id).await;
		} else {
			self.stop(supporter_id).await;
		}
	}
	
	async fn start(&self, supporter_id: &SupporterId) {
		if self.try_cancel(supporter_id).await {
			return;
		}
		self.mqtt_write.set_power_state(supporter_id, true).await;
	}
	
	async fn try_cancel(&self, supporter_id: &SupporterId) -> bool {
		self
		.shutdown_schedule
		.write()
		.await
		.remove(supporter_id)
		.is_some()
	}
	
	async fn stop(&self, supporter_id: &SupporterId) {
		self
		.shutdown_schedule
		.write()
		.await
		.insert(
			supporter_id.clone(),
			Instant::now() + self.get_winddown_period(supporter_id)
		)
		.expect("double shutdown"); // todo
	}
	
	fn get_winddown_period(&self, machine: &SupporterId) -> Duration {
		self
		.trailing_times
		.get(machine)
		.copied()
		.unwrap_or_default()
	}
	
	pub async fn execute_due_shutdowns(&self) {
		let now = Instant::now();

		self
		.shutdown_schedule
		.write()
		.await
		.extract_if(|_id, time| *time <= now)
		.map(async |(id, _time)| self.mqtt_write.set_power_state(&id, false).await)
		.pipe(join_all)
		.await;
	}
}

fn get_trailing_times(config: &cfg::Main) -> HashMap<SupporterId, Duration> {
	config
		.supporters
		.iter()
		.map(|(id, properties)| (id.clone(), Duration::from_secs_f64(properties.trailing_time)))
		.collect()
}