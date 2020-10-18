use super::*;
use spec_checks::ping::{PingCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn responds() {
    let check = PingCheck::read(&toml! {
        target = "192.168.0.1"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Pinging ‘192.168.0.1’ should receive a response");
}

#[test]
fn responds_explicitly() {
    let check = PingCheck::read(&toml! {
        target = "192.168.0.1"
        state = "responds"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Pinging ‘192.168.0.1’ should receive a response");
}

#[test]
fn no_response() {
    let check = PingCheck::read(&toml! {
        target = "192.168.0.1"
        state = "no-response"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Pinging ‘192.168.0.1’ should time out");
}


// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = PingCheck::read(&toml! {
        target = "aoeuaoeu"
        state = "filtered"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"filtered\"’ is invalid (it must be ‘responds’ or ‘no-response’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_target() {
    let check = PingCheck::read(&toml! {
        target = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘target’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_target_type() {
    let check = PingCheck::read(&toml! {
        target = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘target’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = PingCheck::read(&toml! {
        target = "some.host"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘responds’ or ‘no-response’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = PingCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘target’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = PingCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
