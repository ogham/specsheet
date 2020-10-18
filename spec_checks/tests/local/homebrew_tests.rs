use super::*;
use spec_checks::homebrew::{HomebrewCheck, RunBrew};
use pretty_assertions::assert_eq;


struct MockHomebrew(&'static str);

impl RunBrew for MockHomebrew {
    fn find_formula(&self, _: &mut Executor, formula_name: &str) -> Result<bool, Rc<ExecError>> {
        Ok(formula_name == self.0)
    }
}


// ---- regular tests ----

#[test]
fn installed() {
    let check = HomebrewCheck::read(&toml! {
        formula = "pry"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Formula ‘pry’ is installed");

    let results = check.check(&mut Executor::new(), &MockHomebrew("pry"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockHomebrew("something-else"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is not installed",
    ]);
}


#[test]
fn missing() {
    let check = HomebrewCheck::read(&toml! {
        formula = "pry"
        state = "missing"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Formula ‘pry’ is not installed");

    let results = check.check(&mut Executor::new(), &MockHomebrew("another-formula"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "PASS it is not installed",
    ]);

    let results = check.check(&mut Executor::new(), &MockHomebrew("pry"));
    let phrases = results.into_iter().map(phrase).collect::<Vec<_>>();
    assert_eq!(phrases, vec![
        "FAIL it is installed",
    ]);
}


// ---- parameter combinations ----

#[test]
fn installed_explicitly() {
    let check = HomebrewCheck::read(&toml! {
        formula   = "pry"
        state = "installed"
    }).unwrap();

    assert_eq!(check.to_string(),
               "Formula ‘pry’ is installed");
}


// ---- invalid string errors ----

#[test]
fn err_slashy_gem_name() {
    let check = HomebrewCheck::read(&toml! {
        formula = "this/that"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘formula’ value ‘\"this/that\"’ is invalid (it must not contain a ‘/’ character)");
}

#[test]
fn err_whitespace_gem_name() {
    let check = HomebrewCheck::read(&toml! {
        formula = "this and that"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘formula’ value ‘\"this and that\"’ is invalid (it must not contain whitespace)");
}

#[test]
fn err_bad_state() {
    let check = HomebrewCheck::read(&toml! {
        formula = "wib"
        state = "oobleck"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"oobleck\"’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_gem_name() {
    let check = HomebrewCheck::read(&toml! {
        formula = ""
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘formula’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = HomebrewCheck::read(&toml! {
        formula = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘formula’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = HomebrewCheck::read(&toml! {
        formula = "demux-redux"
        state = []
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘installed’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = HomebrewCheck::read(&Map::new().into()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘formula’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = HomebrewCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
