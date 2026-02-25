use itertools::Itertools;
use maud::*;
use crate::web::fab_api::object::{Machine, Usage};
use crate::web::page::button;

pub fn overview(resources: &[Machine], hide_unbooked: bool) -> Markup {
    let group_map =
        resources
        .iter()
        .filter(|resource| !hide_unbooked || resource.usage == Usage::Yours)
        .map(|res| (res.category.clone(), res))
        .into_group_map();
    
    html! {
        header {}

        main class="overview" {
            details {
                summary class="fake-button" { "SCAN QR-CODE" }
                p class="notice" {}
            }
        
            @for (category, categorized_resources) in group_map {
                h2 { (category) }
                @for resource in categorized_resources {
                    div class="resource" {
                        h3 { (resource.name) }
                        p class=(format!("status-{:?}", resource.usage)) {}
                        (button("➔", &format!("/{}", resource.urn), "goto"))
                    }
                }
            }
        }
    }
}