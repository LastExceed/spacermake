use maud::*;
use tap::Pipe;
use warp::reply::Response;
use super::button;

pub fn error(error: &anyhow::Error) -> Response {
    html! {
        (DOCTYPE)
        link rel="stylesheet" href="/style.css";
        meta charset="utf-8";

        body class="error-page" {
            h1 { (format!("error:\n\n{error:#?}")) }
            (button("Go Back", "/", ""))
        }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}