use super::*;
use spec_checks::udp::{UdpCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn port_open() {
    let check = UdpCheck::read(&toml! {
        port = 8080
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ responds");
}

#[test]
fn port_closed() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        state = "no-response"
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ does not respond");
}

#[test]
fn port_open_at_address() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        address = "127.0.0.1"
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ on ‘127.0.0.1’ responds");
}

#[test]
fn port_open_from_address() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        source = "127.0.0.1"
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ from ‘127.0.0.1’ responds");
}

#[test]
fn port_open_from_interface() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        source = "%eth1"
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ from interface ‘eth1’ responds");
}


// ---- parameter combinations ----

#[test]
fn explicitly_open() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        state = "responds"
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ responds");
}

#[test]
fn everything() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        address = "192.168.3.3"
        source = "%eth1"
        state = "no-response"
    }).unwrap();

    assert_eq!(check.to_string(),
               "UDP port ‘8080’ on ‘192.168.3.3’ from interface ‘eth1’ does not respond");
}


// ---- invalid value errors ----

#[test]
fn err_port_too_low() {
    let check = UdpCheck::read(&toml! {
        port = 0
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ value ‘0’ is invalid (it must be between 1 and 65535)");
}

#[test]
fn err_port_too_high() {
    let check = UdpCheck::read(&toml! {
        port = 99999
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ value ‘99999’ is invalid (it must be between 1 and 65535)");
}

#[test]
fn err_invalid_source_name() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        source = "???"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘source’ value ‘\"???\"’ is invalid (it must be an IP address or an interface)");
}


// ---- invalid value errors ----

#[test]
fn err_bad_state() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        state = "filtered"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"filtered\"’ is invalid (it must be ‘responds’ or ‘no-response’)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_port_type() {
    let check = UdpCheck::read(&toml! {
        port = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ value ‘[]’ is invalid (it must be an integer)");
}

#[test]
fn err_invalid_address_type() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        address = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘address’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_source_type() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        source = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘source’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_ufw_type() {
    let check = UdpCheck::read(&toml! {
        port = 8080
        ufw = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘ufw’ value ‘[]’ is invalid (it must be a table)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = UdpCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = UdpCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
