use std::{io, ops::Div};
use std::collections::HashMap;
use std::fs;
use std::fs::File;

use chrono::Local;
use colour::red_ln;
use csv::WriterBuilder;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::utils::booking::Booking; 

#[derive(Debug, Clone)]
pub struct UserData {
    id: Option<i32>,
    to_be_used: bool
}

#[derive(Debug, Clone)]
pub struct MachineData {
    id: Option<i32>,
    to_be_used: bool,
    power_sense: bool,
    divider: i32
}

lazy_static! {
    pub static ref DATA_USER: HashMap<String, UserData> = fs::read_to_string("DataUser.csv")
        .expect("failed to open DataUser.csv")
        .lines()
        .map(|line| {
            let mut splits = line.split(',');
            
            let name = splits.next().unwrap().to_string();

            let ud = UserData {
                id        : splits.next().unwrap().parse       ().ok(),
                to_be_used: splits.next().unwrap().parse::<i32>().unwrap_or(1) == 1,
            };
            
            (name, ud)
        })
        .collect();
    pub static ref DATA_MACHINES: HashMap<String, MachineData> = fs::read_to_string("DataMachines.csv")
        .expect("failed to open DataMachines.csv")
        .lines()
        .map(|line| {
            let mut splits = line.split(',');
            
            let name = splits.next().unwrap().to_string();
            let md = MachineData {
                id         : splits.next().unwrap().parse       ().ok(),
                to_be_used : splits.next().unwrap().parse::<i32>().unwrap_or(1) == 1,
                power_sense: splits.next().unwrap().parse::<i32>().unwrap()     == 1,
                divider    : splits.next().unwrap().parse       ().unwrap()
            };
            
            (name, md)
        })
        .collect();
}

#[derive(Debug, Serialize)]
struct BillingRecord {
    user_id: String,                      // id nachschlagen in DataUser.csv. Wenn Spalte 3 ("toBeUsed") == 0 dann skip. Wenn nicht vorhanden dann fallback zum Namen
    quelle: &'static str,                 // "allgemeiner Beleg"
    brutto_netto: i32,                    // 2
    artikel_id: String,                      // DataMachine.csv#2
    positionsdetails: String,             // Date
    anzahl: i32,                          // minutes divided by DataMachine.csv#5 (ceil)
    rechnungstyp: i32                     // 0
}

pub fn billinglog(machine: &str, booking: &Booking) -> io::Result<()> {
    let user_id =
        if let Some(user_data) = &DATA_USER.get(&booking.user.to_string()) {
            if !user_data.to_be_used { return Ok(()); }
            user_data
                .id
                .map(|i| i.to_string())
                .unwrap_or(booking.user.to_string())
        } else {
            booking.user.to_string()
        };
        
    let artikel_id =
        if let Some(machine_data) = &DATA_MACHINES.get(&machine.to_string()) {
            if !machine_data.to_be_used { return Ok(()); }
            machine_data
                .id
                .map(|i| i.to_string())
                .unwrap_or(machine.to_string())
        } else {
            machine.to_string()
        };
    
    let bill = BillingRecord {
        user_id,
        quelle: "allgemeiner Beleg",
        brutto_netto: 2,
        artikel_id,
        positionsdetails: Local::now()
            .format("%Y-%m-%d")
            .to_string(),
        anzahl: booking
            .total_runtime()
            .as_secs_f32()
            .div(60.0)
            .ceil()
            as _,
        rechnungstyp: 0,
    };
    
    let file_writer = File::options()
        .create(true)
        .append(true)
        .open("billinglog.csv")?;

    WriterBuilder::new()
        .has_headers(false)
        .from_writer(file_writer)
        .serialize(&bill)
        .map_err(|error| {
            red_ln!("error while serializing: {error}\n{bill:#?}");
            io::ErrorKind::Other.into()
        })
}