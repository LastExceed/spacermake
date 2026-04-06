use std::{io::Write, ops::Div};
use std::io;
use std::fs::File;

use chrono::Local;
use colour::red_ln;
use csv::WriterBuilder;
use serde::Serialize;

use crate::settings::AppSettings;

#[derive(Debug, Serialize)]
struct Record<'string> {
    machine: &'string str,
    date: String,
    time_booked: String,
    time_released: String,
    booking_duration: i32, // minutes
    runtime: i32, // minutes
    user: &'string str
}

pub fn machinelog(machine: &str, booking: &Booking, config: &AppSettings) -> io::Result<()> {
    billinglog(machine, booking, config)?;

    let record = Record {
        machine,
        date: booking.creation_datetime.date_naive().to_string(),
        time_booked: booking.creation_datetime.time().to_string(),
        time_released: Local::now().time().to_string(),
        booking_duration: booking.creation_instant.elapsed().as_secs_f32().div(60.0).ceil() as _,
        runtime: booking.total_runtime().as_secs_f32().div(60.0).ceil() as _,
        user: &booking.user
    };

    let file_writer = File::options()
        .create(true)
        .append(true)
        .open(&config.machine_log)?;

    WriterBuilder::new()
        .has_headers(false)
        .from_writer(file_writer)
        .serialize(&record)
        .map_err(|error| {
            red_ln!("error while serializing: {error}\n{record:#?}");
            io::ErrorKind::Other.into()
        })
}

pub fn log_debug(topic: &str, payload: &str, result: Result<(), &str>, config: &AppSettings) -> io::Result<()> {
    if let Err(error) = result {
        red_ln!("error: {error}");
        red_ln!("	topic: {topic}");
        red_ln!("	payload: {payload}");
        red_ln!();
    }

    let time = Local::now().to_string();
    let result = result.err().unwrap_or("ok");

    let record = format!("
time:    {time}
topic:   {topic}
payload: {payload}
result:  {result}",
    );

    File::options()
        .append(true)
        .open(&config.debug_log)?
        .write_all(record.as_bytes())
}

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

pub fn billinglog(machine: &str, booking: &Booking, config: &AppSettings) -> io::Result<()> {
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