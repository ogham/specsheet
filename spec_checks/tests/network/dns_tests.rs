use super::*;
use spec_checks::dns::{DnsCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn present() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        value = "159.65.215.200"
    }).unwrap();

    assert_eq!(check.to_string(),
               "DNS ‘A’ record for ‘millimeter.io’ exists with value ‘159.65.215.200’");
}

#[test]
fn missing() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        state = "absent"
    }).unwrap();

    assert_eq!(check.to_string(),
               "DNS ‘A’ record for ‘millimeter.io’ is missing");
}

#[test]
fn present_using_nameserver() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        value = "159.65.215.200"
        nameserver = "1.1.1.1"
    }).unwrap();

    assert_eq!(check.to_string(),
               "DNS ‘A’ record for ‘millimeter.io’ exists with value ‘159.65.215.200’ (according to 1.1.1.1)");
}


// ---- invalid parameter combination errors ----

#[test]
fn err_missing_with_value() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        state = "absent"
        value = "1.2.3.4"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘value’ is inappropriate when parameter ‘state’ is ‘\"absent\"’");
}

#[test]
fn err_state_with_no_value() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        state = "present"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘value’ is missing");
}

// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        value = ""
        state = "propagating"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"propagating\"’ is invalid (it must be ‘present’ or ‘absent’)");
}

#[test]
fn err_bad_type() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "CATS"
        value = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘type’ value ‘\"CATS\"’ is invalid (it must be a string such as ‘A’, ‘MX’, ‘SRV’...)");
}

#[test]
fn err_bad_rameserver() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "MX"
        value = "10 example.org"
        nameserver = "please mr postman"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘nameserver’ value ‘\"please mr postman\"’ is invalid (it must be an IP address)");
}


// ---- empty string errors ----

#[test]
fn err_empty_domain() {
    let check = DnsCheck::read(&toml! {
        domain = ""
        type = "A"
        value = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_domain_type() {
    let check = DnsCheck::read(&toml! {
        domain = []
        type = "A"
        value = "1.2.3.4"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_nameserver_type() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        nameserver = []
        type = "A"
        value = "1.2.3.4"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘nameserver’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        value = "1.2.3.4"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_type_type() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = []
        value = "1.2.3.4"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘type’ value ‘[]’ is invalid (it must be a string such as ‘A’, ‘MX’, ‘SRV’...)");
}

#[test]
fn err_invalid_value_type() {
    let check = DnsCheck::read(&toml! {
        domain = "millimeter.io"
        type = "A"
        value = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘value’ value ‘[]’ is invalid (it must be a string)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = DnsCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = DnsCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
