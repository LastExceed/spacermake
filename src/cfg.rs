use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

use anyhow::bail;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tap::prelude::Pipe;
use tokio::{fs, io};
use tokio::sync::RwLock;

use crate::newtypes::*;
use crate::custom::verein_online;

pub mod template;

pub static INSTANCE: LazyLock<RwLock<Main>> = LazyLock::new(RwLock::default);

pub async fn init() -> anyhow::Result<bool> {
	log::trace!("init cfg");
	
	let path_dir = dir();
	let path_main = path_dir.join("main.toml");
	let path_vo = path_dir.join("verein_online.toml");
	
	log::trace!("check if cfg dir exists");
	let dir_exists = fs::try_exists(&path_dir).await?;
	if !dir_exists {
		log::trace!("create cfg dir");
		fs::create_dir(&path_dir).await?;
	}
	let existed_main = ensure_exists::<Main>(&path_main).await?;
	let existed_vo = ensure_exists::<verein_online::Config>(&path_vo).await?;

	if !dir_exists || !existed_main || !existed_vo {
		return Ok(false);
	}
	
	log::trace!("load main cfg");
	let config = Main::read_from_file(&path_main).await?;
	config.validate()?;
	*INSTANCE.write().await = config;
	Ok(true)
}

pub fn dir() -> PathBuf {
	crate::dir()
	.join("configs")
}

pub async fn create_template<T: Default + Serialize>(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
	let contents = toml::to_string(&T::default())?;
	
	fs::write(file_path, contents).await?;
	
	Ok(())
}

pub async fn ensure_exists<T: Default + Serialize>(file_path: impl AsRef<Path>) -> anyhow::Result<bool> {
	log::trace!("ensure_exists");
	
	let exists = fs::try_exists(file_path.as_ref()).await?;
	if !exists {
		create_template::<T>(file_path).await?;
	}
	
	Ok(exists)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MqttBroker {
	pub host: String,
	pub username: Option<String>,
	pub password: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Supporter {
	pub topic_start: String,
	pub topic_stop: String,
	pub payload_start: String,
	pub payload_stop : String,
	pub trailing_seconds: u64
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Leader {
	pub description          : String,
	pub display_category     : String,
	pub dependencies_booktime: Vec<SupporterId>,
	pub dependencies_runtime : Vec<SupporterId>,
	pub runtime_display_id   : Option<RuntimeDisplayId>
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Role {
	pub free_use: bool,
	pub can_assign_roles: bool,
	pub leaders: Vec<LeaderId>
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
	pub roles: Vec<RoleId>
}

type Supporters = HashMap<SupporterId, Supporter>;
type Leaders    = HashMap<   LeaderId, Leader   >;
type Roles      = HashMap<     RoleId, Role     >;
type Users      = HashMap<   UserName, User     >;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Main {
	pub mqtt_broker: MqttBroker,
	pub supporters : Supporters,
	pub leaders    : Leaders,
	pub roles      : Roles,
	pub users      : Users
}

impl Main {
	async fn read_from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		fs::read_to_string(path)
		.await?
		.pipe_as_ref(toml::from_str::<Self>)?
		.pipe(Ok)
	}
	
	pub fn validate(&self) -> anyhow::Result<()> {
		log::trace!(self:?; "validate config");
		
		log::trace!("validate supporter configs");
		let all_dependencies =
			self
			.leaders
			.values()
			.flat_map(|leader_cfg| &leader_cfg.dependencies_booktime)
			.chain(
				self.leaders.values().flat_map(|leader_cfg| &leader_cfg.dependencies_runtime)
			)
			.collect_vec();

		for id in self.supporters.keys().filter(|id| !all_dependencies.contains(&id)) {
			log::warn!("supporter `{}` is not used", id.0);
		}

		log::trace!("validate leader configs");
		for (id, cfg) in &self.leaders {
			for dep in cfg.dependencies_booktime.iter().chain(cfg.dependencies_runtime.iter()) {
				if !self.supporters.contains_key(dep) {
					bail!(
						"configuration missing for supporter `{}` (a dependency of leader `{}`)",
						dep.0,
						id.0
					);
				}
			}
		}

		log::trace!("validate role configs");
		for (role_id, role) in &self.roles {
			for leader_id in &role.leaders {
				if !self.leaders.contains_key(leader_id) {
					bail!(
						"configuration missing for leader `{}` (assigned to role `{}`)",
						leader_id.0,
						role_id.0
					);
				}
			}
		}

		log::trace!("validate user configs");
		for (user_id, user) in &self.users {
			for role_id in &user.roles {
				if !self.roles.contains_key(role_id) {
					bail!(
						"configuration missing for role `{}` (assigned to user `{}`)",
						role_id.0,
						user_id.0
					);
				}
			}
		}

		log::trace!("config validated");
		
		Ok(())
	}

	pub fn permissions_of(&self, user_name: &UserName) -> impl Iterator<Item=(&LeaderId, bool)> {
		self
		.users
		.get(user_name)
		.map(|user| user.roles.as_slice())
		.unwrap_or_default()
		.iter()
		.flat_map(|role_id| {
			let role = &self.roles[role_id];
			role.leaders.iter().map(|leader_id| (leader_id, role.free_use))
		})
		.unique()
	}
}

impl Default for Main {
	fn default() -> Self {
		template::build()
	}
}

pub struct Watcher(SystemTime);

impl Watcher {
	pub async fn new() -> Self {
		dir()
		.join("main.toml")
    	.pipe_ref(get_time_modified)
    	.await
		.expect("get time modified")
		.pipe(Self)
	}
	
	pub async fn refresh(&mut self) {
		let path = dir().join("main.toml");
		let modified_at =
			match get_time_modified(&path).await {
				Ok(time) => time,
				Err(error) => {
					log::error!(error:?; "could not read config file metadata");
					return;
				}
			};

		if modified_at > self.0 {
			self.0 = modified_at;

			let new =
				match Main::read_from_file(&path).await {
					Ok(value) => value,
					Err(error) => {
						log::error!(error:?; "could not reload config file");
						return;
					}
				};
			
			if let Err(error) = new.validate() {
				log::error!(error:?; "invalid config");
			}
			
			*INSTANCE.write().await = new;
			self.0 = modified_at;
		}
	}
}

async fn get_time_modified(path: &PathBuf) -> io::Result<SystemTime> {
	fs::metadata(&path).await?.modified()
} 