//! The DNS remote check involves making a DNS request, and checking the
//! response.
//!
//! # Check example
//!
//! ```toml
//! [[dns]]
//! domain = "millimeter.io"
//! type = "A"
//! value = "159.89.251.132"
//! ```
//!
//! # Commands
//!
//! This check works by running `dig`.


use std::fmt;
use std::net::IpAddr;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// The DNS check makes a DNS request and checks the response.
#[derive(PartialEq, Debug)]
pub struct DnsCheck {
    request: Request,
    condition: Condition,
}

/// The details of a DNS that can be made.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Request {

    /// Which nameserver should be used for this request.
    pub nameserver: Nameserver,

    /// The domain to query.
    pub domain: String,

    /// The record type to specify during the query.
    pub rtype: RecordType,
}

/// Which nameserver should be used for this request.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum Nameserver {

    /// Use the system default resolver.
    DefaultResolver,

    /// Use the DNS server at the given address.
    ByIP(IpAddr),
}

/// The DNS record type specified in the query.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    MX,
    TXT,
}

/// The condition we are checking on the response.
#[derive(PartialEq, Debug)]
enum Condition {

    /// There should be a value present for this domain and type.
    Present(String),

    /// There should be no value present for this domain and type.
    Missing,
}

// ---- the check description ----

impl fmt::Display for DnsCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { request, condition } = &self;

        write!(f, "DNS ‘{:?}’ record for ‘{}’", request.rtype, request.domain)?;

        match condition {
            Condition::Present(cond)  => write!(f, " exists with value ‘{}’", cond)?,
            Condition::Missing        => write!(f, " is missing")?,
        }

        if let Nameserver::ByIP(ip) = &request.nameserver {
            write!(f, " (according to {})", ip)?;
        }

        Ok(())
    }
}


// ---- reading from TOML ----

impl Check for DnsCheck {
    const TYPE: &'static str = "dns";
}

impl DnsCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["nameserver", "domain", "type", "state", "value"])?;

        let request = Request::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { request, condition })
    }
}

impl Request {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let nameserver = Nameserver::read(table)?;
        let domain = table.get_or_read_error("domain")?.string_or_error("domain")?;
        if domain.is_empty() {
            return Err(ReadError::invalid("domain", domain.into(), "it must not be empty"));
        }

        let rtype = RecordType::read(table)?;
        Ok(Self { nameserver, domain, rtype })
    }
}

impl Nameserver {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        if let Some(ns_value) = table.get("nameserver") {
            let ns = ns_value.string_or_error("nameserver")?;

            match ns.parse() {
                Ok(ip) => {
                    Ok(Self::ByIP(ip))
                }
                Err(e) => {
                    warn!("Error parsing IP address {:?}: {}", ns, e);
                    Err(ReadError::invalid("nameserver", ns_value.clone(), "it must be an IP address"))
                }
            }
        }
        else {
            Ok(Self::DefaultResolver)
        }
    }
}

impl RecordType {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let rtype = &table.get_or_read_error("type")?
                          .string_or_error2("type", "it must be a string such as ‘A’, ‘MX’, ‘SRV’...")?
                          .to_ascii_uppercase()[..];

        match rtype {
            "A"    => Ok(Self::A),
            "AAAA" => Ok(Self::AAAA),
            "CAA"  => Ok(Self::CAA),
            "MX"   => Ok(Self::MX),
            "TXT"  => Ok(Self::TXT),
            other  => Err(ReadError::invalid("type", other.into(), "it must be a string such as ‘A’, ‘MX’, ‘SRV’...")),
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let value = table.get("value").map(|v| {
            v.string_or_error("value")
        }).transpose()?;

        if let Some(state_value) = table.get("state") {
            match &state_value.string_or_error("state")?[..] {
                "present" => {
                    // continue
                }
                "absent" => {
                    if value.is_some() {
                        return Err(ReadError::conflict2("value", "state", state_value.clone()));
                    }
                    else {
                        return Ok(Self::Missing);
                    }
                }
                _ => {
                    return Err(ReadError::invalid("state", state_value.clone(), OneOf(&["present", "absent"])));
                }
            }
        }

        if let Some(value) = value {
            Ok(Self::Present(value))
        }
        else {
            Err(ReadError::MissingParameter { parameter_name: "value" })
        }
    }
}


// ---- running the check ----

/// The interface to making DNS requests used by [`DnsCheck`].
pub trait RunDns {

    /// Primes the command for running a request.
    #[allow(unused)]
    fn prime(&mut self, request: &Request) { }

    /// Running the command if it hasn’t been run already, examines the
    /// output and returns the value in the DNS response.
    fn get_values(&self, executor: &mut Executor, request: &Request) -> Result<Vec<Rc<str>>, Rc<ExecError>>;
}

impl<D: RunDns> RunCheck<D> for DnsCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, dig: &mut D) {
        dig.prime(&self.request);
    }

    fn check(&self, executor: &mut Executor, dig: &D) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let results = match dig.get_values(executor, &self.request) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, results.is_empty()) {
            (Condition::Present(expected_value), false) => {
                if results.iter().any(|a| **a == *expected_value) {
                    vec![ CheckResult::Passed(Pass::RecordPresent) ]
                }
                else {
                    vec![ CheckResult::Failed(Fail::RecordDifferent { got_values: results }) ]
                }
            }
            (Condition::Present(_), true) => {
                vec![ CheckResult::Failed(Fail::RecordMissing) ]
            }
            (Condition::Missing, false) => {
                vec![ CheckResult::Failed(Fail::RecordPresent) ]
            }
            (Condition::Missing, true) => {
                vec![ CheckResult::Passed(Pass::RecordMissing) ]
            }
        }
    }
}

/// The successful result of a DNS check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// We were able to communicate with the DNS server and receive a response
    /// (even if the response is not correct).
    DnsSuccess,

    /// There is a record present with the correct value at the domain and type.
    RecordPresent,

    /// The domain exists, but there is no record for the given type.
    RecordMissing,
}

/// The failure result of running a DNS check.
#[derive(PartialEq, Debug)]
pub enum Fail {

    /// There was an error communicating with the DNS server.
    DnsFailure,

    /// We got a NXDOMAIN response from the DNS server.
    NoSuchDomain,

    /// No record exists for the given type.
    RecordMissing,

    /// There is a record for the given type.
    RecordPresent,

    /// There is a record for the given type, but its value was not what we
    /// were expecting.
    RecordDifferent {
        got_values: Vec<Rc<str>>,
    }
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DnsSuccess => {
                write!(f, "DNS connection succeeded")
            }
            Self::RecordPresent => {
                write!(f, "there is a record present")
            }
            Self::RecordMissing => {
                write!(f, "there is no record present")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DnsFailure => {
                write!(f, "DNS connection failed")
            }
            Self::NoSuchDomain => {
                write!(f, "no such domain")
            }
            Self::RecordMissing => {
                write!(f, "the record is missing")
            }
            Self::RecordPresent => {
                write!(f, "there is a record present")
            }
            Self::RecordDifferent { got_values } => {
                write!(f, "the record is different, got ‘{:?}’ instead", got_values)
            }
        }
    }
}
