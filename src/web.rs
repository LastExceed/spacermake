#![allow(clippy::absolute_paths, reason = "warp")]

use std::net::Ipv4Addr;
use std::sync::Arc;
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use http::StatusCode;
use http::header::*;
use tap::Pipe;
use tokio::sync::RwLock;
use warp::filters::path::FullPath;
use warp::reply::*;
use warp::*;

mod page;
mod model;

pub async fn start(state: Arc<RwLock<AppState>>) {
    path::full()
    .and(warp::header::optional(AUTHORIZATION.as_str()))
    .then(move |path, auth| on_request(path, auth, Arc::clone(&state)))
    .pipe(|main| warp::fs::dir("www").or(main))
    .pipe(warp::serve)
    .run((Ipv4Addr::UNSPECIFIED, 80))
    .await;
}

async fn on_request(path: FullPath, auth: Option<String>, state: Arc<RwLock<AppState>>) -> warp::reply::Response {
    let state_guard = state.read().await;
    
	try_handle(path, auth, &*state_guard)
    .await
    .unwrap_or_else(|err| page::error(&err))
}

async fn try_handle(path: FullPath, auth: Option<String>, state: &AppState) -> anyhow::Result<warp::reply::Response> {
    let path =
        path
        .as_str()
        .trim_start_matches('/')
        .trim_end_matches('?');
    
    if path == "favicon.ico" {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    let Some(auth) = auth
    else {
        return Ok(StatusCode::UNAUTHORIZED.with_auth().into_response());
    };

	let [username, password] =
        auth
        .trim_start_matches("Basic ")
        .pipe(decode_auth)?;
    
    todo!("check creds");

    let mut splits = path.split('/').filter(|split| !split.is_empty());

    let target = splits.next();
    let toggle = splits.next() == Some("toggle");

    let resources: [Machine; 0] = todo!();

    let Some(target_urn) = target
    else {
        return page::overview(&resources, config.hide_unbooked)
        .pipe_ref(page::template)
        .pipe(Ok);
    };
    
    if target_urn.eq_ignore_ascii_case("debug") {
        return Ok(page::debug(&resources));
    }

    let target_resource =
        resources
        .iter()
        .find(|resource| resource.urn == target_urn)
        .ok_or_else(|| anyhow!("unknown resource"))?;

    if toggle {
        redirect(&format!("/{}", target.unwrap()))
    } else {
        page::resource(target_resource)
        .pipe_ref(page::template)
    }.pipe(Ok)
}

fn decode_auth(base64: &str) -> anyhow::Result<[String; 2]> {
    BASE64_STANDARD
    .decode(base64)?
    .pipe(String::from_utf8)?
    .split_once(':').ok_or_else(||anyhow!("couldn't split decoded credentials"))?
    .pipe(|(name, pw)| [name.to_owned(), pw.to_owned()])
    .pipe(Ok)
}

fn redirect(location: &str) -> warp::reply::Response {
    StatusCode::SEE_OTHER
    .with_header("Location", location)
    .into_response()
}

#[extend::ext]
impl<T: Reply> T {
    fn with_header(self, name: &str, value: &str) -> WithHeader<T> {
        with_header(self, name, value)
    }

    fn with_status(self, status: StatusCode) -> WithStatus<T> {
        with_status(self, status)
    }

    fn with_auth(self) -> WithHeader<WithStatus<T>> {
	    self
	    .with_status(StatusCode::UNAUTHORIZED)
		.with_header(WWW_AUTHENTICATE.as_str(), "Basic")
    }
}
