use std::time::Duration;
use std::ops::Div;
use std::io::{self, Write};
use std::fs::File;

use chrono::Local;
use serde::Serialize;
use tap::Pipe;

use crate::utils::booking::Booking;

#[derive(Serialize)]
struct Record<'s> {
    machine: &'s str,
    date: String,
    time_booked: String,
    time_released: String,
    booking_duration: i32, //minutes
    runtime: Duration, //minutes
    user: &'s str
}

pub fn machinelog(machine: &str, booking: &Booking) -> io::Result<()> {
    let record = Record {
        machine,
        date: booking.creation_datetime.date_naive().to_string(),
        time_booked: booking.creation_datetime.time().to_string(),
        time_released: Local::now().time().to_string(),
        booking_duration: booking.creation_instant.elapsed().as_secs_f32().div(60.0).ceil() as _,
        runtime: booking.total_runtime(),
        user: &booking.user
    };

    File::options()
        .append(true)
        .open("/root/machinelog")?
        .pipe(csv::Writer::from_writer)
        .serialize(record)
        .map_err(|_| io::ErrorKind::Other.into())
}

pub fn log_debug(topic: &str, payload: &str, result: Result<(), &str>) -> io::Result<()> {
    if let Err(error) = result {
        println!("error: {error}");
        println!("	topic: {topic}");
        println!("	payload: {payload}");
        println!()
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