use std::net::Ipv4Addr;
use std::sync::Arc;
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use colour::*;
use http::*;
use http::header::*;
use tap::Pipe;
use tokio::task;
use warp::filters::path::FullPath;
use warp::reply::*;
use warp::*;
use fab_api::FrontDesk;

use crate::my_config::MyConfig;

mod fab_api;
mod page;

pub async fn start(config: Arc<MyConfig>) {
    let local_set = task::LocalSet::new();

    let front_desk = fab_api::start_local(&local_set, config).await;
    let web_server = server(front_desk);

    local_set.run_until(web_server).await;
}

pub async fn server(front_desk: FrontDesk) {
    let front_desk = Arc::new(front_desk);
    
    path::full()
    .and(warp::header::optional(AUTHORIZATION.as_str()))
    .then(move |path, auth| on_request(path, auth, Arc::clone(&front_desk)))
    .pipe(|main| warp::fs::dir("www").or(main))
    .pipe(warp::serve)
    .run((Ipv4Addr::UNSPECIFIED, 80))
    .await;
}

async fn on_request(path: FullPath, auth: Option<String>, front_desk: Arc<FrontDesk>) -> warp::reply::Response {
	try_handle(path, auth, &front_desk)
    .await
    .unwrap_or_else(|err| {
        if format!("{err:?}") == "(code = invalidCredentials)" {
            reply().with_auth().into_response()
        } else {
            red_ln!("{err:#?}");
            page::error(err)
        }
    })
}

async fn try_handle(path: FullPath, auth: Option<String>, front_desk: &FrontDesk) -> anyhow::Result<warp::reply::Response> {
    yellow_ln!("{}", path.as_str());
    let path =
        path
        .as_str()
        .trim_start_matches('/')
        .trim_end_matches('?');
    
    let Some(auth) = auth
    else {
        return Ok(reply().with_auth().into_response());
    };

	let [username, password] =
        auth
        .trim_start_matches("Basic ")
        .pipe(decode_auth)?;

    let mut splits = path.split('/').filter(|split| !split.is_empty());

    let target = splits.next();
    let toggle = splits.next() == Some("toggle");

    let resources =
    	front_desk
    	.exchange(
     		username,
       		password,
        	target.filter(|_| toggle).map(str::to_owned)
     	)
     	.await?;

    let Some(target_urn) = target
    else {
        return Ok(page::overview(&resources));
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
    reply().with_redirect(location).into_response()
}

#[extend::ext]
impl<T: Reply> T {
    fn with_header(self, name: &str, value: &str) -> WithHeader<T> {
        with_header(self, name, value)
    }

    fn with_status(self, status: StatusCode) -> WithStatus<T> {
        with_status(self, status)
    }

    fn with_set_cookie(self, name: &str, value: &str) -> WithHeader<T> {
        self
        .with_header("Set-Cookie", &format!("{name}={value}"))
    }

    fn with_redirect(self, location: &str) -> WithStatus<WithHeader<T>> {
        self
        .with_header("Location", location)
        .with_status(http::StatusCode::SEE_OTHER)
    }

    fn with_auth(self) -> WithHeader<WithStatus<T>> {
	    self
	    .with_status(StatusCode::UNAUTHORIZED)
		.with_header(WWW_AUTHENTICATE.as_str(), "Basic")
    }
}
