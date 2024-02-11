use std::ops::Sub;
use std::time::{Duration, Instant};

use boolinator::Boolinator;
use rumqttc::EventLoop;
use rumqttc::Event::Incoming;
use rumqttc::Packet::Publish;

use crate::{State, Listener, BOOKING_TOPIC, SLAVES_BY_MASTER, SLAVE_PROPERTIES};
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

            self.on_publish(publish).await;
        }
    }

    async fn on_publish(&mut self, publish: rumqttc::Publish) {
        //pretty ugly, cant figure out a clean way to do this
        let Ok(payload) = String::from_utf8(publish.payload.clone().into())
            else {
                log_debug(&publish.topic, &format!("{:?}", &publish.payload), Err("non-utf8 payload"))
                    .expect("debug log failed");

                return;
            };
        //end of ugly

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

        println!("info: {user} {status} {machine}");

        Ok(())
    }

    #[allow(clippy::ptr_arg)] //false positive
    async fn try_book(&mut self, machine: &String, user: &String) -> Result<(), &'static str> {
        let mut bookings = self.bookings.write().await;
        if bookings.contains_key(machine) {
            return Err("machine got double-booked");
        }
        bookings.insert(machine.clone(), Booking::new(user.clone()));
        drop(bookings);
        self.update_slaves(machine, false, true, true).await
    }

    async fn try_release(&mut self, machine: &String) -> Result<(), &'static str> {
        let mut booking = self
            .bookings
            .write()
            .await
            .remove(machine)
            .ok_or("released unbooked machine")?;

        let was_running = booking.track(false);
        self.update_slaves(machine, was_running, true, false).await?;

        machinelog(machine, &booking)
            .expect("machine log failed");

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

        self.update_slaves(machine, true, false, power).await?;

        println!("info: {machine} got turned {power_string}");

        Ok(())
    }

    pub async fn update_slaves(&mut self, master: &String, short_slaves: bool, long_slaves: bool, power: bool) -> Result<(), &'static str> {
        let slaves_used_by_others = self
            .bookings
            .read()
            .await
            .iter()
            .filter(|(_, booking)| booking.is_running())
            .flat_map(|(machine, _)| &SLAVES_BY_MASTER[machine]) //todo: error handing
            .cloned()
            .collect();

        let slaves_to_update = SLAVES_BY_MASTER
            .get(master)
            .ok_or("unknown master")?
            .sub(&slaves_used_by_others)
            .into_iter()
            .filter(|slave| if SLAVE_PROPERTIES[slave][0] { long_slaves } else { short_slaves });

        for slave in slaves_to_update {
            if !power && SLAVE_PROPERTIES[&slave][1] {
                let shutdown_timestamp = Instant::now() + Duration::from_secs(30);
                self.scheduled_shutdowns.write().await.push_back((shutdown_timestamp, slave));
                continue;
            }

            self.update_power_state(&slave, power).await;
        }

        Ok(())
    }
}