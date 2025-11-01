use maud::*;
use itertools::Itertools;
use tap::Pipe;
use warp::reply::Response;

use super::button;
use crate::web::fab_api::object::{Machine, Usage};

pub fn overview(resources: &[Machine]) -> Response {
    let filtered =
        resources
        .iter()
        .filter(|resource| resource.usage == Usage::Yours)
        .collect_vec();
    
    html! {
        (DOCTYPE)
        link rel="stylesheet" href="/style.css";
        meta charset="utf-8";

        h1 class="overview" { "Overview" }
        
        ul {
            @for resource in filtered {
                (button(&resource.name, &format!("/{}", resource.urn), "button-list"))
            }
        }
        
        i { ("iro illwuf evernas orgetfix ouyah ariemer. hankton ouyah orful verythingers /3<~%") }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}