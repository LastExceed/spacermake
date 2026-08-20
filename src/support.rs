use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use futures::prelude::future;
use tap::prelude::Pipe;

use crate::{cfg, newtypes::*};
use super::iot::*;

#[derive(Debug)]
pub struct Network {
	currently_running: HashSet<SupporterId>,
	shutdown_schedule: HashMap<SupporterId, Instant>,
	controller: Controller
}

impl Network {
	pub(crate) fn new(controller: Controller) -> Self {
		Self {
			currently_running: HashSet::default(),
			shutdown_schedule: HashMap::default(),
			controller
		}
	}
	
	pub async fn update_running_set(&mut self, mut new_set: HashSet<&SupporterId>) {
		log::trace!(self:?, new_set:?; "update running set");
		
		let now = Instant::now();
		
		let cfg_guard = cfg::INSTANCE.read().await;
		
		// schedule stops
		for id in self.currently_running.extract_if(|id| !new_set.contains(&id)) {
			let trailing_time = cfg_guard.supporters[&id].trailing_seconds.pipe(Duration::from_secs);
			let terminus = now + trailing_time;
			log::debug!("schedule shutdown - supporter: `{}`, delay: {:?}", id.0, trailing_time);
			self.shutdown_schedule.insert(id, terminus);
		}
		
		// do starts
		new_set.retain(|id| !self.currently_running.contains(id));

		new_set
    	.iter()
		.map(|id| self.controller.set_power_state(id, true))
		.pipe(future::join_all)
		.await;
	
		self.currently_running.extend(new_set.into_iter().cloned());
	}
	
	pub async fn update_screens(&self, times: impl Iterator<Item = (&LeaderId, Duration)>) {
		// log::trace!("update screens");
		
		let cfg_guard = cfg::INSTANCE.read().await;
		
		times
		.filter_map(|(leader_id, runtime)|
			cfg_guard
			.leaders
			[leader_id]
			.runtime_display_id
			.as_ref()
			.map(|display_id|
				self.controller.set_screen_text(display_id, runtime)
			)
		)
		.pipe(future::join_all)
		.await;
	}
	
	pub async fn shutdown_due(&mut self) {		
		// log::trace!("shutdown_due");

		let now = Instant::now();
		
		self
		.shutdown_schedule
		.extract_if(|_id, time| *time < now)
		.map(async |(id, _)| self.controller.set_power_state(&id, false).await)
		.pipe(future::join_all)
		.await;
	}
}