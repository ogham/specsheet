use super::*;
use spec_checks::npm::{NpmCheck, RunNpm};
use pretty_assertions::assert_eq;


struct MockNpm(&'static str);

impl RunNpm for MockNpm {
    fn find_package(&self, _: &mut Executor, global_package_name: &str) -> Result<bool, Rc<ExecError>> {
        Ok(global_package_name == self.0)
    }
}


// ---- regular tests ----

#[test]
fn installed() {
    let check = NpmCheck::read(&toml! {
        package = "typescript"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘typescript’ is installed");

    let results = check.check(&mut Executor::new(), &MockNpm("typescript"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockNpm("border-collie"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not installed",
    ]);
}


#[test]
fn missing() {
    let check = NpmCheck::read(&toml! {
        package = "typescript"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘typescript’ is not installed");

    let results = check.check(&mut Executor::new(), &MockNpm("border-collie"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is not installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockNpm("typescript"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is installed",
    ]);
}


// ---- parameter combinations ----

#[test]
fn installed_explicitly() {
    let check = NpmCheck::read(&toml! {
        package = "typescript"
        state = "installed"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Package ‘typescript’ is installed");
}


// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = NpmCheck::read(&toml! {
        package = "wib"
        state = "demi"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"demi\"’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_package_name() {
    let check = NpmCheck::read(&toml! {
        package = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = NpmCheck::read(&toml! {
        package = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = NpmCheck::read(&toml! {
        package = "demux-redux"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = NpmCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘package’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = NpmCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
