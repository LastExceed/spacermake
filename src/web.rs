#![allow(clippy::absolute_paths, reason = "warp")]

use std::convert::Infallible;
use std::net::Ipv4Addr;
use std::ptr;
use anyhow::bail;
use itertools::Itertools;
use tap::prelude::Pipe;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use warp::http::header::*;
use warp::Filter;
use warp::filters::path::FullPath;
use warp::reply::*;

use crate::{App, cfg};
use crate::newtypes::{LeaderId, RoleId, UserName};

pub mod auth;
mod page;
mod extensions;

use extensions::*;

use self::auth::Credentials;

pub async fn host(app: &RwLock<App>) -> ! {
	log::trace!(app:?; "host web");

	// The following is a workaround for warp requiring all captures to be static
	// This requirement likely boils down to warp's reliance on `tokio::spawn()`,
	// which famously lacks a scoped counterpart (due to it being notoriously difficult to implement)

	// SAFETY:
	// This function returning the never-type guarantees that
	// * the `app` parameter cannot be outlived
	// * the following value will never get dropped
	let server =
		Server {
			app: unsafe { extend_lifetime(app) },
		};

	// SAFETY:
	// See above.
	let warp =
		unsafe { extend_lifetime(&server) }
		.pipe(routes)
		.pipe(warp::serve);
	
	log::info!(app:?; "begin hosting");

	warp.run((Ipv4Addr::UNSPECIFIED, 80))
	.await;

	// SAFETY:
	// `warp::server::run::Standard::run()` will never return,
	// because it contains a loop without exit condition.
	unsafe { std::hint::unreachable_unchecked() }
}

fn routes(server_static: &'static Server) -> impl Clone + Filter<Extract=impl Reply, Error=Infallible> {
	let favicon =
		warp::path("favicon.ico")
		.map(|| StatusCode::NO_CONTENT);

	let static_files =
		warp::fs::dir("www");

	let logged_in =
		warp::path::full()
		.and(warp::header(AUTHORIZATION.as_str()))
		.then(move |path, auth| server_static.on_request(path, auth));

	let fallback =
		warp::any()
		.map(|| reply().with_auth());

	favicon
	.or(static_files)
	.or(logged_in)
	.or(fallback)
}

struct Server {
	app: &'static RwLock<App>,
}

impl Server {
	async fn on_request(&self, path: FullPath, auth: String) -> Response {
		log::trace!(path:?, auth:%; "handle web request");

		self
		.try_handle(path, auth)
		.await
		.unwrap_or_else(|err| page::error::render(&err))
	}

	async fn try_handle(&self, path: FullPath, auth: String) -> anyhow::Result<Response> {
		let credentials = auth.pipe_as_ref(auth::decode_header)?;

		let app_guard = self.app.read().await;
		let _full_name = app_guard.vo_client.verify_login(&credentials).await?;

		let mut cfg_guard = cfg::INSTANCE.write().await;
		add_to_cfg_if_new(&mut cfg_guard, &credentials.username);

		let mut segments = split_segments(&path);

		let Some(target_leader) = segments.next()
		else { return page::overview::render(&cfg_guard, &app_guard, &credentials.username).pipe(Ok) };
		
		if target_leader == "toggle_role" {
			return toggle_role(&mut cfg_guard, segments, &credentials);
		}
		
		let target_leader = target_leader.to_owned().pipe(LeaderId);

		let Some(leader_cfg) = cfg_guard.leaders.get(&target_leader)
		else { bail!("unknown target") };

		let has_permission = cfg_guard.permissions_of(&credentials.username).map(|(id, _)| id).contains(&target_leader);

		let Some(command) = segments.next()
		else { return page::resource::render(&app_guard, &target_leader, &credentials.username, leader_cfg, has_permission).pipe(Ok) };

		if command != "toggle" {
			bail!("unknown command");
		}

		if !has_permission {
			bail!("no permission");
		}

		drop(app_guard); // meh.
		self.app.write().await.on_booking_request(&target_leader, &credentials.username).await?;

		reply()
		.with_redirect(&format!("/{}", target_leader.0))
		.into_response()
		.pipe(Ok)
	}	
}

fn toggle_role<'seg>(cfg: &mut cfg::Main, mut segments: impl Iterator<Item=&'seg str>, credentials: &Credentials) -> anyhow::Result<Response> {
	let Some(input_user) = segments.next()
	else { bail!("no user specified"); };
	
	let Some(input_role) = segments.next()
	else { bail!("no role specified"); };
	
	let has_permission =
		cfg
		.users
		[&credentials.username] // infallible because all users are inserted on first login
		.roles
		.iter()
		.any(|role_id|
			cfg.roles[role_id].can_assign_roles
		);

	if !has_permission {
		bail!("no permission");
	}
	
	let username = UserName(input_user.to_owned());
	let Some(cfg_user_target) = cfg.users.get_mut(&username)
	else { bail!("unknown user"); };
		
	let role_id = RoleId(input_role.to_owned());
	if !cfg.roles.contains_key(&role_id) {
		bail!("unknown role");
	}
	
	let message =
		if let Some(pos) = cfg_user_target.roles.iter().position(|existing| *existing == role_id) {
			cfg_user_target.roles.swap_remove(pos);
			"role added"
		} else {
			cfg_user_target.roles.push(role_id);
			"role removed"
		};
		
	message.into_response().pipe(Ok)
}

fn add_to_cfg_if_new(cfg: &mut cfg::Main, username: &UserName) {
	_ =
		cfg
		.users
		.entry(username.clone())
		.or_default();
}

// probably shouldn't do this manually
fn split_segments(full_path: &FullPath) -> impl Iterator<Item=&str> {
	full_path
	.as_str()
	.trim_start_matches('/')
	.trim_end_matches('?')
	.split('/')
	.filter(|split| !split.is_empty())
}

/// # SAFETY
/// The returned reference must not outlive the borrowed value.
const unsafe fn extend_lifetime<T>(source: &T) -> &'static T {
	unsafe { ptr::from_ref(source).as_ref_unchecked() }
}