use serde::Serialize;
use tap::prelude::Pipe;
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use tap::prelude::Conv;

use crate::newtypes::UserName;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Credentials {
	pub username: UserName,
	pub password: String
}

pub fn decode_header(header: &str) -> anyhow::Result<Credentials> {
	log::trace!(header:%; "decode_header");
	
	BASE64_STANDARD
	.decode(header.trim_start_matches("Basic "))?
	.pipe(String::from_utf8)?
	.split_once(':')
	.ok_or_else(|| anyhow!("couldn't split decoded credentials"))?
	.conv::<[_; 2]>()
	.map(str::to_owned)
	.pipe(|[username, password]| Credentials { username: UserName(username), password })
	.pipe(Ok)
}