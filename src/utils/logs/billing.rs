use std::{io, ops::Div};
use std::fs::File;

use chrono::Local;
use colour::red_ln;
use csv::WriterBuilder;
use serde::Serialize;

use crate::utils::booking::Booking;
use crate::my_config::{MachineData, MyConfig};

#[derive(Debug, Serialize)]
struct BillingRecord {
    user_id: String,                      // id nachschlagen in DataUser.csv. Wenn Spalte 3 ("toBeUsed") == 0 dann skip. Wenn nicht vorhanden dann fallback zum Namen
    quelle: &'static str,                 // "allgemeiner Beleg"
    brutto_netto: i32,                    // 2
    artikel_id: String,                   // DataMachine.csv#2
    positionsdetails: String,             // Date
    anzahl: i32,                          // minutes divided by DataMachine.csv#5 (ceil)
    rechnungstyp: i32                     // 0
}

pub fn billinglog(machine: &str, booking: &Booking, config: &MyConfig) -> io::Result<()> {
    let user_id =
        if let Some(user_data) = &config.data_user.get(&booking.user) {
            if !user_data.to_be_used { return Ok(()); }
            user_data
                .id
                .map_or_else(|| booking.user.clone(), |i|i.to_string())
        } else {
            booking.user.clone()
        };
        
    let machine_data = &config
        .data_machines
        .get(machine)
        .unwrap_or(&MachineData {
            id: None,
            to_be_used: true,
            power_sense: true,
            divider: 1
        });
        
    if !machine_data.to_be_used { return Ok(()); }
    
    let artikel_id = machine_data
        .id
        .map_or_else(|| machine.to_owned(), |i| i.to_string());
    
    let anzahl =
        if machine_data.power_sense {
            booking.total_runtime()
        } else {
            booking.creation_instant.elapsed()
        }
        .as_secs_f32()
        .div(60.0)
        .div(machine_data.divider as f32)
        .ceil()
        as _;
    
    let bill = BillingRecord {
        user_id,
        quelle: "allgemeiner Beleg",
        brutto_netto: 2,
        artikel_id,
        positionsdetails: Local::now()
            .format("%Y-%m-%d")
            .to_string(),
        anzahl,
        rechnungstyp: 0,
    };
    
    let file_writer = File::options()
        .create(true)
        .append(true)
        .open(&config.billing_log)?;

    WriterBuilder::new()
        .has_headers(false)
        .from_writer(file_writer)
        .serialize(&bill)
        .map_err(|error| {
            red_ln!("error while serializing: {error}\n{bill:#?}");
            io::ErrorKind::Other.into()
        })
}