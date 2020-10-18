use std::convert::TryInto;
use std::fmt;

use log::*;

use crate::read::{TomlValue, ValueExtras, ReadError};


#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct PortNumber(pub u16);

impl PortNumber {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let port_value = table.get_or_read_error("port")?;

        match port_value.number_or_error("port")?.try_into() {
            Ok(port) => {
                if port > 0 {
                    Ok(Self(port))
                }
                else {
                    warn!("Port number was zero");
                    Err(ReadError::invalid("port", port_value.clone(), PortNumberOutOfRange))
                }
            }
            Err(out_of_range) => {
                warn!("Error parsing port number: {}", out_of_range);
                Err(ReadError::invalid("port", port_value.clone(), PortNumberOutOfRange))
            }
        }
    }
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct PortNumberOutOfRange;

impl fmt::Display for PortNumberOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "it must be between 1 and 65535")
    }
}
