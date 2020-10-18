use super::*;
use spec_checks::homebrew_tap::{HomebrewTapCheck, RunBrewTap};
use pretty_assertions::assert_eq;


struct MockHomebrewTap(&'static str);

impl RunBrewTap for MockHomebrewTap {
    fn find_tap(&self, _: &mut Executor, tap_name: &str) -> Result<bool, Rc<ExecError>> {
        Ok(tap_name == self.0)
    }
}


// ---- regular tests ----

#[test]
fn installed() {
    let check = HomebrewTapCheck::read(&toml! {
        tap = "cask/room"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Tap ‘cask/room’ is present");

    let results = check.check(&mut Executor::new(), &MockHomebrewTap("cask/room"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is present",
    ]);

    let results = check.check(&mut Executor::new(), &MockHomebrewTap("emul/ators"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not present",
    ]);
}


#[test]
fn missing() {
    let check = HomebrewTapCheck::read(&toml! {
        tap = "cask/room"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Tap ‘cask/room’ is not present");

    let results = check.check(&mut Executor::new(), &MockHomebrewTap("emul/ators"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is not present",
    ]);

    let results = check.check(&mut Executor::new(), &MockHomebrewTap("cask/room"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is present",
    ]);
}


// ---- parameter combinations ----

#[test]
fn installed_explicitly() {
    let check = HomebrewTapCheck::read(&toml! {
        tap   = "cask/room"
        state = "present"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Tap ‘cask/room’ is present");
}


// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = HomebrewTapCheck::read(&toml! {
        tap = "FutureSex/LoveSounds"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"oobleck\"’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_tap_name() {
    let check = HomebrewTapCheck::read(&toml! {
        tap = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘tap’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = HomebrewTapCheck::read(&toml! {
        tap = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘tap’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = HomebrewTapCheck::read(&toml! {
        tap = "demux/redux"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = HomebrewTapCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘tap’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = HomebrewTapCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
