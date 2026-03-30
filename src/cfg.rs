use std::collections::HashMap;
use std::fs;

use tap::prelude::Pipe;

use crate::*;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Main {
	pub mqtt_broker: MqttBroker,
	pub leaders: HashMap<LeaderId, Leader>,
	pub supporters: HashMap<SupporterId, Supporter>,
}

impl Main {
	pub fn load() -> Self {
		let mut path_buf =
			dirs::config_dir()
			.expect("system config dir is not defined")
			.join("spacermake");
		
		if !path_buf.exists() {
			fs::create_dir(&path_buf)
			.expect("could not create config dir");
		}

		path_buf.push("config");
		path_buf.set_extension("toml");

		let exists =
			fs::exists(&path_buf)
			.expect("could not check for pre-existing file");
		
		if exists {
			fs::read_to_string(&path_buf)
			.expect("could not read pre-existing file")
			.pipe_as_ref(toml::from_str)
			.expect("could not parse config file")
		} else {
			let template = Self::template();
			
			toml::to_string_pretty(&template)
			.expect("could not serialize config template")
			.pipe(|serialized| fs::write(&path_buf, serialized))
			.expect("could not write template to file");

			template
		}
	}
	
	pub fn validate(&self) {
		let supporters: Vec<_> = self.supporters.keys().collect();
		
		for (leader_id, leader) in &self.leaders {
			for dep in leader.dependencies_fulltime.iter().chain(leader.dependencies_runtime.iter()) {
				assert!(supporters.contains(&dep), "configuration missing for supporter `{}` (a dependency of leader `{}`)", dep.0, leader_id.0);
			}
		}
		
		for (supporter_id, supporter) in &self.supporters {
			assert!(supporter.trailing_time.is_finite()       , "supporter `{}` has invalid trailing_time: {}", supporter_id.0, supporter.trailing_time);
			assert!(supporter.trailing_time.is_sign_positive(), "supporter `{}` has illegal trailing_time: {}", supporter_id.0, supporter.trailing_time);
		}
	}
	
	fn template() -> Self {
		Self {
			mqtt_broker: MqttBroker {
				host: "mqtt.example.com".to_owned(),
				username: Some("JohnDoe42".to_owned()),
				password: Some("SuperSecret123!".to_owned())
			},
			leaders: [
				(
					LeaderId("ThingDoer3000".to_owned()),
					Leader {
						category: "ExcitingMachines".to_owned(),
						description: "Does things, I think".to_owned(),
						dependencies_fulltime: vec![SupporterId("SuePorter".to_owned()), SupporterId("SoupMortar".to_owned())],
						dependencies_runtime: vec![SupporterId("SubParTar".to_owned())],
					}
				),
				(
					LeaderId("StuffMaker9001".to_owned()),
					Leader {
						category: "BoringMachines".to_owned(),
						description: "Makes stuff, I'm told".to_owned(),
						dependencies_fulltime: vec![],
						dependencies_runtime: vec![SupporterId("SoupMortar".to_owned())],
					}
				)
			].into(),
			supporters: [
				(
					SupporterId("SuePorter".to_owned()),
					Supporter {
						trailing_time: 0.0,
						mqtt_topic: String::new(),
						mqtt_payload_startup: String::new(),
						mqtt_payload_shutdown: String::new(),
    				}
				),
				(
					SupporterId("SoupMortar".to_owned()),
					Supporter {
						trailing_time: 15.0,
						mqtt_topic: String::new(),
						mqtt_payload_startup: String::new(),
						mqtt_payload_shutdown: String::new(),
    				}
				),
				(
					SupporterId("SubParTar".to_owned()),
					Supporter {
						trailing_time: 0.0,
						mqtt_topic: String::new(),
						mqtt_payload_startup: String::new(),
						mqtt_payload_shutdown: String::new(),
    				}
				),
			].into()
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MqttBroker {
	pub host: String,
	pub username: Option<String>,
	pub password: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Leader {
	pub category: String,
	pub description: String,
	pub dependencies_fulltime: Vec<SupporterId>,
	pub dependencies_runtime: Vec<SupporterId>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Supporter {
	pub trailing_time: f64,
	pub mqtt_topic: String,
	pub mqtt_payload_startup: String,
	pub mqtt_payload_shutdown: String
}

pub struct VereinOnline {
	pub verein: String,
	pub admin_username: String,
	pub admin_password: String
}