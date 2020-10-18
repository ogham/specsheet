//! The HTTP check makes an HTTP request and checks the response.
//!
//! # Check example
//!
//! ```toml
//! [[http]]
//! url = "https://specsheet.software/hello.txt"
//! status = 200
//! ```
//!
//! # Commands
//!
//! This check works by running `curl`.


use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use log::*;
use mime::Mime;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::contents::{self, ContentsMatcher};
use crate::read::{TomlValue, ValueExtras, ReadError, Rewrites};

/// The HTTP check makes a HTTP request and checks the response.
#[derive(PartialEq, Debug)]
pub struct HttpCheck {

    /// The data sent in the request, including the URL.
    request: RequestParams,

    /// Test: What the response HTTP status should be.
    status: Option<i32>,

    /// Extra conditions for the HTTP headers.
    headers: HeaderConditions,

    body: Option<ContentsMatcher>,
}

/// The parameters that make up a complete HTTP request.
#[derive(PartialEq, Debug)]
pub struct RequestParams {

    /// The full URL of the request.
    pub url: String,

    /// Any extra HTTP headers to be sent.
    pub extra_headers: BTreeMap<String, String>,
}

#[derive(PartialEq, Debug)]
struct HeaderConditions {

    /// Test: What the response `Content-Type` header should be.
    content_type: Option<ContentTypeCheck>,

    /// Test: What the response `Location` header should be, assuming the HTTP
    /// status is a 3xx status.
    redirect_to: Option<String>,

    /// Test: What the response `Server` header should be.
    server: Option<String>,

    /// Test: What the request `Accept-Encoding` header should be, and thus,
    /// what the response `Content-Encoding` header should be.
    encoding: Option<String>,

    /// Test: A collection of other headers.
    also: BTreeMap<String, String>,
}

#[derive(PartialEq, Debug)]
enum ContentTypeCheck {

    /// A general type, such as `PNG`.
    Class(&'static str),

    /// An actual MIME type, such as `text/html`.
    MimeType(String),
}


// ---- the check description ----

impl fmt::Display for HttpCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { request, status, headers, body } = &self;

        write!(f, "HTTP request to ‘{}’", request.url)?;

        if let Some(status) = status {
            write!(f, " has status ‘{}’", status)?;
        }

        if let Some(ct) = &headers.content_type {
            if status.is_some() { write!(f, ",")?; }
            write!(f, " has content type ‘{}’", ct)?;
        }

        if let Some(r) = &headers.redirect_to {
            if headers.content_type.is_some() { write!(f, ",")?; }
            write!(f, " redirects to ‘{}’", r)?;
        }

        if let Some(s) = &headers.server {
            if headers.redirect_to.is_some() { write!(f, ",")?; }
            write!(f, " has server ‘{}’", s)?;
        }

        if let Some(e) = &headers.encoding {
            if headers.server.is_some() { write!(f, ",")?; }
            write!(f, " has encoding ‘{}’", e)?;
        }

        if let Some(contents_matcher) = body {
            if headers.encoding.is_some() { write!(f, ",")?; }
            contents_matcher.describe(f, "body")?;
        }

        if status.is_none() && headers.content_type.is_none() && headers.redirect_to.is_none()
        && headers.server.is_none() && headers.encoding.is_none() && body.is_none() {
            write!(f, " succeeds")?;
        }

        Ok(())
    }
}

impl fmt::Display for ContentTypeCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(class) => write!(f, "{}", class),
            Self::MimeType(mt) => write!(f, "{}", mt),
        }
    }
}


// ---- reading from TOML ----

impl Check for HttpCheck {
    const TYPE: &'static str = "http";
}

impl HttpCheck {
    pub fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["url", "headers", "status", "server", "encoding", "content_type", "redirect_to", "body", "also"])?;

        let request = RequestParams::read(table, rewrites)?;
        let status = table.get("status").map(|e| e.as_integer().unwrap() as i32);
        let headers = HeaderConditions::read(table, rewrites)?;
        let body = table.get("body").map(|e| ContentsMatcher::read("body", e)).transpose()?;
        Ok(Self { request, status, headers, body })
    }
}

