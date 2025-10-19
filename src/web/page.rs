use itertools::Itertools;
use maud::*;
use tap::Pipe;
use warp::reply::Response;

use super::fab_api::object::Machine;

pub fn login() -> Response {
    html! {
        (DOCTYPE)
        form action="/login" method="post" {
            fieldset {
                input type="text"
                    name="username"
                    placeholder="username"
                    required;
                br;
                input type="password"
                    name="password"
                    placeholder="password"
                    required;
                br;
                button type="submit" { "Login" }    
            }
        }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}

pub fn _overview2(machines: &[Machine]) -> Response {
    let group_map = machines.iter().into_group_map_by(|machine| machine.category.clone());

    html! {
        (DOCTYPE)
        form action="/logout" {
            button type="submit" { "logout" }
        }
        hr;
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
                    @for machine in group.into_iter().sorted_by_key(|machine| &machine.name) {
                        tr {
                            td { (category) }
                            td { (machine.name) }
                            td { (machine.id) }
                            td { (machine.urn) }
                            td { (machine.description) }
                            td { (format!("{:?}", machine.usage)) }
                            td {
                                form action="/toggle_machine" {
                                    button type="submit" { (machine.usage.button_text()) }
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

pub fn overview(machines: &[Machine]) -> Response {
    let group_map = machines.iter().into_group_map_by(|machine| machine.category.clone());
    
    html! {
        (DOCTYPE)
        form action="/logout" {
            button type="submit" { "logout" }
        }
        hr;
        @for (category, group) in group_map.into_iter().sorted_by_key(|(category, _grp)| category.to_owned()) {
            details open {
                br;
                summary { (category) }

                @for machine in group {
                    form action=(format!("/toggle_machine/{}", machine.urn)) {
                        fieldset {
                            legend { (machine.name) }

                            input type="text"
                                    value=(format!("status: {:?}", machine.usage))
                                    disabled;
                            button type="submit" { (machine.usage.button_text()) }
                            br;
                            small { (machine.description) }
                        }
                    }
                }
            }
            br;hr;br;
        }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}

pub fn error(error: &anyhow::Error) -> Response {
    html! {
        (DOCTYPE)
        body bgcolor="red" {
            (format!("{error:#?}"))
            form action="/" {
                button type="submit" { "go back" }
            }
        }
    }
    .into_string()
    .pipe(|html| Response::new(html.into()))
}