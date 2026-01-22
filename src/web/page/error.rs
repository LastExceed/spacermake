use maud::*;
use tap::Pipe;
use warp::reply::Response;
use super::button;

pub fn error(error: &anyhow::Error) -> Response {
    html! {
        (DOCTYPE)
        meta charset="utf-8";

        body class="error-page" {
            (format!("{error:#?}"))
            (button("Go Back", "/", ""))
        }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}