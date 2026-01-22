use std::time::Instant;
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};

use colour::dark_grey_ln;
use rumqttc::{AsyncClient, QoS};
use tokio::sync::RwLock;

use crate::config::SpacerConfig;
use crate::utils::booking::Booking;

mod announcer;
mod listener;

//markers
pub struct Listener;
pub struct Announcer;

pub struct State<Kind> {
    #[expect(dead_code, reason = "like PhantomData")]
    pub kind: Kind,
    pub config: Arc<SpacerConfig>,
    pub client: Arc<RwLock<AsyncClient>>,
    pub bookings: Arc<RwLock<HashMap<String, Booking>>>,
    pub scheduled_shutdowns: Arc<RwLock<VecDeque<(Instant, String)>>>
}

impl<Kind> State<Kind> {
    pub fn new(kind: Kind, client: AsyncClient, config: Arc<SpacerConfig>) -> Self {
        Self {
            kind,
            config,
            client: Arc::new(RwLock::new(client)),
            bookings: Default::default(),
            scheduled_shutdowns: Default::default()
        }
    }

    pub fn duplicate_as<NewKind>(&self, kind: NewKind) -> State<NewKind> {
        State {
            kind,
            config: Arc::clone(&self.config),
            client: Arc::clone(&self.client),
            bookings: Arc::clone(&self.bookings),
            scheduled_shutdowns: Arc::clone(&self.scheduled_shutdowns)
        }
    }

    //probably doesn't belong here, dunno where else to put it
    async fn set_power_state(&self, machine: &str, new_state: bool) {
        dark_grey_ln!("set power state - {machine} {new_state}");
        let props = &self.config.slave_properties[machine];
        let payload = if new_state { &props.payload_on } else { &props.payload_off };

        dark_grey_ln!("publishing\n  topic: {}\n  payload: {:?}", props.topic, payload);
        self.client
            .read()
            .await
            .publish(&props.topic, QoS::AtMostOnce, false, payload.as_bytes())
            .await
            .expect("failed to publish");
    }
}