use super::*;
use spec_checks::group::{GroupCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn exists() {
    let check = GroupCheck::read(&toml! {
        group = "folk"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Group ‘folk’ exists");
}

#[test]
fn missing() {
    let check = GroupCheck::read(&toml! {
        group = "folk"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Group ‘folk’ does not exist");
}


// ---- parameter combinations ----

#[test]
fn exists_explicitly() {
    let check = GroupCheck::read(&toml! {
        group = "folk"
        state = "present"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Group ‘folk’ exists");
}


// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = GroupCheck::read(&toml! {
        group = "folk"
        state = "ish"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"ish\"’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_group_name() {
    let check = GroupCheck::read(&toml! {
        group = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘group’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = GroupCheck::read(&toml! {
        group = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘group’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = GroupCheck::read(&toml! {
        group = "folk"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = GroupCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘group’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = GroupCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
