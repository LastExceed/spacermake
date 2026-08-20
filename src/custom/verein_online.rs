use std::fs;

use serde::de::DeserializeOwned;
use serde::*;
use tap::prelude::*;

use crate::cfg;
use crate::web::auth::VerifyLogin;

pub mod model;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
	pub verein: String,
	pub admin_username: String,
	pub admin_password: String
}

impl Config {	
	fn read_from_file() -> anyhow::Result<Self> {
		cfg::dir()
		.join("verein_online.toml")
		.pipe(fs::read_to_string)?
		.pipe_as_ref(toml::from_str::<Self>)?
		.pipe(Ok)
	}
}

#[derive(Debug)]
pub struct ApiClient {
	config: Config,
	http_client: reqwest::Client
}

impl ApiClient {
	pub fn new() -> Self {
		Self {
			config: Config::read_from_file().unwrap(),
			http_client: reqwest::Client::new()
		}
	}

	async fn query<Response: DeserializeOwned>(&self, api: &str, payload: &impl Serialize) -> reqwest::Result<Response> {
		let Config { // just for readability
			verein,
			admin_username,
			admin_password
		} = &self.config;

		let pw_hash = md5::compute(admin_password);

		self
		.http_client
		.post(format!("https://www.vereinonline.org/{verein}/?api={api}"))
		.header("Authorization", format!("A/{admin_username}/{pw_hash:x}"))
		.json(payload)
		.send()
		.await?
		.json::<Response>()
		.await
	}

	pub async fn verify_login(&self, user: &str, password: &str) -> reqwest::Result<Vec<String>> {
		log::trace!(user:%, password:%; "verify login");
		
		let payload = VerifyLogin {
			user,
			password,
			result: "id"
		};
		
		self
		.query("VerifyLogin", &payload)
		.await
	}

	pub async fn get_members(&self) -> reqwest::Result<Vec<Member>> {
		log::trace!("get VO members");
		
		self
    	.query("GetMembers", &()) // the unit payload here is actually required by VereinOnline
    	.await
	}
}