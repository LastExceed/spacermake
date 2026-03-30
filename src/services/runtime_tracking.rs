use std::collections::HashMap;
use std::sync::Arc;
use std::time::*;

use csv as _;
use futures::future::join_all;
use parking_lot::RwLock;
use tap::Pipe;

use crate::LeaderId;
use crate::cfg;

use super::mqtt;

pub struct Service {
	mqtt_write: Arc<mqtt::write::Service>,
	records: RwLock<HashMap<LeaderId, Record>>
}

impl Service {
	pub(super) fn new(_config: &cfg::Main, mqtt_write: Arc<mqtt::write::Service>) -> Self {
		Self {
			mqtt_write,
			records: RwLock::default()
		}
	}
	
	pub fn track(&self, leader_id: &LeaderId, new_state: bool) {
		let mut records_guard = self.records.write();
		
		let record =
			records_guard
			.entry(leader_id.clone())
			.or_default();

		if record.is_running() == new_state {
			todo!("double start/stop");
		}
		
		record.toggle();
	}
	
	pub fn take(&self, leader_id: &LeaderId) -> Result<Duration, NotFoundError> {
		self
		.records
		.write()
		.remove(leader_id)
		.ok_or(NotFoundError)
		.map(|record| record.total_runtime())
	}
	
	#[expect(clippy::await_holding_lock, clippy::needless_collect, reason = "false positives")]
	pub async fn refresh_displays(&self) {
		let records = self.records.read();
		
		let data: Vec<_> =
			records
			.iter()
			.filter(|(_id, record)|
				record.is_running()
			)
			.map(|(id, record)| {
				(id.clone(), record.total_runtime())
			})
			.collect(); // clone and collect so we can drop the guard
		
		drop(records);
		
		data
    		.into_iter()
			.map(async |(id, time)| {
				self.mqtt_write.set_screen_text(&id, time).await;
			})
			.pipe(join_all)
			.await;
	}
}

pub struct NotFoundError;

#[derive(Default)]
struct Record {
	past_runtime: Duration,
	running_since: Option<Instant>
}

impl Record {
	fn toggle(&mut self) {
		if let Some(running_since) = self.running_since.take() {
			self.past_runtime += running_since.elapsed();
		}
		else {
			self.running_since = Some(Instant::now());
		}
	}

	const fn is_running(&self) -> bool {
		self.running_since.is_some()
	}

	fn total_runtime(&self) -> Duration { 
		self
		.running_since
		.map(|inst| inst.elapsed())
		.unwrap_or_default()
		+ self.past_runtime
	}
}