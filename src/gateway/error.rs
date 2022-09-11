use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum GatewayError {
    InvalidOpCode,
    UnknownEvent(String),
    InvalidSession
}

impl Display for GatewayError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use GatewayError::*;

        match self {
            InvalidOpCode   => Display::fmt("Invalid OP code", f),
            UnknownEvent(e) => Display::fmt(&format!("Unknown event: {e}"), f),
            InvalidSession  => Display::fmt("Invalid session", f)
        }
    }
}