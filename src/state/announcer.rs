use std::time::{Duration, Instant};

use colour::{blue_ln, red_ln};
use futures::join;
use futures::future::join_all;
use rumqttc::QoS;
use tap::Pipe;
use tokio::time::sleep;

use crate::MACHINE_IDS;
use crate::utils::{create_display_update_message, minute_mark};
use crate::{Announcer, State};

impl State<Announcer> {
    pub async fn run(self) {
        loop {
            join!(
                self.update_all_runtime_displays(),
                self.perform_scheduled_shutdowns()
            );

            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn update_all_runtime_displays(&self) {
        self.bookings
            .read()
            .await
            .iter()
            .filter_map(|(machine, booking)| {
                if !booking.is_running() || !minute_mark(booking.total_runtime()) {
                    return None;
                }

                blue_ln!("updating display of {machine}");

                let Some(id) = MACHINE_IDS.get(machine) else {
                    red_ln!("error: no ID found for {machine}");
                    return None;
                };

                let future = self.update_runtime_display(id, booking.total_runtime());
                Some(future)
            })
            .pipe(join_all)
            .await;
    }

    async fn update_runtime_display(&self, machine_id: &str, runtime: Duration) {
        self.client
            .read()
            .await
            .publish(
                format!("/cmnd/reader/{machine_id}"),
                QoS::AtMostOnce,
                false,
                create_display_update_message(runtime)
            )
            .await
            .expect("failed to publish display update");
    }

    async fn perform_scheduled_shutdowns(&self) {
        let now = Instant::now();
        let mut schedule = self.scheduled_shutdowns.write().await;
        while let Some((time, machine)) = schedule.front() {
            if time > &now {
                break;
            }
            blue_ln!("performing scheduled shutdown of {machine}");

            self.set_power_state(machine, false).await;
            schedule.pop_front();
        }
    }
}