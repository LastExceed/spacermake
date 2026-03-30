use std::collections::HashMap;

use itertools::Itertools;

use crate::cfg::Leader;
use crate::*;

pub struct Service {
	machine_control: Arc<machine_control::Service>,
	leader_states  : HashMap<LeaderId, bool>,
	configurations : HashMap<LeaderId, DependencyConfig>
}

impl Service {
	pub(super) fn new(config: &cfg::Main, machine_control: Arc<machine_control::Service>) -> Self {
		Self {
			machine_control,
			leader_states: HashMap::default(),
			configurations: DependencyConfig::get_all(config)
		}
	}
	
	pub async fn update(
		&self,
		this_leader: &LeaderId,
		scope: Scope,
		new_state: bool
	) {
		let Some(deps_of_this) = self.configurations.get(this_leader)
		else { return };
		
		let deps_of_others = self.supporters_needed_by_any_other_than(this_leader).collect_vec();

		let deps_to_toggle =
			match scope {
				Scope::Booktime => &deps_of_this.booktime,
				Scope::Runtime  => &deps_of_this.runtime,
			}
			.iter()
			.filter(|dep| !deps_of_others.contains(dep));

		for supporter in deps_to_toggle {
			self.machine_control.update_machine(supporter, new_state).await;
		}
	}

	// todo
	// fn validate() {
		// Booktime+true -- none
		// booktime+false -- Some(false)
		// runtime+true -- Some(false)
		// runtime+false -- Some(true)
	// }

	// better name pending
	fn supporters_needed_by_any_other_than(&self, exception: &LeaderId) -> impl Iterator<Item=&SupporterId> {
		// imperative implementation because the functional approach turns out quite messy
		let mut out = vec![];
		
		for (leader, &is_running) in &self.leader_states {
			if leader == exception { continue }

			let Some(config) = self.configurations.get(leader)
			else { continue };
			
			out.extend(&config.booktime);
			
			if is_running {
				out.extend(&config.runtime);
			}
		}
		
		out.into_iter().unique()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Scope {
	Booktime,
	Runtime
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DependencyConfig {
    booktime: Vec<SupporterId>,
    runtime : Vec<SupporterId>
}

impl DependencyConfig {
	fn get_all(config: &cfg::Main) -> HashMap<LeaderId, Self> {
		config
		.leaders
		.iter()
		.map(|(id, properties)|
			(id.clone(), Self::of(properties))
		)
		.collect()
	}
	
	fn of(properties: &Leader) -> Self {
		Self {
			booktime: properties.dependencies_fulltime.clone(),
			runtime: properties.dependencies_runtime.clone()
		}
	}
}