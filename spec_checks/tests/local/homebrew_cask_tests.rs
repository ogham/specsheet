use super::*;
use spec_checks::homebrew_cask::{HomebrewCaskCheck, RunBrewCask};
use pretty_assertions::assert_eq;


struct MockHomebrewCask(&'static str);

impl RunBrewCask for MockHomebrewCask {
    fn find_cask(&self, _: &mut Executor, cask_name: &str) -> Result<bool, Rc<ExecError>> {
        Ok(cask_name == self.0)
    }
}


// ---- regular tests ----

#[test]
fn installed() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = "alacritty"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Cask ‘alacritty’ is installed");

    let results = check.check(&mut Executor::new(), &MockHomebrewCask("alacritty"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockHomebrewCask("exa"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not installed",
    ]);
}


#[test]
fn missing() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = "alacritty"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Cask ‘alacritty’ is not installed");

    let results = check.check(&mut Executor::new(), &MockHomebrewCask("exa"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is not installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockHomebrewCask("alacritty"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is installed",
    ]);
}


// ---- parameter combinations ----

#[test]
fn installed_explicitly() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask   = "alacritty"
        state = "installed"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Cask ‘alacritty’ is installed");
}


// ---- invalid string errors ----

#[test]
fn err_slashed_cask() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = "wib/wob"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘cask’ value ‘\"wib/wob\"’ is invalid (it must not contain a ‘/’ character)");
}

#[test]
fn err_bad_state() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = "wib"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"oobleck\"’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_cask_name() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘cask’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘cask’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = HomebrewCaskCheck::read(&toml! {
        cask = "demux-redux"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = HomebrewCaskCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘cask’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = HomebrewCaskCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
