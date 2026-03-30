use maud::*;
use tap::Pipe;
use warp::reply::Response;

use crate::LeaderId;
use crate::services::booking;

use super::button;

pub async fn resource(id: &LeaderId, user: &str, booking_service: &booking::Service) -> Response {
	let name = &id.0;
	
	let (status, status_style, button_class, button_text) =
		match booking_service.status_for(id, user).await {
			None        => ("Free"    , "color: green", "button-toggle"              , "Claim"  ),
			Some(true ) => ("Yours"   , "color: gold" , "button-toggle"              , "Release"),
			Some(false) => ("Occupied", "color: red"  , "button-toggle button-danger", "Reset"  )
		};

	html! {
		(DOCTYPE)
        link rel="stylesheet" href="/style.css";
        meta charset="utf-8";

        div class="top" {
            div { (button("<--", "/", "button-back")) }
            h1 { (name) }
            // p { ("description") }
        }

        h1 style=(status_style) { (status) }

        (button(button_text, &format!("/{name}/toggle"), button_class))
	}
	.into_string()
	.pipe(|html| Response::new(html.into()))
}