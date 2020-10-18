use super::*;
use spec_checks::http::{HttpCheck};
use spec_checks::read::Rewrites;
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn http_call_succeeds() {
    let check = HttpCheck::read(&toml! {
        url = "https://example.com/"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "HTTP request to ‘https://example.com/’ succeeds");
}

#[test]
fn http_call_succeeds_with_status() {
    let check = HttpCheck::read(&toml! {
        url = "https://example.com/"
        status = 200
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "HTTP request to ‘https://example.com/’ has status ‘200’");
}


// ---- empty string errors ----

#[test]
fn err_empty_url() {
    let check = HttpCheck::read(&toml! {
        url = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘url’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_url_type() {
    let check = HttpCheck::read(&toml! {
        url = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘url’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_body_type() {
    let check = HttpCheck::read(&toml! {
        url = "index.html"
        body = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘body’ value ‘[]’ is invalid (it must be a table)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = HttpCheck::read(&Map::new().into(), &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘url’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = HttpCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
