use maud::{Markup, html};

mod debug;
mod error;
mod overview;
mod resource;

pub use debug::debug;
pub use error::error;
pub use overview::overview;
pub use resource::resource;

fn button(text: &str, dst: &str, class: &str) -> Markup {
    html! {
        form action=(dst) {
            button type="submit" class=(class) { (text) }
        }
    }
}