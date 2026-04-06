use itertools::Itertools;

use super::BookingIndex;
use crate::settings::ResourceSettings;
use crate::{ResourceId, app_settings, mqtt};

mod shutdown;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
	Booktime,
	Runtime
}

pub struct State<'mqtt, 'shutdown> {
	mqtt_write: &'mqtt mqtt::write::State,
	shutdown: &'shutdown shutdown::State
}

impl State<'_, '_> {
	pub async fn update_all(
		&mut self,
		bookings: &BookingIndex,
		resource: &ResourceId,
		kind: Kind,
		new_state: bool
	) {
		let resource_settings = &app_settings().resources;

		let deps_in_use = get_all_in_use(resource_settings, bookings).collect_vec();
		let deps_to_toggle =
			match kind {
				Kind::Booktime => &resource_settings[resource].dependencies_booktime,
				Kind::Runtime  => &resource_settings[resource].dependencies_runtime,
			}
			.iter()
			.filter(|dep| !deps_in_use.contains(dep));

		for dependency in deps_to_toggle {
			self.update_one(resource_settings, dependency, new_state).await;
		}
	}

	async fn update_one(
		&mut self,
		resource_settings: &ResourceSettings,
		resource_id: &ResourceId,
		new_state: bool
	) {
		if !new_state {
			self.shutdown.schedule(resource_id);
			return;
		}
		
		if self.shutdown.is_scheduled(resource_id) {
			self.shutdown.cancel(resource_id);
			return;
		}

		self.mqtt_write.set_power_state(resource_id, true).await;
	}
}

fn get_all_in_use<'settings>(
	resource_settings: &'settings ResourceSettings,
	bookings: &BookingIndex
) -> impl Iterator<Item = &'settings ResourceId> {
	let booktime_deps =
		bookings
		.keys()
		.flat_map(|resource|
			resource_settings
			[resource]
			.dependencies_booktime
			.iter()
		);
		
	let runtime_deps = 
		bookings
		.iter()
		.filter(|(_resource, booking)| booking.is_running())
		.flat_map(|(resource, _booking)|
			resource_settings
			[resource]
			.dependencies_runtime
			.iter()
		);
	
	booktime_deps.chain(runtime_deps)
}