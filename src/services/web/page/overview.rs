use std::collections::HashMap;

use futures::prelude::future::join_all;
use maud::*;
use warp::reply::Response;

use crate::{LeaderId, services::*};

use super::{button, template};

pub async fn overview(booking_service: &booking::Service, categories: &HashMap<&String, Vec<&LeaderId>>, user: &str) -> Response {
    let statuses =
        categories
        .iter()
        .flat_map(|(_category, leader_ids)| leader_ids)
        .map(async |leader_id| (leader_id, booking_service.status_for(leader_id, user).await))
        .pipe(join_all)
        .await
        .into_iter()
        .map(|(leader_id, value)| {
            let status = match value {
                Some(true ) => "Yours",
                Some(false) => "Occupied",
                None        => "Free"
            };
            
            (leader_id, status)
        })
        .collect::<HashMap<_, _>>();
    
    html! {
        header {}

        main class="overview" {
            details {
                summary class="fake-button" { "SCAN QR-CODE" }
                p class="notice" {}
            }
        
            @for (category, leader_ids) in categories {
                h2 { (category) }
                @for leader_id in leader_ids {
                    div class="resource" {
                        h3 { (leader_id.0) }
                        p class=(format!("status-{:?}", statuses[leader_id])) {}
                        (button("➔", &format!("/{}", leader_id.0), "goto"))
                    }
                }
            }
        }
    }
    .pipe_ref(template)
}