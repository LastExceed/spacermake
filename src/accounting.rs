use std::collections::HashMap;

use chrono::prelude::*;
use chrono::TimeDelta;
use tap::Pipe;

use crate::{ResourceId, mqtt};

use self::dependencies::Kind;

mod dependencies;
mod runtime_display;

type UserId = String;
pub type BookingIndex = HashMap<ResourceId, Booking>;

pub struct State<'mqtt, 'shutdown> {
    bookings: BookingIndex,
    dependencies: dependencies::State<'mqtt, 'shutdown>,
}

impl State<'_, '_> {
    pub async fn try_update(&mut self, resource: &ResourceId, user: &str, new_state: bool) -> Result<(), &'static str> {    
        if new_state {
            self.try_book(resource, user).await?;
        } else { 
            self.try_release(resource, user).await?;
        }
        
        self.dependencies.update_all(&self.bookings, mqtt_write, resource, Kind::Booktime, new_state).await;
        
        Ok(())
    }

    async fn try_book(&mut self, resource: &ResourceId, user: &str) -> Result<(), &'static str> {
        if self.bookings.contains_key(resource) {
            return Err("occupied");
        }
        self.bookings.insert(resource.clone(), Booking::new(user.to_owned()));

        Ok(())
    }

    async fn try_release(&mut self, resource: &ResourceId, user: &str) -> Result<(), &'static str> {
        let Some(booking) = self.bookings.remove(resource)
        else {
            return Err("unoccupied");
        };
        
        if booking.user != user {
            return Err("not yours");
        }

        if booking.is_running() {
            return Err("still running");
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Booking {
    pub user: UserId,
    pub booked_since: DateTime<Local>,
    pub running_since: Option<DateTime<Local>>,
    pub past_runtime: TimeDelta
}

impl Booking {
    pub fn new(user: String) -> Self {
        Self {
            user,
            booked_since: Local::now(),
            running_since: None,
            past_runtime: TimeDelta::zero()
        }
    }

    pub fn track_activity(&mut self, power: bool) -> bool {
		if self.is_running() == power {
            return false;
        }
		
		let now = Local::now();

        if power {
            self.running_since = Some(now);
        } else {
            self.past_runtime +=
				self
                .running_since
                .take()
                .unwrap() //SAFETY: guaranteed by .is_running()
                .pipe(|startup| now.signed_duration_since(startup));
        }

        true
    }

    pub const fn is_running(&self) -> bool {
        self.running_since.is_some()
    }

    pub fn total_runtime(&self) -> TimeDelta {
        let mut total = self.past_runtime;

        if let Some(startup) = self.running_since {
            total += Local::now().signed_duration_since(startup);
        }

        total
    }
}