use std::sync::Arc;
use std::time::Duration;

use futures::future::{join, join3};
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use self::cfg::Main;
use self::services::*;
use self::services::support::Scope;

mod cfg;
mod services;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct SupporterId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct LeaderId(String);

#[tokio::main]
async fn main() {
	log::info!("start");
	
	let config = Main::load();
	config.validate();

	let sc = services::Collation::new(&config).await;
	
	join3(
		sc.web.run(),
		listen(sc.mqtt_read, Arc::clone(&sc.runtime_tracking), sc.support),
		periodic(sc.machine_control, sc.runtime_tracking)
	)
	.await;
}

async fn listen(
	mut mqtt_read: mqtt::read::Service,
	runtime_tracking: Arc<runtime_tracking::Service>,
	support: Arc<support::Service>,
) -> ! {
	loop {
		let (leader_id, new_state) = mqtt_read.next_power_state().await;

		runtime_tracking.track(&leader_id, new_state);
		support.update(&leader_id, Scope::Runtime, new_state).await;
	}
}

async fn periodic(machine_control: Arc<machine_control::Service>, runtime_tracking: Arc<runtime_tracking::Service>) -> ! {
	// gotta figure out how to properly use a DelayQueue instead
	let mut interval = interval(Duration::from_secs(1));

	loop {
		interval.tick().await;

		join(
			machine_control.execute_due_shutdowns(),
			runtime_tracking.refresh_displays()			
		).await;
	}
}