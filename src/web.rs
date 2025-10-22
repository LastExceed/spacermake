use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use tap::Pipe;
use tokio::task;
use warp::reply::*;
use warp::*;
use fab_api::FrontDesk;

use crate::my_config::MyConfig;

mod fab_api;
mod page;

type FormData = HashMap<String, String>;

const COOKIE_NAME: &str = "fab_credentials";

pub async fn start(config: Arc<MyConfig>) {
    let local_set = task::LocalSet::new();
    
    let front_desk = fab_api::start_local(&local_set, config).await;
    let web_server = server(front_desk);

    local_set.run_until(web_server).await;
}

pub async fn server(front_desk: FrontDesk) {
    let front_desk1 = Arc::new(front_desk);
    let front_desk2 = Arc::clone(&front_desk1);
    let front_desk3 = Arc::clone(&front_desk1);

    let login =
        path("login")
        .and(body::form())
        .then(move |form| on_login(form, Arc::clone(&front_desk1)));
    
    let logout =
        path("logout")
        .map(|| reply().with_set_cookie(COOKIE_NAME, "").with_redirect("/"));

    let toggle_machine = 
        path!("toggle_machine" / String)
        .and(cookie(COOKIE_NAME))
        .then(move |urn, cookie| on_toggle_machine(cookie, urn, Arc::clone(&front_desk2)));

    let overview =
        cookie::optional(COOKIE_NAME)
        .then(move |cookie| on_overview(cookie, Arc::clone(&front_desk3)));
        
    login
    .or(logout)
    .or(toggle_machine)
    .or(overview)
    .pipe(warp::serve)
    .run(([0, 0, 0, 0], 80))
    .await;
}

async fn on_overview(cookie: Option<String>, front_desk: Arc<FrontDesk>) -> reply::Response {
    let Some(Ok([name, pw])) = cookie.map(|json| serde_json::from_str::<[String; 2]>(&json))
    else { return page::login(); };

    match front_desk.exchange(name, pw, None).await {
        Ok(machines) => page::overview(&machines),
        Err(error)   => page::error(&error),
    }
}

async fn on_login(mut form: FormData, front_desk: Arc<FrontDesk>) -> reply::Response {
    let (Some(username), Some(password)) = (form.remove("username"), form.remove("password"))
    else { return page::error(&anyhow!("form data incomplete")); };
    
    match front_desk.exchange(username.clone(), password.clone(), None).await {
        Err(error) => page::error(&error),
        Ok(machines) =>
            page::overview(&machines)
            .with_set_cookie(COOKIE_NAME, &serde_json::to_string(&[username, password]).unwrap())
            .with_redirect("/")
            .into_response()
    }
}

async fn on_toggle_machine(cookie: String, urn: String, front_desk: Arc<FrontDesk>) -> reply::Response {
    let Ok([name, pw]) = serde_json::from_str::<[String; 2]>(&cookie)
    else { return page::login(); };
    
    match front_desk.exchange(name, pw, Some(urn)).await {
        Err(error) => page::error(&error),
        Ok(machines) => page::overview(&machines)
            .with_redirect("/")
            .into_response()
    }
}

#[extend::ext]
impl<T: Reply> T {
    fn with_set_cookie(self, name: &str, value: &str) -> WithHeader<T> {
        with_header(self, "Set-Cookie", &format!("{name}={value}"))
    }
    
    fn with_redirect(self, location: &str) -> WithStatus<WithHeader<T>> {
        with_header(self, "Location", location)
        .pipe(|x| with_status(x, http::StatusCode::SEE_OTHER))
    }
}