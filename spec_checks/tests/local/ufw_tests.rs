use super::*;
use spec_checks::ufw::{UfwCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn tcp_rule_present() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        allow = "Anywhere"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Rule for TCP port ‘8080’ exists with allow ‘Anywhere’");
}

#[test]
fn udp_rule_present() {
    let check = UfwCheck::read(&toml! {
        port = 53
        protocol = "udp"
        allow = "Anywhere"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Rule for UDP port ‘53’ exists with allow ‘Anywhere’");
}

#[test]
fn tcp_rule_missing() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Rule for TCP port ‘8080’ does not exist");
}

#[test]
fn udp_rule_ipv6() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "udp"
        allow = "Anywhere"
        ipv6 = true
    }).unwrap();

    assert_eq!(check.to_string(),
               "Rule for UDP port ‘8080’ (IPv6) exists with allow ‘Anywhere’");
}


// ---- parameter combinations ----

#[test]
fn present_explicitly() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "udp"
        state = "present"
        allow = "Anywhere"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Rule for UDP port ‘8080’ exists with allow ‘Anywhere’");
}


// ---- invalid parameter combination errors ----

#[test]
fn err_missing_with_allow() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        state = "missing"
        allow = "Anywhere"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘allow’ is inappropriate when parameter ‘state’ is ‘\"missing\"’");
}


// ---- invalid string/value errors ----

#[test]
fn err_port_too_high() {
    let check = UfwCheck::read(&toml! {
        port = 99999
        protocol = "tcp"
        allow = "Anywhere"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ value ‘99999’ is invalid (it must be between 1 and 65535)");
}

#[test]
fn err_bad_protocol() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "sctp"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘protocol’ value ‘\"sctp\"’ is invalid (it must be ‘tcp’ or ‘udp’)");
}

#[test]
fn err_bad_state() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        state = "filtered"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"filtered\"’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_allow() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        allow = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘allow’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_port_type() {
    let check = UfwCheck::read(&toml! {
        port = []
        protocol = "tcp"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ value ‘[]’ is invalid (it must be an integer)");
}

#[test]
fn err_invalid_protocol_type() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘protocol’ value ‘[]’ is invalid (it must be ‘tcp’ or ‘udp’)");
}

#[test]
fn err_invalid_state_type() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘present’ or ‘missing’)");
}

#[test]
fn err_invalid_allow_type() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        allow = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘allow’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_ipv6_type() {
    let check = UfwCheck::read(&toml! {
        port = 8080
        protocol = "tcp"
        ipv6 = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘ipv6’ value ‘[]’ is invalid (it must be a boolean)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = UfwCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘port’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = UfwCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