impl RequestParams {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        let url_value = table.get_or_read_error("url")?;
        let url = rewrites.url(url_value.string_or_error("url")?);
        if url.is_empty() {
            return Err(ReadError::invalid("url", url_value.clone(), "it must not be empty"));
        }

        let extra_headers = table.get("headers")
                                 .map(|e| e.string_map_or_read_error("headers").unwrap())
                                 .unwrap_or_default();

        Ok(Self { url, extra_headers })
    }
}

impl HeaderConditions {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        Ok(Self {
            content_type: ContentTypeCheck::read(table)?,
            redirect_to: table.get("redirect_to").map(|e| e.string_or_error("redirect_to")).transpose()?.map(|e| rewrites.url(e)),
            server: table.get("server").map(|e| e.string_or_error("server")).transpose()?,
            encoding: table.get("encoding").map(|e| e.string_or_error("encoding")).transpose()?,
            also: table.get("also").map(|e| e.string_map_or_read_error("also")).transpose()?.unwrap_or_default(),
        })
    }
}

impl ContentTypeCheck {
    fn read(table: &TomlValue) -> Result<Option<Self>, ReadError> {
        let ct1 = match table.get("content_type") {
            Some(ct)  => ct,
            None      => { return Ok(None); },
        };

        let ct = ct1.string_or_error("content_type")?;
        if ct.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            let classes = &[
                "ATOM", "CSS", "EOT", "GIF", "HTML", "ICO", "JPEG", "JS", "JSON",
                "OTF", "PDF", "PNG", "SVG", "TTF", "WOFF", "WOFF2", "XML", "ZIP",
                "WEBP", "FLIF", "TXT", "JSONFEED",
            ];

            if let Some(class) = classes.iter().find(|e| ***e == *ct) {
                Ok(Some(Self::Class(class)))
            }
            else {
                Err(ReadError::invalid("content_type", ct1.clone(), "it must be a valid content type"))
            }
        }
        else {
            Ok(Some(Self::MimeType(ct)))
        }
    }
}


// ---- running the check ----

/// The interface to making HTTP requests used by [`HttpCheck`].
pub trait RunHttp {

    /// The result of making an HTTP response.
    type Output: HttpResponse;

    /// Primes the command for running, to make the given HTTP request.
    #[allow(unused)]
    fn prime(&mut self, request: HttpRequest, print_body: bool) { }

    /// Running the command if it hasn’t been run already for the given
    /// request, examine the result and return its fields as an output
    /// value.
    fn get_response(&self, executor: &mut Executor, request: HttpRequest) -> Result<Rc<Self::Output>, Rc<ExecError>>;
}

/// Accessors for parts of an HTTP response.
pub trait HttpResponse {

    /// The HTTP status.
    fn status(&self) -> Option<i32>;

    /// The `Content-Type` header.
    fn content_type(&self) -> Option<&str>;

    /// The `Content-Encoding` header.
    fn encoding(&self) -> Option<&str>;

    /// The `Location` header.
    fn location(&self) -> Option<&str>;

    /// The value of an arbitrary header.
    fn header(&self, header_name: &str) -> Option<&str>;

    /// The HTTP body, as bytes.
    fn body(&self) -> Vec<u8>;
}

/// The fields that make up an HTTP request. Requests get made by a type
/// that implements [`RunHttp`].
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct HttpRequest {

    /// The URL to fetch.
    pub url: String,

    /// Any extra HTTP headers to send as part of the request.
    pub headers: BTreeMap<String, String>,
}


impl HttpCheck {

