use maud::*;

mod debug;
mod error;
mod overview;
mod resource;

pub use debug::debug;
pub use error::error;
pub use overview::overview;
pub use resource::resource;
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

fn button(text: &str, dst: &str, class: &str) -> Markup {
    html! {
        form action=(dst) {
            button type="submit" class=(class) { (text) }
        }
    }
}