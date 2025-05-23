#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct Slave {
    pub runs_continuously: bool,
    pub needs_trailing_time: bool,
    pub topic: String,
    pub payload_on: String,
    pub payload_off: String,
}