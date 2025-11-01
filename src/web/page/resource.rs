use maud::*;
use tap::Pipe;
use warp::reply::Response;

use super::button;
use crate::web::fab_api::object::Machine;

pub fn resource(resource: &Machine) -> Response {
	use crate::web::fab_api::object::Usage::*;
	
	let (status_style, button_class, button_text) = match resource.usage {
	    Free     => ("color: green", "button-toggle"              , "Claim"  ),
	    Yours    => ("color: gold" , "button-toggle"              , "Release"),
	    Occupied => ("color: red"  , "button-toggle button-danger", "Reset"  ),
	    Unknown  => ("color: gray" , "button-toggle"              , "???"    )
	};
	
	html! {
		(DOCTYPE)
        link rel="stylesheet" href="/style.css";
        meta charset="utf-8";
        
        div class="top" {
            div { (button("<--", "/", "button-back")) }
            h1 { (resource.name) }
            p { (resource.description) }
        }
        
        h1 style=(status_style) { (format!("{:?}", resource.usage)) }

        (button(button_text, &format!("/{}/toggle", resource.urn), button_class))
	}
	.into_string()
	.pipe(|html| Response::new(html.into()))
}