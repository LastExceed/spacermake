use std::collections::HashMap;
use std::time::{Duration, Instant};

use strum::EnumIs;

pub type Map = HashMap<Id, State>;
pub type Id = String;

#[derive(EnumIs)]
pub enum State {
    Running,
    ShuttingDown(Instant),
    Sleeping
}

impl State {
    fn try_abort(&mut self) -> bool {
        if self.is_shutting_down() {
            *self = Self::Running;
            true
        } else {
            false
        }
    }
}

impl From<bool> for State {
    fn from(value: bool) -> Self {
        if value { Self::Running }
        else { Self::Sleeping }
    }
}

pub struct Configuration {
    pub name                 : String,
	pub shutdown_delay       : f32,
    pub mqtt_topic           : String,
    pub mqtt_payload_startup : String,
    pub mqtt_payload_shutdown: String,
}

impl super::Supporter {
    pub async fn begin_shutdown(&self) {
        let time = Instant::now() + Duration::from_secs_f32(self.configuration.shutdown_delay);

        let mut state = self.state.write().await;        
        if !state.is_running() {
            todo!("invalid state");
        }
        *state = State::ShuttingDown(time);
    }
}