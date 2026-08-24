#![expect(clippy::cargo_common_metadata, reason = "todo")]

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use futures::prelude::*;
use tap::prelude::Pipe;
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::newtypes::*;

use self::booking::ToggleOutcome;
use self::custom::verein_online;
use self::custom::verein_online::model::UserId;
use self::iot::Observer;
// use self::web::auth::VerifyLogin;

mod booking;
mod cfg;
mod custom;
mod iot;
mod logging;
mod newtypes;
mod support;
mod web;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn dir() -> PathBuf {
	dirs::data_dir()
	.expect("data dir not defined")
	.join(APP_NAME)
}

#[tokio::main]
async fn main() {
	logging::Logger::init();

	// let admin_username = "christoph.beckmann";
	// let admin_password = "S!M5f^!#otf!sM";
	// let pleb_username = "bristoph.chreckmann";
	// let pleb_password = "tD@o$k0U3m0Y@G";
	
	// let verein = "Makerspace_Bocholt_gUG";
	// let api = "VerifyLogin";
	
	// let sys_user = admin_username;
	// let sys_pw_hash = md5::compute(admin_password);
	
	// let payload = VerifyLogin {
	// 	user: pleb_username,
	// 	password: pleb_password,
	// 	result: "id"
	// };

	// println!("{}", serde_json::to_string(&payload).unwrap());
	
	// let rsp =
	// 	reqwest
	// 	::Client
	// 	::new()
	// 	.post(format!("https://www.vereinonline.org/{verein}/?api={api}"))
	// 	.header("Authorization", format!("A/{sys_user}/{sys_pw_hash:x}"))
	// 	.json(&payload)
	// 	.send()
	// 	.await
	// 	.unwrap()
	// 	.text()
	// 	.await
	// 	.unwrap();
	
	// println!(">> {rsp}");

	// return;
	
	log::info!("app start");
	if !cfg::init().await.unwrap() {
		log::info!("A config template has been generated (in {}). Customize it, then run again.", cfg::dir().display());
		log::warn!("Exiting due to missing config");
		return;
	}

	let (controller, observer) = iot::new().await;
	let app = App::new(controller).pipe(RwLock::new);

	future::join3(
		listen(observer, &app),
		periodic(&app),
		web::host(&app)
	).await;
}

async fn listen(mut observer: Observer, app: &RwLock<App>) -> ! {
	log::info!(app:?; "begin listening");

	loop {
		let (leader_id, new_state) = observer.next_power_state().await;
		log::info!(leader_id:?, new_state:%; "power state changed - leader_id: {}, new_state: {}", leader_id.0, new_state);
		
		app.write().await.on_power_activity(&leader_id, new_state).await;
	}
}

async fn periodic(app: &RwLock<App>) -> ! {
	log::info!(app:?; "begin ticking");

	let mut cfg_watcher = cfg::Watcher::new().await;
	let mut interval = interval(Duration::from_secs(1));

	loop {
		_ = interval.tick().await;
		
		cfg_watcher.refresh().await;
		app.write().await.on_tick().await;
	}
}

#[derive(Debug)]
struct App {
	support: support::Network,
	bookings: booking::Registry,
	vo_client: verein_online::ApiClient
}

impl App {
	pub fn new(controller: iot::Controller) -> Self {
		log::trace!(controller:?; "create app state");

		Self {
			support: support::Network::new(controller),
			bookings: booking::Registry::default(),
			vo_client: verein_online::ApiClient::new()
		}
	}

	pub async fn on_booking_request(&mut self, leader_id: &LeaderId, user_name: &UserName, user_id: UserId) -> Result<(), booking::ToggleError> {
		log::trace!(leader_id:?, user_name:?; "on_toggle_booking_request");

		let outcome = self.bookings.try_toggle(leader_id, user_name)?;

		self.update_supporters().await;

		if let ToggleOutcome::Released { booked_time, runtime } = outcome {
			let result = custom::Accountant.write_bill(user_name, user_id, leader_id, booked_time, runtime).await;
			if let Err(error) = result {
				log::error!(error:?; "failed to write bill");
			}
		}

		Ok(())
	}

	pub async fn on_power_activity(&mut self, leader_id: &LeaderId, new_state: bool) {
		log::trace!(leader_id:?; "on power activity");

		self.bookings.track_activity(leader_id, new_state);
		self.update_supporters().await;
	}

	async fn update_supporters(&mut self) {
		log::trace!("update supporters");

		let cfg_guard = cfg::INSTANCE.read().await;
		let mut needed_supporters = HashSet::new();
		
		for (leader_id, is_running) in self.bookings.all_states() {
			let leader_cfg = &cfg_guard.leaders[leader_id];

			needed_supporters.extend(&leader_cfg.dependencies_booktime);

			if is_running {
				needed_supporters.extend(&leader_cfg.dependencies_runtime);
			}
		}

		self.support.update_running_set(needed_supporters).await;
	}
	
	pub async fn on_tick(&mut self) {
		// log::trace!("tick");
		
		self.support.shutdown_due().await;
		self.refresh_runtime_displays().await;
	}
	
	async fn refresh_runtime_displays(&self) {
		// log::trace!("refresh_runtime_displays");
		
		let times = self.bookings.active_runtimes();
		self.support.update_screens(times).await;
	}
}