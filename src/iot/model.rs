use serde::*;
use strum::EnumIs;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all="UPPERCASE")]
pub struct Payload {
	pub margins: Margins
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct Margins {
	pub power_high: PowerHigh
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, EnumIs)]
#[serde(rename_all="UPPERCASE")]
pub enum PowerHigh {
	On,
	Off
}