    /// Puts together a request value given the parameters passed to this
    /// HTTP check.
    fn curl_request(&self) -> HttpRequest {
        let mut extra_headers = self.request.extra_headers.clone();

        if let Some(encoding) = &self.headers.encoding {
            extra_headers.insert("Accept-Encoding".into(), encoding.clone());
        }

        HttpRequest {
            url: self.request.url.clone(),
            headers: extra_headers,
        }
    }
}

impl<H: RunHttp> RunCheck<H> for HttpCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, curl: &mut H) {
        curl.prime(self.curl_request(), self.body.is_some());
    }

    fn check(&self, executor: &mut Executor, curl: &H) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let rs = match curl.get_response(executor, self.curl_request()) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        let mut results = vec![ CheckResult::Passed(Pass::HttpSucceeded) ];

        let got_status = match rs.status() {
            Some(stat) => stat,
            None       => return vec![ CheckResult::Failed(Fail::HttpFailed) ],
        };

        if let Some(status) = self.status {
            results.push(self.status_result(status, got_status));
        }

        if let Some(check) = &self.headers.content_type {
            results.push(self.content_type_result(&*rs, check));
        }

        if let Some(redirect) = &self.headers.redirect_to {
            results.push(self.redirect_result(&redirect, rs.location(), got_status));
        }

        if let Some(encoding) = &self.headers.encoding {
            results.push(self.encoding_result(&*rs, &encoding));
        }

        if let Some(content_matcher) = &self.body {
            results.push(self.body_result(&rs.body(), content_matcher));
        }

        for (header, expected) in &self.headers.also {
            if let Some(actual) = rs.header(header) {
                if actual == expected {
                    results.push(CheckResult::Passed(Pass::HeaderMatch(header.into())));
                }
                else {
                    results.push(CheckResult::Failed(Fail::HeaderMismatch(header.into(), actual.into())));
                }
            }
            else {
                results.push(CheckResult::Failed(Fail::HeaderMissing(header.into())));
            }
        }

        results
    }
}

impl HttpCheck {

    /// The check result that should be added to the list, given expected and
    /// received HTTP statuses.
    fn status_result(&self, expected_status: i32, got_status: i32) -> CheckResult<Pass, Fail> {
        if got_status == expected_status {
            CheckResult::Passed(Pass::StatusMatch)
        }
        else {
            CheckResult::Failed(Fail::StatusMismatch(got_status))
        }
    }

    /// The check result for `Content-Type` header values.
    fn content_type_result(&self, rs: &impl HttpResponse, check: &ContentTypeCheck) -> CheckResult<Pass, Fail> {
        if let ContentTypeCheck::Class(class) = check {
            let actual = match rs.content_type() {
                Some(a) => a,
                None    => return CheckResult::Failed(Fail::ContentTypeMissing),
            };

            let mime: Mime = match actual.parse() {
                Ok(mime) => {
                    mime
                }
                Err(e) => {
                    warn!("Error parsing MIME type: {}", e);
                    return CheckResult::Failed(Fail::InvalidMimeType(actual.into()));
                }
            };

            let matches = match *class {
                "ATOM"     => mime_is_p(&mime, "application", "atom", "xml"),
                "CSS"      => mime_is  (&mime, "text",        "css"),
                "EOT"      => mime_is  (&mime, "application", "vnd.ms-fontobject"),
                "FLIF"     => mime_is  (&mime, "image",       "flif"),
                "GIF"      => mime_is  (&mime, "image",       "gif"),
                "ICO"      => mime_is  (&mime, "image",       "x-icon")      || mime_is(&mime, "image", "vnd.microsoft.icon"),
                "HTML"     => mime_is  (&mime, "text",        "html"),
                "JPEG"     => mime_is  (&mime, "image",       "jpeg"),
                "JS"       => mime_is  (&mime, "text",        "javascript")  || mime_is(&mime, "application", "javascript"),
                "JSON"     => mime_is  (&mime, "application", "json"),
                "JSONFEED" => mime_is_p(&mime, "application", "feed", "json"),
                "OTF"      => mime_is  (&mime, "font",        "opentype"),
                "PDF"      => mime_is  (&mime, "application", "pdf"),
                "PNG"      => mime_is  (&mime, "image",       "png"),
                "SVG"      => mime_is_p(&mime, "image",       "svg", "xml"),
                "TTF"      => mime_is  (&mime, "font",        "ttf"),
                "TXT"      => mime_is  (&mime, "text",        "plain"),
                "WEBP"     => mime_is  (&mime, "image",       "webp"),
                "WOFF"     => mime_is  (&mime, "font",        "woff")        || mime_is(&mime, "application", "font-woff"),
                "WOFF2"    => mime_is  (&mime, "font",        "woff2")       || mime_is(&mime, "application", "font-woff2"),
                "XML"      => mime_is  (&mime, "text",        "xml")         || mime_is(&mime, "application", "xml"),
                "ZIP"      => mime_is  (&mime, "application", "zip"),
                _          => unreachable!()
            };

            if matches {
                CheckResult::Passed(Pass::ContentTypeMatch)
            }
            else {
                CheckResult::Failed(Fail::ContentTypeMismatch(actual.into()))
            }
        }
        else if let ContentTypeCheck::MimeType(mime) = check {
            let actual = rs.content_type();

            if actual == Some(mime) {
                CheckResult::Passed(Pass::ContentTypeMatch)
            }
            else if let Some(actual) = actual {
                CheckResult::Failed(Fail::ContentTypeMismatch(actual.into()))
            }
            else {
                CheckResult::Failed(Fail::ContentTypeMissing)
            }
        }
        else {
            unreachable!()
        }
    }

