use maud::*;
use crate::machine::Machine;
use super::button;

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