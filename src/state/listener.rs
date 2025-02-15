use std::{collections::HashSet, ops::Sub};
use std::time::{Duration, Instant};

use boolinator::Boolinator;
use colour::{cyan_ln, dark_grey_ln, red_ln};
use rumqttc::EventLoop;
use rumqttc::Event::Incoming;
use rumqttc::Packet::Publish;

use crate::{State, Listener, BOOKING_TOPIC, SLAVES_BY_MASTER, SLAVE_PROPERTIES};
use crate::utils::index;
use crate::utils::get_power_state;
use crate::utils::logs::{log_debug, machinelog};
use crate::utils::booking::Booking;

impl State<Listener> {
    pub async fn run(mut self, mut event_loop: EventLoop) {
        loop {
            let Incoming(Publish(publish)) = event_loop
                .poll()
                .await
                .expect("notification error")
                else { continue };

            dark_grey_ln!("publish received");
            self.on_publish(publish).await;
        }
    }

    async fn on_publish(&mut self, publish: rumqttc::Publish) {
        let Ok(payload) = String::from_utf8(publish.payload.clone().into()) else {
            red_ln!("publish with non-utf8 payload received - {:?}", publish.payload);
            return;
        };
        dark_grey_ln!("payload: {payload}");

        let result = self.handle_payload(&publish.topic, &payload).await;

        log_debug(&publish.topic, &payload, result)
            .expect("debug log failed")
    }

    async fn handle_payload(&mut self, topic: &str, payload: &str) -> Result<(), &str> {
        let splits: Result<[_; 3], _> = topic
            .split('/')
            .collect::<Vec<_>>()
            .try_into();

        match splits {
            Ok(["tele", machine_name, "MARGINS"])
                => self.on_machine_activity(payload, &machine_name.into()).await,

            _ if topic == BOOKING_TOPIC
                => self.on_booking_change(payload).await,

            _   => Err("unknown topic")
        }
    }

    async fn on_booking_change(&mut self, payload: &str) -> Result<(), &'static str> {
        let [machine, user, status] = payload
            .split(';')
            .map(String::from)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| "unexpected data count in payload")?;

        match status.as_str() {
            "booked" => self.try_book(&machine, &user).await?,
            "released" => self.try_release(&machine).await?,
            _ => return Ok(()) //ignore other statuses
        }

        cyan_ln!("{user} {status} {machine}");

        Ok(())
    }

    #[allow(clippy::ptr_arg)] //false positive
    async fn try_book(&mut self, machine: &String, user: &String) -> Result<(), &'static str> {
        dark_grey_ln!("booking {machine}");
        let mut bookings = self.bookings.write().await;
        if bookings.contains_key(machine) {
            return Err("machine got double-booked");
        }
        bookings.insert(machine.clone(), Booking::new(user.clone()));
        drop(bookings);
        self.update_slaves(machine, false, true, true).await
    }

    async fn try_release(&mut self, machine: &String) -> Result<(), &'static str> {
        dark_grey_ln!("releasing {machine}");
        let mut booking = self
            .bookings
            .write()
            .await
            .remove(machine)
            .ok_or("released unbooked machine")?;

        machinelog(machine, &booking)
            .expect("machine log failed");

        let was_running = booking.track(false);
        self.update_slaves(machine, was_running, true, false).await?;

        Ok(())
    }

    async fn on_machine_activity(&mut self, payload: &str, machine: &String) -> Result<(), &'static str> {
        let power_string = get_power_state(payload)?;

        let (power, err) =
            match power_string.as_str() {
                "ON"  => (true, "machine was turned on while already running"),
                "OFF" => (false, "machine was turned off without running in the first place"),
                _     => return Err("unknown power state")
            };

        self.bookings
            .write()
            .await
            .get_mut(machine)
            .ok_or("received activity from unbooked machine")?
            .track(power)
            .as_result((), err)?;

        cyan_ln!("info: {machine} got turned {power_string}");

        self.update_slaves(machine, true, false, power).await?;

        Ok(())
    }

    pub async fn update_slaves(&mut self, master: &String, short_slaves: bool, long_slaves: bool, power: bool) -> Result<(), &'static str> {
        dark_grey_ln!("updating slaves...");

        let fallback = HashSet::new();

        let slaves_used_by_others = self
            .bookings
            .read()
            .await
            .iter()
            .filter(|(other, _booking)| *other != master)
            .flat_map(|(machine, booking)|
                SLAVES_BY_MASTER
                    .get(machine)
                    .unwrap_or(&fallback) // machine being unknown already got logged when it got turned on, so we can ignore it here
                    .iter()
                    .filter(|slave| booking.is_running() || SLAVE_PROPERTIES[*slave][index::RUNS_CONTINUOUSLY])
            )
            .cloned()
            .collect();

        let slaves_to_update = SLAVES_BY_MASTER
            .get(master)
            .ok_or("unknown master")?
            .sub(&slaves_used_by_others)
            .into_iter()
            .filter(|slave| if SLAVE_PROPERTIES[slave][index::RUNS_CONTINUOUSLY] { long_slaves } else { short_slaves });

        for slave in slaves_to_update {
            if SLAVE_PROPERTIES[&slave][index::NEEDS_TRAILING_TIME] {
                if power {
                    self.cancel_scheduled_shutdown(&slave).await;
                } else {
                    self.schedule_shutdown(slave).await;
                    continue;
                }
            }

            self.set_power_state(&slave, power).await;
        }

        Ok(())
    }

    async fn schedule_shutdown(&self, slave: String) {
        dark_grey_ln!("scheduling delayed shutdown for {}", slave);

        let shutdown_timestamp = Instant::now() + Duration::from_secs(30);

        self.scheduled_shutdowns
            .write()
            .await
            .push_back((shutdown_timestamp, slave));
    }

    async fn cancel_scheduled_shutdown(&self, slave: &String) {
        let mut schedule = self.scheduled_shutdowns.write().await;

        if let Some(index) = schedule.iter().position(|(_, name)| name == slave) {
            dark_grey_ln!("cancelling scheduling shutdown for {}", slave);
            schedule.remove(index);
        }
    }
}