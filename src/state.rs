use std::time::Instant;
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};

use colour::dark_grey_ln;
use rumqttc::{AsyncClient, QoS};
use tokio::sync::RwLock;

use crate::utils::index;
use crate::utils::booking::Booking;
use crate::SLAVE_PROPERTIES;

mod announcer;
mod listener;

//markers
pub struct Listener;
pub struct Announcer;

pub struct State<Kind> {
    pub kind: Kind,
    pub client: Arc<RwLock<AsyncClient>>,
    pub bookings: Arc<RwLock<HashMap<String, Booking>>>,
    pub scheduled_shutdowns: Arc<RwLock<VecDeque<(Instant, String)>>>
}

impl<Kind> State<Kind> {
    pub fn new(kind: Kind, client: AsyncClient) -> Self {
        Self {
            kind,
            client: Arc::new(RwLock::new(client)),
            bookings: Default::default(),
            scheduled_shutdowns: Default::default()
        }
    }

    pub fn duplicate_as<NewKind>(&self, kind: NewKind) -> State<NewKind> {
        State {
            kind,
            client: Arc::clone(&self.client),
            bookings: Arc::clone(&self.bookings),
            scheduled_shutdowns: Arc::clone(&self.scheduled_shutdowns)
        }
    }

    //probably doesn't belong here, dunno where else to put it
    async fn set_power_state(&self, machine: &str, new_state: bool) {
        dark_grey_ln!("set power state - {machine} {new_state}");
        let is_tasmota = SLAVE_PROPERTIES[machine][index::IS_TASMOTA];
        let topic =
            if is_tasmota {
                format!("cmnd/{machine}/Power")
            } else {
                format!("shellies/{machine}/relay/0/command")
            };

        #[allow(clippy::collapsible_else_if)]
        let payload =
            if is_tasmota {
                if new_state { b"ON".as_slice() } else { b"OFF".as_slice() }
            } else {
                if new_state { b"on".as_slice() } else { b"off".as_slice() }
            };

        dark_grey_ln!("publishing\n  topic: {topic}\n  payload: {payload:?}");
        self.client
            .read()
            .await
            .publish(topic, QoS::AtMostOnce, false, payload)
            .await
            .expect("failed to publish");
    }
}