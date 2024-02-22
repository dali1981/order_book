use crate::quote::Exchange;

pub struct IncomingMsg {
    pub exchange: Exchange,
    pub msg: String,
}