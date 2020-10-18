use super::*;
use spec_checks::cmd::{CommandCheck};
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn command_runs() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ executes");
}

#[test]
fn command_runs_with_status() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        status = 0
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ returns ‘0’");
}

#[test]
fn command_with_environment_runs() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        environment = { "JUMBUCK" = "tucker-bag" }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘JUMBUCK=tucker-bag ls’ executes");
}

#[test]
fn command_with_two_environments_runs() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        environment = { "A" = "b", "C" = "d" }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘A=b C=d ls’ executes");
}

#[test]
fn command_runs_with_empty_stderr() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        stderr = { empty = true }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ executes with empty stderr");
}

#[test]
fn command_runs_with_string_in_stdout() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        stdout = { string = "ERROR" }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ executes with stdout containing ‘ERROR’");
}


// ---- parameter combinations ----

#[test]
fn status_and_stderr() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        status = 7
        stderr = { empty = false }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ returns ‘7’ with non-empty stderr");
}

#[test]
fn nonempty_stdout_and_stderr() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        stdout = { empty = false }
        stderr = { empty = false }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ executes with non-empty stdout and stderr");
}

#[test]
fn empty_stdout_and_stderr() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        stdout = { empty = true }
        stderr = { empty = true }
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ executes with empty stdout and stderr");
}

#[test]
fn stdout_and_stderr_and_status() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        stdout = { empty = true }
        stderr = { empty = true }
        status = 44
    }).unwrap();

    assert_eq!(check.to_string(),
               "Command ‘ls’ returns ‘44’ with empty stdout and stderr");
}


// ---- empty string errors ----

#[test]
fn err_empty_shell_command() {
    let check = CommandCheck::read(&toml! {
        shell = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘shell’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_shell_type() {
    let check = CommandCheck::read(&toml! {
        shell = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘shell’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_status_type() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        status = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘status’ value ‘[]’ is invalid (it must be an integer)");
}


// ---- numeric errors ----

#[test]
fn err_status_too_high() {
    let check = CommandCheck::read(&toml! {
        shell = "ls"
        status = 9999999
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘status’ value ‘9999999’ is invalid (it must be between 0 and 255)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = CommandCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘shell’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = CommandCheck::read(&toml! {
        uehinuheisnthuesnh = "hsnhtndndndhdt"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘uehinuheisnthuesnh’ is unknown");
}
