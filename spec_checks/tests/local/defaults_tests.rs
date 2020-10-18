use super::*;
use spec_checks::defaults::{DefaultsCheck, DefaultsLocation, RunDefaults};
use spec_checks::read::Rewrites;
use pretty_assertions::assert_eq;


struct MockDefaults(DefaultsLocation, &'static str);

impl RunDefaults for MockDefaults {
    fn get_value(&self, _: &mut Executor, location: &DefaultsLocation) -> Result<Option<Rc<str>>, Rc<ExecError>> {
        if *location == self.0 {
            Ok(Some(self.1.into()))
        }
        else {
            Ok(None)
        }
    }
}


// ---- regular tests ----

#[test]
fn key_present() {
    let check = DefaultsCheck::read(&toml! {
        domain   = "Apple Global Domain"
        key      = "AppleAquaColorVariant"
        value    = 6
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "Defaults value ‘Apple Global Domain/AppleAquaColorVariant’ is ‘6’");
}

#[test]
fn key_missing() {
    let check = DefaultsCheck::read(&toml! {
        domain   = "Apple Global Domain"
        key      = "TireCount"
        state    = "absent"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "Defaults value ‘Apple Global Domain/TireCount’ is absent");
}

#[test]
fn key_present_in_file() {
    let check = DefaultsCheck::read(&toml! {
        file   = "~/Library/Containers/com.apple.Safari/Data/Library/Preferences/com.apple.Safari"
        key    = "ShowIconsInTabs"
        value  = 1
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "Defaults value ‘~/Library/Containers/com.apple.Safari/Data/Library/Preferences/com.apple.Safari/ShowIconsInTabs’ is ‘1’");
}


// ---- parameter combinations ----

#[test]
fn key_present_explicitly() {
    let check = DefaultsCheck::read(&toml! {
        domain   = "Apple Global Domain"
        key      = "AppleAquaColorVariant"
        value    = 6
        state    = "present"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "Defaults value ‘Apple Global Domain/AppleAquaColorVariant’ is ‘6’");
}


// ---- invalid parameter combination errors ----

#[test]
fn err_state_with_no_value() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = "key"
        state = "present"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘value’ is missing");
}

#[test]
fn err_missing_with_value() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = "key"
        value = "value"
        state = "absent"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘value’ is inappropriate when parameter ‘state’ is ‘\"absent\"’");
}

#[test]
fn err_neither_domain_nor_file() {
    let check = DefaultsCheck::read(&toml! {
        key = "quay"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ is missing");
}

#[test]
fn err_domain_with_file() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        file = "something"
        key = "key"
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ is inappropriate when parameter ‘file’ is given");
}


// ---- invalid string errors ----

#[test]
fn err_bad_state() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = "key"
        value = "value"
        state = "kinda"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"kinda\"’ is invalid (it must be ‘present’ or ‘absent’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_domain() {
    let check = DefaultsCheck::read(&toml! {
        domain = ""
        key = "key"
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_file() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        file = ""
        key = "key"
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘file’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_key() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = ""
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘key’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_domain_type() {
    let check = DefaultsCheck::read(&toml! {
        domain = []
        key = "key"
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘domain’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_file_type() {
    let check = DefaultsCheck::read(&toml! {
        file = []
        key = "key"
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘file’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_key_type() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = []
        value = "value"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘key’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = "key"
        value = "value"
        state = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘present’ or ‘absent’)");
}

#[test]
fn err_invalid_value_type() {
    let check = DefaultsCheck::read(&toml! {
        domain = "domain"
        key = "key"
        value = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘value’ value ‘[]’ is invalid (it must be a string or a number)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = DefaultsCheck::read(&Map::new().into(), &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘key’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = DefaultsCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
