use std::collections::HashMap;
use std::sync::Arc;

use strum::{EnumIs, EnumTryAs};

use crate::booking::Booking;
use super::*;

pub type Name = String;
pub type Map = HashMap<Name, super::Resource>;

#[derive(Debug, Clone, EnumIs, EnumTryAs)]
pub enum State {
    Booked(Booking),
    Free
}

