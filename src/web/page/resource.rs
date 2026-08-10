use maud::*;
use tap::{Conv, Pipe};
use warp::reply::Response;

use crate::{App, cfg};
use crate::newtypes::{LeaderId, UserName};

use super::{Status, button, template};

pub fn render(
	app       : &App,
	id        : &LeaderId,
	user      : &UserName,
	leader_cfg: &cfg::Leader,
	has_perm  : bool
) -> Response {
	let name = &id.0;
	let description = &leader_cfg.description;

	let status =
		app
		.bookings
		.is_the_occupant(id, user)
		.conv::<Status>();

	let style_class = format!("status-{status}");

	html! {
		div class="top" {
			div { (button("<--", "", "button-back", true)) }
			h1 { (id.0) }
			p { (description) }
		}

		h1 class=(style_class) { "" } // set via css

		(button(
			"",
			format!("{name}/toggle"),
			&style_class,
			has_perm || status == Status::Yours
		))
	}
	.pipe_ref(template)
}