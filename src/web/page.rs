use std::fmt::Display;
use maud::*;

pub mod error;
pub mod overview;
pub mod resource;

use strum::Display;
use tap::Pipe;
use warp::reply::Response;

pub fn template(content: &Markup) -> Response {
	html! {
		(DOCTYPE)
		link rel="stylesheet" href="/style.css";
		meta charset="utf-8";

		(content)
	}
	.into_string()
	.pipe(|html| Response::new(html.into()))
}

fn button(
	text   : &str,
	dst    : impl Display,
	class  : &str,
	enabled: bool
) -> Markup {
	// there gotta be a better way to do this
	let inner =
		if enabled {
			html! { button type="submit" class=(class)          { (text) } }
		} else {
			html! { button type="submit" class=(class) disabled { (text) } }
		};

	html! {
		form action={ "/"(dst) } {
			(inner)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
enum Status {
	Free,
	Yours,
	Occupied
}

impl From<Option<bool>> for Status {
	fn from(value: Option<bool>) -> Self {
		match value {
			Some(true ) => Self::Yours,
			Some(false) => Self::Occupied,
			None        => Self::Free
		}
	}
}