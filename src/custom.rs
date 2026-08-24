use std::fs::File;
use std::io::Write;
use std::ops::Div;
use std::time::Duration;

use std::{fs, io};

use chrono::prelude::Local;
use itertools::Itertools;

use crate::cfg;
use crate::newtypes::{LeaderId, UserName};

use self::data_machines::BillingScope;
use self::verein_online::model::{Bill, UserId};

pub mod data_machines;
pub mod verein_online;

#[derive(Debug, Clone, Default)]
pub struct Accountant;
impl Accountant {
	pub async fn write_bill(&self, user_name: &UserName, user_id: UserId, leader_id: &LeaderId, booked_time: Duration, runtime: Duration, vo_client: &verein_online::ApiClient) -> anyhow::Result<()> {
		log::trace!(user_name:?, leader_id:?, booked_time:?, runtime:?; "write bill");
		
		let data_machines_row = &data_machines::load().await?[leader_id];
		
		let machine_is_free_for_all = data_machines_row.artikel_id.0 == 0;
		let user_has_free_use_permission =
			cfg::INSTANCE
			.read()
			.await
			.permissions_of(user_name)
			.filter(|(_id, free_use)| *free_use)
			.map(|(id, _)| id)
			.contains(leader_id);
		
		if machine_is_free_for_all || user_has_free_use_permission {
			return Ok(());
		}
	
		let anzahl =
			match data_machines_row.billing_scope {
				BillingScope::BookedTime => booked_time,
				BillingScope::Runtime    => runtime
			}
			.as_secs_f32()
			.div(60.0)
			.div(data_machines_row.minutes_per_unit as f32)
			.ceil()
			as i32;
		
		let now = Local::now();
		
		let bill =
			Bill {
				user_id,// id nachschlagen in DataUser.csv. Wenn Spalte 3 ("toBeUsed") == 0 dann skip. Wenn nicht vorhanden dann fallback zum Namen
				quelle: "allgemeiner Beleg",
				brutto_netto: 2,
				artikel_id: data_machines_row.artikel_id,// DataMachine.csv#2
				positionsdetails: now.format("%Y-%m-%d").to_string(),
				anzahl,// minutes divided by DataMachine.csv#5 (ceil)
				rechnungstyp: 0
			};
			
		csv::WriterBuilder::new()
		.has_headers(false)
		.from_writer(open_file(now)?)
		.serialize(&bill)?;

		Ok(())
	}
}

fn open_file(now: chrono::DateTime<Local>) -> io::Result<File> {
	let mut path = crate::dir().join("bills");
	if !path.exists() {
		fs::create_dir(&path)?;
	}
	
	let file_name = format!("bills_{}.csv", now.format("%Y-%m"));
	path.push(file_name);
	
	let new = !path.exists();
	
	let mut file =
		File
		::options()
		.create(true)
		.append(true)
		.open(path)?;
	
	if new {
		writeln!(&mut file, "UserID,Quelle,BruttoNetto,artikelid,Positionsdetails,Anzahl,rechnungstyp")?;
	}
	
	Ok(file)
}