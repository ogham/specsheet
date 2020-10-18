use super::*;
use spec_checks::systemd::{SystemdCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn service_is_running() {
    let check = SystemdCheck::read(&toml! {
        service = "sshd"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Service ‘sshd’ is running");
}

#[test]
fn service_is_explicitly_running() {
    let check = SystemdCheck::read(&toml! {
        service = "sshd"
        state = "running"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Service ‘sshd’ is running");
}

#[test]
fn service_is_stopped() {
    let check = SystemdCheck::read(&toml! {
        service = "sshd"
        state = "stopped"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Service ‘sshd’ is stopped");
}

#[test]
fn service_is_missing() {
    let check = SystemdCheck::read(&toml! {
        service = "sshd"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Service ‘sshd’ is missing");
}


// ---- invalid string errors ----

#[test]
fn err_slashy_service_name() {
    let check = SystemdCheck::read(&toml! {
        service = "Forest/Wilderness"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘service’ value ‘\"Forest/Wilderness\"’ is invalid (it must not contain a ‘/’ character)");
}

#[test]
fn err_bad_state() {
    let check = SystemdCheck::read(&toml! {
        service = "wibd"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"oobleck\"’ is invalid (it must be ‘running’ or ‘stopped’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_service_name() {
    let check = SystemdCheck::read(&toml! {
        service = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘service’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = SystemdCheck::read(&toml! {
        service = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘service’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = SystemdCheck::read(&toml! {
        service = "httpd"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘running’ or ‘stopped’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = SystemdCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘service’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = SystemdCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
