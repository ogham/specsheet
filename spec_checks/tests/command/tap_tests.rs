use super::*;
use spec_checks::tap::{TapCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn tapped_output() {
    let check = TapCheck::read(&toml! {
        shell = "./some-prog"
    }).unwrap();

    assert_eq!(check.to_string(),
               "TAP tests for command ‘./some-prog’");
}

#[test]
fn tapped_output_with_environment() {
    let check = TapCheck::read(&toml! {
        shell = "./some-prog"
        environment = { "A" = "b", "C" = "d" }
    }).unwrap();

    assert_eq!(check.to_string(),
               "TAP tests for command ‘A=b C=d ./some-prog’");
}


// ---- empty string errors ----

#[test]
fn err_empty_shell_command() {
    let check = TapCheck::read(&toml! {
        shell = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘shell’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_shell_type() {
    let check = TapCheck::read(&toml! {
        shell = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘shell’ value ‘[]’ is invalid (it must be a string)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = TapCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘shell’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = TapCheck::read(&toml! {
        ouginoeuhinstoh = "oehsutnaoehutn"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘ouginoeuhinstoh’ is unknown");
}
