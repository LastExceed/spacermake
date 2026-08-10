use super::*;

pub fn build() -> Main {
	let [supporter_id1, supporter_id2, supporter_id3] = supporter_ids();
	let [leader_id1, leader_id2] = leader_ids();
	let [role_id1, role_id2] = role_ids();
	let [user1, user2, user3, user4] = user_names();
	
	let supporter = Supporter {
		topic: "mqtt/topic/here".to_owned(),
		payload_start: "mqtt/payload/here".to_owned(),
		payload_stop: "mqtt/payload/here".to_owned(),
		trailing_seconds: 0,
	};
	
	let leader1 = Leader {
		description: "Does things, I think".to_owned(),
		display_category: "ExcitingMachines".to_owned(),
		dependencies_booktime: vec![supporter_id1.clone(), supporter_id2.clone()],
		dependencies_runtime : vec![supporter_id3.clone()]
	};
	
	let leader2 = Leader {
		description: "Makes stuff, I'm told".to_owned(),
		display_category: "BoringMachines".to_owned(),
		dependencies_booktime: vec![],
		dependencies_runtime : vec![supporter_id2.clone()]
	};
	
	
	
	Main {
		mqtt_broker: MqttBroker {
			host: "mqtt.example.com".to_owned(),
			username: Some("JohnDoe42".to_owned()),
			password: Some("SuperSecret123!".to_owned())
		},
		supporters: [
			(supporter_id1, supporter.clone()),
			(supporter_id2, supporter.clone()),
			(supporter_id3, supporter),
		].into(),
		leaders: [
			(leader_id1.clone(), leader1),
			(leader_id2.clone(), leader2)
		].into(),
		roles: [
			(role_id1.clone(), Role { free_use: true , can_assign_roles: true , leaders: vec![leader_id1, leader_id2.clone()] }),
			(role_id2.clone(), Role { free_use: false, can_assign_roles: false, leaders: vec![            leader_id2        ] }),
		].into(),
		users: [
			(user1, User { roles: vec![role_id1.clone()  ] }),
			(user2, User { roles: vec![role_id2.clone()  ] }),
			(user3, User { roles: vec![role_id1, role_id2] }),
			(user4, User { roles: vec![                  ] })
		].into(),
	}
}

fn leader_ids() -> [LeaderId; 2] {
	[
		"ThingDoer3000",
		"StuffMaker9001"
	]
	.map(str::to_owned)
	.map(LeaderId)
}

fn supporter_ids() -> [SupporterId; 3] {
	[
		"SuePorter",
		"SoupOtter",
		"SubparTar"
	]
	.map(str::to_owned)
	.map(SupporterId)
}

fn role_ids() -> [RoleId; 2] {
	[
		"Admin",
		"Pleb"
	]
	.map(str::to_owned)
	.map(RoleId)
}

fn user_names() -> [UserName; 4] {
	[
		"Alice",
		"Bob",
		"Charlie",
		"Diana"
	]
	.map(str::to_owned)
	.map(UserName)
}