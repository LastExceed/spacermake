use std::collections::HashMap;

use anyhow::anyhow;
use chrono::prelude::*;
use tokio::sync::RwLock;

use crate::*;

type UserId = String;

pub struct Service {
    runtime_tracking: Arc<runtime_tracking::Service>,
    support: Arc<support::Service>,
    
    bookings: RwLock<HashMap<LeaderId, Booking>>
}

impl Service {
    pub(super) fn new(_config: &cfg::Main, runtime_tracking: Arc<runtime_tracking::Service>, support: Arc<support::Service>) -> Self {
        Self {
            runtime_tracking,
            support,

            bookings: RwLock::default()
        }
    }
    
    pub async fn try_update(&self, leader_id: &LeaderId, user: &str, new_state: bool) -> anyhow::Result<()> {
        if new_state {
            self.try_book(leader_id, user).await?;
        } else { 
            self.try_release(leader_id, user).await?;
        }

        self.support.update(leader_id, Scope::Booktime, new_state).await;
        
        Ok(())
    }
    
    async fn try_book(&self, leader_id: &LeaderId, user: &str) -> anyhow::Result<()> {
        let mut bookings = self.bookings.write().await;
        
        if bookings.contains_key(leader_id) {
            return Err(anyhow!("occupied"));
        }
        
        let new =
            Booking {
                user: user.to_owned(),
                since: Local::now()
            };
        
        bookings.insert(leader_id.clone(), new);

        Ok(())
    }
    
    async fn try_release(&self, leader_id: &LeaderId, user: &str) -> anyhow::Result<()> {
        let mut bookings = self.bookings.write().await;
        
        let Some(booking) = bookings.get(leader_id)
        else {
            return Err(anyhow!("unoccupied"));
        };
        
        if booking.user != user {
            return Err(anyhow!("not yours"));
        }
        
        let Ok(runtime) = self.runtime_tracking.take(leader_id)
        else {
            return Err(anyhow!("still running"))
        };
    
        // todo!("accounting");
        
        _ = bookings.remove(leader_id);
        
        Ok(())
    }
    
    pub async fn status_for(&self, leader_id: &LeaderId, user: &str) -> Option<bool> {
        self
        .bookings
        .read()
        .await
        .get(leader_id)
        .map(|booking| booking.user == user)
    }
}

#[derive(Debug, Clone)]
struct Booking {
    user: UserId,
    since: DateTime<Local>
}