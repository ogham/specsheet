use super::*;
use spec_checks::hashes::{HashCheck};
use spec_checks::read::Rewrites;
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn hash_matching() {
    let check = HashCheck::read(&toml! {
        path = "/usr/bin/specsheet"
        algorithm = "md5"
        hash = "3f22baaf4ba820a800dfc51af5ba1892"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/bin/specsheet’ has MD5 hash ‘3f22baaf4ba820a800dfc51af5ba1892’");
}


// ---- invalid string errors ----

#[test]
fn err_unknown_alorithm() {
    let check = HashCheck::read(&toml! {
        path = "/usr/bin/specsheet"
        algorithm = "SHREK 12"
        hash = "neuthinthinstaeobiasnubiansuoh"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘algorithm’ value ‘\"SHREK 12\"’ is invalid (it must be an algorithm such as ‘MD5’, ‘SHA256’...)");
}


// ---- empty string errors ----

#[test]
fn err_empty_path() {
    let check = HashCheck::read(&toml! {
        path = ""
        algorithm = "md5"
        hash = "3f22baaf4ba820a800dfc51af5ba1892"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘path’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_path_type() {
    let check = HashCheck::read(&toml! {
        path = []
        algorithm = "md5"
        hash = "3f22baaf4ba820a800dfc51af5ba1892"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘path’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_algorithm_type() {
    let check = HashCheck::read(&toml! {
        path = "/here"
        algorithm = []
        hash = "3f22baaf4ba820a800dfc51af5ba1892"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘algorithm’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_hash_type() {
    let check = HashCheck::read(&toml! {
        path = "/here"
        algorithm = "md5"
        hash = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘hash’ value ‘[]’ is invalid (it must be a string)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = HashCheck::read(&Map::new().into(), &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘path’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = HashCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
