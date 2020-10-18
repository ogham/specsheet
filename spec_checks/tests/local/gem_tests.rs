use super::*;
use spec_checks::gem::{GemCheck, RunGem};
use pretty_assertions::assert_eq;


struct MockGem(&'static str);

impl RunGem for MockGem {
    fn find_gem(&self, _: &mut Executor, gem_name: &str) -> Result<bool, Rc<ExecError>> {
        Ok(gem_name == self.0)
    }
}


// ---- regular tests ----

#[test]
fn installed() {
    let check = GemCheck::read(&toml! {
        gem = "pry"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Gem ‘pry’ is installed");

    let results = check.check(&mut Executor::new(), &MockGem("pry"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockGem("something-else"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not installed",
    ]);
}


#[test]
fn missing() {
    let check = GemCheck::read(&toml! {
        gem = "pry"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Gem ‘pry’ is not installed");

    let results = check.check(&mut Executor::new(), &MockGem("another-gem"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is not installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockGem("pry"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is installed",
    ]);
}


// ---- parameter combinations ----

#[test]
fn installed_explicitly() {
    let check = GemCheck::read(&toml! {
        gem   = "pry"
        state = "installed"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Gem ‘pry’ is installed");
}


// ---- invalid string errors ----

#[test]
fn err_slashy_gem_name() {
    let check = GemCheck::read(&toml! {
        gem = "this/that"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘gem’ value ‘\"this/that\"’ is invalid (it must not contain a ‘/’ character)");
}

#[test]
fn err_whitespace_gem_name() {
    let check = GemCheck::read(&toml! {
        gem = "this and that"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘gem’ value ‘\"this and that\"’ is invalid (it must not contain whitespace)");
}

#[test]
fn err_bad_state() {
    let check = GemCheck::read(&toml! {
        gem = "wib"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"oobleck\"’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_gem_name() {
    let check = GemCheck::read(&toml! {
        gem = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘gem’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = GemCheck::read(&toml! {
        gem = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘gem’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = GemCheck::read(&toml! {
        gem = "demux-redux"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = GemCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘gem’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = GemCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
