use maud::*;

use super::button;
use crate::web::fab_api::object::Machine;

pub fn resource(resource: &Machine) -> Markup {
	let status_class = format!("status-{:?}", resource.usage);
	
	html! {
		header { (button("<--", "/", "back")) }
		
		main class="resource" {
			h2 { (resource.name) }
			p { (resource.description) }
			
			h1 class=(status_class) {}

			(button("", &format!("/{}/toggle", resource.urn), &status_class))
		}
	}
}