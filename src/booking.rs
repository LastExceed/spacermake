use std::collections::HashMap;
use std::time::{Duration, Instant};
use strum::EnumIs;
use tap::prelude::Pipe;
use thiserror::Error;

use crate::newtypes::*;

#[derive(Debug, Clone, Copy, EnumIs)]
pub enum ToggleOutcome {
	Booked,
	Released { booked_time: Duration, runtime: Duration }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum ToggleError {
	#[error("Target machine is occupied by a different user")]
	Occupied,
	#[error("Target machine is still running ")]
	StillRunning
}

#[derive(Debug, Default)]
pub struct Registry(HashMap<LeaderId, Record>);

impl Registry {
	pub fn try_toggle(&mut self, target: &LeaderId, user_name: &UserName) -> Result<ToggleOutcome, ToggleError> {
		log::trace!(target:?, user_name:?; "try toggle booking");

		let Some(booking) = self.0.get(target)
		else {
			log::trace!("booking");
			self.0.insert(target.clone(), Record::new(user_name.clone()));
			log::info!("`{}` booked `{}`", user_name.0, target.0);
			return Ok(ToggleOutcome::Booked);
		};

		if booking.occupant != *user_name {
			log::debug!("`{}` failed to book `{}` (occupied)", user_name.0, target.0);
			return Err(ToggleError::Occupied)
		}
		if booking.is_running() {
			log::debug!("`{}` failed to release `{}` (still running)", user_name.0, target.0);
			return Err(ToggleError::StillRunning)
		}

		log::trace!("releasing");

		let booking =
			self.0
			.remove(target)
			.unwrap(); // we just verified that the entry exists
		
		log::info!("`{}` released `{}`", user_name.0, target.0);
		
		ToggleOutcome::Released {
			booked_time: booking.booked_at.elapsed(),
			runtime: booking.total_runtime()
		}
		.pipe(Ok)
	}

	pub fn track_activity(&mut self, target: &LeaderId, new_state: bool) {
		log::trace!(target:?, new_state:%; "track activity");

		let Some(booking) = self.0.get_mut(target)
		else {
			log::error!(target:?, new_state:%; "activity from unbooked");
			return;
		};

		if booking.running_since.is_some() == new_state {
			log::error!(target:?, new_state:%; "wrong power state reported");
			return;
		}

		booking.toggle_running();
	}
	
	pub fn all_states(&self) -> impl Iterator<Item=(&LeaderId, bool)> {
		self.0
		.iter()
		.map(|(id, booking)|
			(id, booking.running_since.is_some())
		)
	}

	pub fn active_runtimes(&self) -> impl Iterator<Item=(&LeaderId, Duration)> {
		// log::trace!(self:?; "get all running machines");

		self.0
		.iter()
		.filter(|(_id, booking)| booking.is_running())
		.map(|(id, booking)| (id, booking.total_runtime()))
	}

	pub fn is_the_occupant(&self, leader_id: &LeaderId, user_name: &UserName) -> Option<bool> {
		log::trace!(leader_id:?, user_name:?; "check occupant");

		self.0
		.get(leader_id)
		.map(|booking| booking.occupant == *user_name)
	}
}

#[derive(Debug, Clone)]
struct Record {
	occupant: UserName,
	booked_at: Instant,
	accumulated_runtime: Duration,
	running_since: Option<Instant>
}

impl Record {
	fn new(occupant: UserName) -> Self {
		Self {
			occupant,
			booked_at: Instant::now(),
			accumulated_runtime: Duration::ZERO,
			running_since: None
		}
	}

	fn toggle_running(&mut self) {
		if let Some(started_at) = self.running_since.take() {
			self.accumulated_runtime += started_at.elapsed();
		} else {
			self.running_since = Some(Instant::now());
		}
	}

	const fn is_running(&self) -> bool {
		self.running_since.is_some()
	}

	fn total_runtime(&self) -> Duration { 
		self
		.running_since
		.as_ref()
		.map(Instant::elapsed)
		.unwrap_or_default()
		+ self.accumulated_runtime
	}
}