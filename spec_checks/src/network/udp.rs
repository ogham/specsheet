//! UDP network checks
//!
//! # Check example
//!
//! ```toml
//! [[udp]]
//! port = 8302
//! state = 'no-response'
//! ```
//!
//! # Commands
//!
//! No commands are run for network checks; Specsheet deals with the network
//! itself.


use std::fmt;
use std::net::Ipv4Addr;

use log::*;

use crate::check::{Check, BuiltInCheck, CheckResult, PassResult, FailResult};
use crate::common::PortNumber;
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the network; which other machines the local computer can
/// communicate with.
#[derive(PartialEq, Debug)]
pub struct UdpCheck {
    request: Request,
    condition: Condition,
    ufw: Option<ExtraUfwCheck>,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Request {
    pub target: Option<String>,
    pub port: PortNumber,
    pub source: Source,
}

/// Where the request gets sent from.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Source {
    Automatic,
    Address(Ipv4Addr),
    Interface(String),
}

/// What we expect to learn about the port from the response, if any.
#[derive(PartialEq, Debug)]
enum Condition {
    Responds,
    NoResponse,
}

#[derive(PartialEq, Debug)]
struct ExtraUfwCheck {
    allow: String,
}


// ---- the check description ----

impl fmt::Display for UdpCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { request, condition, ufw } = &self;

        write!(f, "UDP port ‘{}’", request.port.0)?;

        if let Some(target) = &request.target {
            write!(f, " on ‘{}’", target)?;
        }

        if let Source::Address(ipv4_addr) = &request.source {
            write!(f, " from ‘{}’", ipv4_addr)?;
        }
        else if let Source::Interface(iface) = &request.source {
            write!(f, " from interface ‘{}’", iface)?;
        }

        if let Some(ufw) = ufw {
            write!(f, " (with UFW check to ‘{}’)", ufw.allow)?;
        }

        match condition {
            Condition::Responds => {
                write!(f, " responds")?;
            }
            Condition::NoResponse => {
                write!(f, " does not respond")?;
            }
        }

        Ok(())
    }
}


// ---- reading from TOML ----

impl Check for UdpCheck {
    const TYPE: &'static str = "udp";
}

impl UdpCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["port", "address", "source", "state", "ufw"])?;

        let request = Request::read(table)?;
        let condition = Condition::read(table)?;
        let ufw = ExtraUfwCheck::read(table)?;

        Ok(Self { request, condition, ufw })
    }
}

impl Request {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let port = PortNumber::read(table)?;
        let source = Source::read(table)?;

        let target = match table.get("address") {
            Some(a) => Some(a.string_or_error("address")?.parse().unwrap()),
            None    => None,
        };

        Ok(Self { target, port, source })
    }
}

impl Source {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let source_value = match table.get("source") {
            Some(s) => s,
            None    => return Ok(Self::Automatic),
        };

        let source = &source_value.string_or_error("source")?[..];
        if source.starts_with('%') {
            Ok(Self::Interface(source[1..].into()))
        }
        else if let Ok(address) = source.parse() {
            Ok(Self::Address(address))
        }
        else {
            Err(ReadError::invalid("source", source_value.clone(), "it must be an IP address or an interface"))
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let state = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Responds),
        };

        match &state.string_or_error2("state", OneOf(&["responds", "no-response"]))?[..] {
            "responds" => {
                Ok(Self::Responds)
            }
            "no-response" => {
                Ok(Self::NoResponse)
            }
            _ => {
                Err(ReadError::invalid("state", state.clone(), OneOf(&["responds", "no-response"])))
            }
        }
    }
}

impl ExtraUfwCheck {
    fn read(table: &TomlValue) -> Result<Option<Self>, ReadError> {
        if let Some(sub_table) = table.get("ufw") {
            sub_table.ensure_table("ufw")?;
            sub_table.ensure_only_keys(&["allow"])?;
            let allow = sub_table.get_or_read_error("allow")?
                                 .string_or_error("allow")?;

            Ok(Some(Self { allow }))
        }
        else {
            Ok(None)
        }
    }
}


// ---- running the check ----

/// The network interface used to send UDP packets by [`UdpCheck`].
pub trait RunUdp {

    /// Primes the command for running.
    #[allow(unused)]
    fn prime(&mut self, request: &Request) { }

    /// Running the command if it hasn’t been run already, sends a UDP
    /// packet and reports back if we received a response.
    fn send_udp_request(&self, request: &Request) -> bool;
}

impl<N: RunUdp> BuiltInCheck<N> for UdpCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, net: &mut N) {
        net.prime(&self.request)
    }

    fn check(&self, net: &N) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let result = net.send_udp_request(&self.request);

        match (&self.condition, result) {
            (Condition::Responds, true) => {
                vec![ CheckResult::Passed(Pass::ReceivedResponse) ]
            }
            (Condition::Responds, false) => {
                vec![ CheckResult::Failed(Fail::ConnectionRefused) ]
            }
            (Condition::NoResponse, true) => {
                vec![ CheckResult::Failed(Fail::ReceivedResponse) ]
            }
            (Condition::NoResponse, false) => {
                vec![ CheckResult::Passed(Pass::ConnectionRefused) ]
            }
        }
    }
}

impl Request {

    /// Returns the address to send packets to.
    pub fn addr(&self) -> (&str, u16) {
        match self.target {
            Some(ref s)  => (s,           self.port.0),
            None         => ("127.0.0.1", self.port.0),
        }
    }
}

/// The successful result of a network check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {
    ReceivedResponse,
    ConnectionRefused,
}

/// The failure result of running a network check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {
    ConnectionRefused,
    ReceivedResponse,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReceivedResponse => {
                write!(f, "received a response")
            }
            Self::ConnectionRefused => {
                write!(f, "connection refused")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionRefused => {
                write!(f, "connection refused")
            }
            Self::ReceivedResponse => {
                write!(f, "received a response")
            }
        }
    }
}
