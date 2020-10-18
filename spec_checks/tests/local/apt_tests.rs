use super::*;
use spec_checks::apt::{AptCheck, RunApt};
use pretty_assertions::assert_eq;


struct MockApt(&'static str, &'static str);

impl RunApt for MockApt {
    fn find_package(&self, _: &mut Executor, package_name: &str) -> Result<Option<String>, Rc<ExecError>> {
        if package_name == self.0 {
            Ok(Some(self.1.into()))
        }
        else {
            Ok(None)
        }
    }
}


// ---- regular tests ----

#[test]
fn installed() {
    let check = AptCheck::read(&toml! {
        package = "wibble-wobble"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘wibble-wobble’ is installed");

    let results = check.check(&mut Executor::new(), &MockApt("wibble-wobble", "v3.1.4"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockApt("something-else", "v3.1.4"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not installed",
    ]);
}


#[test]
fn installed_with_version() {
    let check = AptCheck::read(&toml! {
        package = "wibble-wobble"
        version = "v3.1.4"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘wibble-wobble’ version ‘v3.1.4’ is installed");

    let results = check.check(&mut Executor::new(), &MockApt("wibble-wobble", "v3.1.4"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
        "PASS version ‘v3.1.4’ is installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockApt("something-else", "v3.1.4"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockApt("wibble-wobble", "v2.2.8"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
        "FAIL version ‘v2.2.8’ is installed",
    ]);
}


#[test]
fn missing() {
    let check = AptCheck::read(&toml! {
        package = "wibble-wobble"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘wibble-wobble’ is not installed");

    let results = check.check(&mut Executor::new(), &MockApt("another-package", ""));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is not installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockApt("wibble-wobble", "v3.1.4"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is installed",
    ]);
}


// ---- parameter combinations ----

#[test]
fn installed_explicitly() {
    let check = AptCheck::read(&toml! {
        package = "wibble-wobble"
        state   = "installed"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘wibble-wobble’ is installed");
}


// ---- invalid parameter combination errors ----

#[test]
fn err_missing_with_version() {
    let check = AptCheck::read(&toml! {
        package = "foo"
        state = "missing"
        version = "0.23"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘version’ is inappropriate when parameter ‘state’ is ‘\"missing\"’");
}


// ---- invalid string errors ----

#[test]
fn err_slashful_package_name() {
    let check = AptCheck::read(&toml! {
        package = "Europe/London"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ value ‘\"Europe/London\"’ is invalid (it must not contain a ‘/’ character)");
}

#[test]
fn err_bad_state() {
    let check = AptCheck::read(&toml! {
        package = "wib"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"oobleck\"’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_package_name() {
    let check = AptCheck::read(&toml! {
        package = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_version() {
    let check = AptCheck::read(&toml! {
        package = "wib"
        version = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘version’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = AptCheck::read(&toml! {
        package = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_version_type() {
    let check = AptCheck::read(&toml! {
        package = "twenty-three"
        version = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘version’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = AptCheck::read(&toml! {
        package = "twenty-three"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = AptCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = AptCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
