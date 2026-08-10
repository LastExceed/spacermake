use std::collections::HashMap;

use itertools::Itertools;
use maud::*;
use tap::prelude::Pipe;
use warp::reply::Response;

use crate::{App, cfg};
use crate::newtypes::{LeaderId, UserName};

use super::{Status, button, template};

pub fn render(cfg: &cfg::Main, app: &App, user_name: &UserName) -> Response {
	let to_display = get_machines(app, cfg, user_name);

	html! {
		header {}

		main .overview {
			details {
				summary class="fake-button" { "SCAN QR-CODE" }
				p class="notice" {}
			}

			@for (category, entries) in to_display {
				h2 { (category) }
				@for (leader_id, status) in entries {
					.resource {
						h3 { (leader_id.0) }
						p class=(format!("status-{status}")) {}
						(button("➔", &leader_id.0, "goto", true))
					}
				}
			}
		}
	}
	.pipe_ref(template)
}

fn get_machines<'cfg>(app: &App, cfg: &'cfg cfg::Main, user_name: &UserName) -> HashMap<&'cfg String, Vec<(&'cfg LeaderId, Status)>> {	
	cfg
	.permissions_of(user_name)
	.map(move |(leader_id, _free_use)| {
		let category = &cfg.leaders[leader_id].display_category;
		let status = app.bookings.is_the_occupant(leader_id, user_name).into();

		(category, (leader_id, status))
	})
	.into_group_map()
}