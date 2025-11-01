use tap::Pipe;

use crate::schema::machine_capnp::machine::MachineState;

use crate::schema::machine_capnp::machine;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    pub category: String,
    pub description: String,
    pub id: String,
    pub name: String,
    pub urn: String,
    pub usage: Usage
}

impl TryFrom<machine::Reader<'_>> for Machine {
    type Error = capnp::Error;
    
    fn try_from(value: machine::Reader<'_>) -> capnp::Result<Self> {
        Self {
            category   : value.get_category()   ?.to_string().map_err(capnp::ErrorKind::TextContainsNonUtf8Data).map_err(capnp::Error::from_kind)?,
            description: value.get_description()?.to_string().map_err(capnp::ErrorKind::TextContainsNonUtf8Data).map_err(capnp::Error::from_kind)?,
            id         : value.get_id()         ?.to_string().map_err(capnp::ErrorKind::TextContainsNonUtf8Data).map_err(capnp::Error::from_kind)?,
            name       : value.get_name()       ?.to_string().map_err(capnp::ErrorKind::TextContainsNonUtf8Data).map_err(capnp::Error::from_kind)?,
            urn        : value.get_urn()        ?.to_string().map_err(capnp::ErrorKind::TextContainsNonUtf8Data).map_err(capnp::Error::from_kind)?,
            usage      : match value.get_state()? {
                MachineState::Free => Usage::Free,
                MachineState::InUse if value.has_inuse() => Usage::Yours,
                MachineState::InUse => Usage::Occupied,
                _ => Usage::Unknown // todo
            }
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString)]
pub enum Usage {
    Free,
    Yours,
    Occupied,
    Unknown
}
