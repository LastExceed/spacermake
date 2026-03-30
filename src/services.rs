use std::sync::Arc;

use tap::prelude::Pipe;

use crate::cfg;

pub mod runtime_tracking;
pub mod booking;
pub mod mqtt;
pub mod support;
pub mod machine_control;
pub mod web;

// `Arc`s so we don't have to use ouroboros
#[expect(dead_code, reason = "clarity")]
pub struct Collation {
	pub mqtt_write      : Arc<mqtt::write     ::Service>,
	pub mqtt_read       :     mqtt::read      ::Service,
	pub runtime_tracking: Arc<runtime_tracking::Service>,
	pub machine_control : Arc<machine_control ::Service>,
	pub support         : Arc<support         ::Service>,
	pub booking         : Arc<booking         ::Service>,
	pub web             : Arc<web             ::Service>
}

impl Collation {
	#[expect(clippy::clone_on_ref_ptr, reason = "sufficiently obvious")]
	pub async fn new(config: &cfg::Main) -> Self {
		let (mqtt_read, mqtt_write) = mqtt::create_split(config).await;
		let mqtt_write = Arc::new(mqtt_write);
		// let mqtt_read = Arc::new(mqtt_read);
		
		let runtime_tracking = runtime_tracking::Service::new(config,       mqtt_write.clone()                 ).pipe(Arc::new);
		let machine_control  =  machine_control::Service::new(config,       mqtt_write.clone()                 ).pipe(Arc::new);
		let support          =          support::Service::new(config,  machine_control.clone()                 ).pipe(Arc::new);
		let booking          =          booking::Service::new(config, runtime_tracking.clone(), support.clone()).pipe(Arc::new);
		let web              =              web::Service::new(config,          booking.clone()                 ).pipe(Arc::new);
		
		Self {
			mqtt_write,
			mqtt_read,
			runtime_tracking,
			machine_control,
			support,
			booking,
			web
		}
	}
}