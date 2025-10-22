use std::{io::Write, ops::Div};
use std::io;
use std::fs::File;

use chrono::Local;
use colour::red_ln;
use csv::WriterBuilder;
use serde::Serialize;

use crate::{my_config::MyConfig, utils::booking::Booking};

use self::billing::billinglog;

pub mod billing;

#[derive(Debug, Serialize)]
struct Record<'string> {
    machine: &'string str,
    date: String,
    time_booked: String,
    time_released: String,
    booking_duration: i32, //minutes
    runtime: i32, //minutes
    user: &'string str
}

pub fn machinelog(machine: &str, booking: &Booking, config: &MyConfig) -> io::Result<()> {
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

pub fn log_debug(topic: &str, payload: &str, result: Result<(), &str>, config: &MyConfig) -> io::Result<()> {
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