    /// The check result for redirects and the `Location` header.
    fn redirect_result(&self, expected_location: &str, got_location: Option<&str>, got_status: i32) -> CheckResult<Pass, Fail> {
        if got_status < 300 && got_status > 303 {
            CheckResult::Failed(Fail::StatusMismatch(got_status))
        }
        else if let Some(got) = got_location {
            if got == expected_location {
                CheckResult::Passed(Pass::RedirectMatch)
            }
            else {
                CheckResult::Failed(Fail::RedirectMismatch(String::from(got)))
            }
        }
        else {
            CheckResult::Failed(Fail::RedirectMissing)
        }
    }

    /// The check result for the `Content-Encoding` header.
    fn encoding_result(&self, rs: &impl HttpResponse, encoding: &str) -> CheckResult<Pass, Fail> {
        if let Some(actual) = rs.encoding() {
            if *actual == *encoding {
                CheckResult::Passed(Pass::EncodingMatch)
            }
            else {
                CheckResult::Failed(Fail::EncodingMismatch(actual.into()))
            }
        }
        else {
            CheckResult::Failed(Fail::EncodingMissing)
        }
    }

    fn body_result(&self, body: &[u8], body_matcher: &ContentsMatcher) -> CheckResult<Pass, Fail> {
        match body_matcher.check(&body) {
            CheckResult::Passed(pass) => {
                CheckResult::Passed(Pass::ContentsPass(pass))
            }
            CheckResult::Failed(fail) => {
                CheckResult::Failed(Fail::ContentsFail(fail))
            }
            CheckResult::CommandError(_) => {
                unreachable!()
            }
        }
    }
}

fn mime_is(mime: &Mime, one: &str, two: &str) -> bool {
    mime.type_() == one && mime.subtype() == two && mime.suffix().is_none()
}

fn mime_is_p(mime: &Mime, one: &str, two: &str, three: &str) -> bool {
    mime.type_() == one && mime.subtype() == two && mime.suffix().map(|e| e.as_str()) == Some(three)
}

/// The successful result of an HTTP check.
#[derive(PartialEq, Debug)]
pub enum Pass {

    /// We were able to make a successful HTTP call.
    HttpSucceeded,

    /// The HTTP status was the expected number.
    StatusMatch,

    /// The `Content-Type` header matches.
    ContentTypeMatch,

    /// The status is a redirect and the `Location` header matches.
    RedirectMatch,

