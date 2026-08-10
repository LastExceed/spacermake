use std::collections::HashMap;
use std::str::FromStr;

use anyhow::anyhow;
use itertools::Itertools;
use tap::prelude::Pipe;
use thiserror::Error;
use tokio::fs;

use crate::cfg;
use crate::newtypes::LeaderId;

use super::verein_online::model::ArtikelId;

pub async fn load() -> anyhow::Result<HashMap<LeaderId, Row>> {
	cfg::dir()
	.join("DataMachines.csv")
	.pipe(fs::read_to_string)
	.await?
	.lines()
	.map(Row::parse)
	.collect()
}

pub struct Row {
	pub artikel_id: ArtikelId,
	pub _ignore: i32,
	pub billing_scope: BillingScope,
	pub minutes_per_unit: i32
}

impl Row {
	fn parse(line: &str) -> anyhow::Result<(LeaderId, Self)> {
		let splits =
			line
			.split(',')
			.collect_array::<5>()
			.ok_or_else(|| anyhow!("incorrect number of columns"))?;
		
		let leader_id =
			splits[0]
			.to_owned()
			.pipe(LeaderId);

		let row =
			Self {
				artikel_id      : splits[1].parse::<i32>()?.pipe(ArtikelId),
				_ignore         : splits[2].parse()?,
				billing_scope   : splits[3].parse()?,
				minutes_per_unit: splits[4].parse()?
			};
		
		Ok((leader_id, row))
	}
}

#[repr(u8)]
pub enum BillingScope {
	BookedTime = 0,
	Runtime = 1
}

impl FromStr for BillingScope {
	type Err = ParseBillingScopeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"0" => Ok(Self::BookedTime),
			"1" => Ok(Self::Runtime),
			_   => Err(ParseBillingScopeError)
		}
	}
}

#[derive(Debug, Clone, Copy, Error)]
#[error("value was neither 0 nor 1")]
pub struct ParseBillingScopeError;