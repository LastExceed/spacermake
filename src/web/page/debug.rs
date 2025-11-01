use itertools::Itertools;
use maud::*;
use tap::Pipe;
use warp::reply::Response;

use crate::web::fab_api::object::Machine;

pub fn debug(resources: &[Machine]) -> Response {
    let group_map =
        resources
        .iter()
        .into_group_map_by(|resource| resource.category.clone());

    html! {
        (DOCTYPE)
        meta charset="utf-8";

        table {
            thead {
                tr {
                    th { "Category" }
                    th { "Name" }
                    th { "ID" }
                    th { "URN" }
                    th { "Description" }
                    th { "State" }
                    th { "Button" }
                }
            }
            @for (category, group) in group_map.into_iter().sorted_by_key(|(cat, _grp)| cat.to_owned()) {
                tbody {
                    @for resource in group.into_iter().sorted_by_key(|resource| &resource.name) {
                        tr {
                            td { (category) }
                            td { (resource.name) }
                            td { (resource.id) }
                            td { (resource.urn) }
                            td { (resource.description) }
                            td { (format!("{:?}", resource.usage)) }
                            td {
                                form action=(format!("/{}", resource.urn)) {
                                    button type="submit" { "View" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}