    /// The `Server` header matches.
    ServerMatch,

    /// The `Encoding` header matches.
    EncodingMatch,

    /// Another header matches.
    HeaderMatch(String),

    /// The body matches its contents predicate.
    ContentsPass(contents::Pass),
}

/// The failure result of running an HTTP check.
#[derive(Debug)]
pub enum Fail {

    /// We were not able to make an HTTP call.
    HttpFailed,

    /// The HTTP status was not the expected number; instead, it was this.
    StatusMismatch(i32),

    /// The `Content-Type` header was this.
    ContentTypeMismatch(String),

    /// The `Content-Type` header was missing.
    ContentTypeMissing,

    /// The `Content-Type` header could not be parsed into a MIME type.
    InvalidMimeType(String),

    /// The `Location` header did not have the expected contents; instead, it
    /// has this.
    RedirectMismatch(String),

    /// The `Location` header was missing.
    RedirectMissing,

    /// The `Server` header was this.
    ServerMismatch(String),

    /// The `Content-Encoding` was not the expected value; instead, it was this.
    EncodingMismatch(String),

    /// The `Content-Encoding` header was missing.
    EncodingMissing,

    /// Another header had an unexpected value.
    HeaderMismatch(String, String),

    /// Another header is missing.
    HeaderMissing(String),

    /// The body did not match its contents predicate.
    ContentsFail(contents::Fail),
}

impl PassResult for Pass {}

impl FailResult for Fail {
    fn command_output(&self) -> Option<(String, &String)> {
        match self {
            Self::ContentsFail(fail)  => fail.command_output("Response body:"),
            _                         => None,
        }
    }

    fn diff_output(&self) -> Option<(String, &String, &String)> {
        match self {
            Self::ContentsFail(fail)  => fail.diff_output(),
            _                         => None,
        }
    }
}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpSucceeded => {
                write!(f, "HTTP connection succeeded")
            }
            Self::StatusMatch => {
                write!(f, "HTTP status matches")
            }
            Self::ContentTypeMatch => {
                write!(f, "Content-Type matches")
            }
            Self::RedirectMatch => {
                write!(f, "Location header matches")
            }
            Self::ServerMatch => {
                write!(f, "Server header matches")
            }
            Self::EncodingMatch => {
                write!(f, "Content-Encoding header matches")
            }
            Self::HeaderMatch(header) => {
                write!(f, "HTTP header ‘{}’ matches", header)
            }
            Self::ContentsPass(contents_pass) => {
                contents_pass.fmt(f)
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpFailed => {
                write!(f, "HTTP connection failed")
            }
            Self::StatusMismatch(stat) => {
                write!(f, "HTTP status is ‘{}’", stat)
            }
            Self::ContentTypeMismatch(ct) => {
                write!(f, "Content-Type is ‘{}’", ct)
            }
            Self::ContentTypeMissing => {
                write!(f, "Content-Type header is missing")
            }
            Self::InvalidMimeType(ct) => {
                write!(f, "Content-Type ‘{}’ is not a valid MIME type", ct)
            }
            Self::RedirectMismatch(loc) => {
                write!(f, "Location header is ‘{}’", loc)
            }
            Self::RedirectMissing => {
                write!(f, "Location header is missing")
            }
            Self::ServerMismatch(srv) => {
                write!(f, "Server header is ‘{}’", srv)
            }
            Self::EncodingMismatch(ce) => {
                write!(f, "Content-Encoding header is ‘{}’", ce)
            }
            Self::EncodingMissing => {
                write!(f, "Content-Encoding header is missing")
            }
            Self::HeaderMismatch(header, got) => {
                write!(f, "HTTP header ‘{}’ was ‘{}’", header, got)
            }
            Self::HeaderMissing(header) => {
                write!(f, "HTTP header ‘{}’ was missing", header)
            }
            Self::ContentsFail(contents_fail) => {
                contents_fail.fmt(f)
            }
        }
    }
}
