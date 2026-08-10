use maud::*;
use tap::Pipe;
use warp::reply::Response;
use super::{button, template};

pub fn render(error: &anyhow::Error) -> Response {
	html! {
		body class="error-page" {
			h1 { (format!("error:\n\n{error:#?}")) }
			(button("Go Back", "", "", true))
		}
	}
	.pipe_ref(template)
}