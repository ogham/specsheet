use super::*;
use spec_checks::user::{UserCheck};
use spec_checks::read::Rewrites;
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn exists() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "User ‘bethany’ exists");
}

#[test]
fn missing() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        state = "missing"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "User ‘bethany’ does not exist");
}

#[test]
fn exists_with_login_shell() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        login_shell = "/usr/local/bin/fish"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "User ‘bethany’ exists with login shell ‘/usr/local/bin/fish’");
}

#[test]
fn exists_with_groups() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        groups = [ "these", "those" ]
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "User ‘bethany’ exists and is a member of groups ‘these’ and ‘those’");
}


// ---- parameter combinations ----

#[test]
fn exists_explicitly() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        state = "present"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "User ‘bethany’ exists");
}

#[test]
fn everything() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        login_shell = "/usr/local/bin/fish"
        groups = [ "these", "those" ]
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "User ‘bethany’ exists with login shell ‘/usr/local/bin/fish’ and is a member of groups ‘these’ and ‘those’");
}


// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        state = "ish"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"ish\"’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_user_name() {
    let check = UserCheck::read(&toml! {
        user = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘user’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_login_shell() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        login_shell = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘login_shell’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_group_name() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        groups = [""]
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘groups’ value ‘[\"\"]’ is invalid (group names must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_name_type() {
    let check = UserCheck::read(&toml! {
        user = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘user’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_login_shell_type() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        login_shell = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘login_shell’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_groups_type() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        groups = "wheel"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘groups’ value ‘\"wheel\"’ is invalid (it must be an array of strings)");
}

#[test]
fn err_invalid_group_type() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        groups = [[]]
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘groups’ value ‘[[]]’ is invalid (it must be an array of strings)");
}

#[test]
fn err_invalid_state_type() {
    let check = UserCheck::read(&toml! {
        user = "bethany"
        state = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = UserCheck::read(&Map::new().into(), &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘user’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = UserCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
