use std::ops::Div;
use std::io::{self, Write};
use std::fs::File;

use chrono::Local;
use colour::red_ln;
use csv::WriterBuilder;
use serde::Serialize;

use crate::utils::booking::Booking;

#[derive(Debug, Serialize)]
struct Record<'s> {
    machine: &'s str,
    date: String,
    time_booked: String,
    time_released: String,
    booking_duration: i32, //minutes
    runtime: i32, //minutes
    user: &'s str
}

pub fn machinelog(machine: &str, booking: &Booking) -> io::Result<()> {
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
        .append(true)
        .open("/root/machinelog.csv")?;

    WriterBuilder::new()
        .has_headers(false)
        .from_writer(file_writer)
        .serialize(&record)
        .map_err(|error| {
            red_ln!("error while serializing: {error}\n{record:#?}");
            io::ErrorKind::Other.into()
        })
}

pub fn log_debug(topic: &str, payload: &str, result: Result<(), &str>) -> io::Result<()> {
    if let Err(error) = result {
        red_ln!("error: {error}");
        red_ln!("	topic: {topic}");
        red_ln!("	payload: {payload}");
        red_ln!()
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
        .open("/root/machinelog_debug.csv")?
        .write_all(record.as_bytes())
}