use std::sync::Arc;

use futures::future::join;
use tap::Pipe;
use tokio::sync::RwLock;

use self::settings::AppSettings;

mod settings;
mod web;
mod accounting;
mod mqtt;

pub type ResourceId = String;

#[tokio::main]
async fn main() {
	log::info!("start");

	let (client, event_loop) = mqtt::create_client().await;
	let client = client.pipe(RwLock::new).pipe(Arc::new);
	
	join(
		web::start(Arc::clone(&client)),
		// listen power states
		// runtime displays
		// scheduled shutdowns
	).await;
}

fn app_settings() -> &'static AppSettings {
	todo!()
}