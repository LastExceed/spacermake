use std::collections::HashMap;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tap::Pipe;

use crate::{ResourceId, app_settings};

pub type Schedule = HashMap<ResourceId, Instant>;

pub struct State {
	schedule: RwLock<Schedule>
}

impl State {
	pub fn is_scheduled(&self, resource_id: &ResourceId) -> bool {
		self.schedule.read().contains_key(resource_id)
	}
	
	pub fn schedule(&mut self, resource_id: &ResourceId) {
		let delay =
			app_settings()
			.resources
			[resource_id]
			.shutdown_delay
			.pipe(Duration::from_secs_f32);
		
		self.schedule.write().insert(resource_id.clone(), Instant::now() + delay);
	}

	pub fn cancel(&mut self, resource_id: &ResourceId) {
		self.schedule.write().remove(resource_id);
	}
}