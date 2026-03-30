#![allow(clippy::absolute_paths, reason = "warp")]

use std::collections::HashMap;
use std::net::Ipv4Addr;
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use http::StatusCode;
use http::header::*;
use itertools::Itertools;
use warp::filters::path::FullPath;
use warp::reply::*;
use warp::*;

use crate::cfg::Leader;
use crate::{LeaderId, services::*};

mod page;

pub struct Service {
    booking_service: Arc<booking::Service>,
    metadatas: HashMap<LeaderId, Metadata>
}

impl Service {
    pub(super) fn new(config: &cfg::Main, booking_service: Arc<booking::Service>) -> Self {
        Self {
            booking_service,
            metadatas: Metadata::load_all(config)
        }
    }
    
    pub async fn run(self: &Arc<Self>) {
        let self2 = Arc::clone(self);

        path::full()
        .and(warp::header::optional(AUTHORIZATION.as_str()))
        .then(move |path, auth| Arc::clone(&self2).on_request(path, auth))
        .pipe(|main| warp::fs::dir("www").or(main))
        .pipe(warp::serve)
        .run((Ipv4Addr::UNSPECIFIED, 80))
        .await;
    }
    
    async fn on_request(self: Arc<Self>, path: FullPath, auth: Option<String>) -> warp::reply::Response { 
        self.try_handle(path, auth)
        .await
        .unwrap_or_else(|err| page::error(&err))
    }

    async fn try_handle(&self, path: FullPath, auth: Option<String>) -> anyhow::Result<warp::reply::Response> {
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
            return reply().with_auth().into_response().pipe(Ok);
        };

        let [username, password] =
            auth
            .trim_start_matches("Basic ")
            .pipe(decode_auth)?;
        
        todo!("check {username} {password}");

        let mut splits = path.split('/').filter(|split| !split.is_empty());

        let Some(target) = splits.next()
        else {
            return
                page
                ::overview(&self.booking_service, &self.get_category_map(), &username)
                .await
                .pipe(Ok);
        };
        let target = LeaderId(target.to_owned());
        
        let Some(metadata) = self.metadatas.get(&target)
        else {
            return Err(anyhow!("unknown resource"));
        };
        
        let Some(command) = splits.next()
        else {
            return Ok(page::resource(&target, &username, &self.booking_service).await);
        };
        
        match command {
            "book" =>
                self
                .booking_service
                .try_update(&target, &username, true)
                .await
                .map(|()| redirect(&format!("/{}", target.0))),
            
            "release" =>
                self
                .booking_service
                .try_update(&target, &username, false)
                .await
                .map(|()| redirect(&format!("/{}", target.0))),
            
            _ => Err(anyhow!("unknown command"))
        }
    }
    
    fn get_category_map(&self) -> HashMap<&String, Vec<&LeaderId>> {
        // might be worth caching this
        self
        .metadatas
        .iter()
        .map(|(leader_id, metadata)| (&metadata.category, leader_id))
        .into_group_map()
    }
}

fn redirect(location: &str) -> warp::reply::Response {
    reply()
    .with_redirect(location)
    .into_response()
}

fn decode_auth(base64: &str) -> anyhow::Result<[String; 2]> {
    BASE64_STANDARD
    .decode(base64)?
    .pipe(String::from_utf8)?
    .split_once(':').ok_or_else(||anyhow!("couldn't split decoded credentials"))?
    .pipe(|(name, pw)| [name.to_owned(), pw.to_owned()])
    .pipe(Ok)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Metadata {
    category: String,
    description: String
}

impl Metadata {
    fn load_all(config: &cfg::Main) -> HashMap<LeaderId, Self> {
        config
        .leaders
        .iter()
        .map(|(id, properties)| (
            id.clone(),
            Self::of(properties)
        ))
        .collect()
    }
    
    fn of(properties: &Leader) -> Self {
        Self {
            category: properties.category.clone(),
            description: properties.description.clone()
        }
    }
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
        .with_status(StatusCode::SEE_OTHER)
    }

    fn with_auth(self) -> WithHeader<WithStatus<T>> {
	    self
	    .with_status(StatusCode::UNAUTHORIZED)
		.with_header(WWW_AUTHENTICATE.as_str(), "Basic")
    }
}
