//! The `curl` command
//!
//! # Sample output
//!
//! The output from `curl` should be the HTTP status, a list of headers, and
//! then, if needed, the body.
//!
//! ```text
//! $ curl --silent --head https://eu.httpbin.org/json
//! HTTP/1.1 200 OK
//! Access-Control-Allow-Credentials: true
//! Access-Control-Allow-Origin: *
//! Content-Length: 429
//! Content-Type: application/json
//! Date: Sat, 05 Oct 2019 07:02:14 GMT
//! Referrer-Policy: no-referrer-when-downgrade
//! Server: nginx
//! X-Content-Type-Options: nosniff
//! X-Frame-Options: DENY
//! X-XSS-Protection: 1; mode=block
//! Connection: keep-alive
//!
//! This is where the body would go.
//! ```

use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;

use log::*;
use once_cell::sync::Lazy;
use regex::Regex;

use spec_checks::http::{RunHttp, HttpRequest, HttpResponse};
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **curl command** that runs the `curl` binary.
#[derive(Debug, Default)]
pub struct CurlCommand {
    results: BTreeMap<HttpRequest, Exec<CurlOutput>>,
    user_agent: Option<String>,
    timeout: Option<Duration>,
}

impl CurlCommand {

    /// Creates a new command to run `curl`.
    pub fn create(global_options: &impl GlobalOptions) -> Option<Self> {
        let mut cmd = Self::default();
        cmd.timeout = global_options.duration("http.timeout");
        Some(cmd)
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().flat_map(|e| e.1.into_command())
    }
}

impl RunHttp for CurlCommand {
    type Output = CurlOutput;

    fn prime(&mut self, request: HttpRequest, print_body: bool) {
        debug!("Priming");

        if ! self.results.contains_key(&request) {
            debug!("Priming curl command with {:?}", request);
            let exec = Exec::actual(self.curl_cmd(&request, print_body));
            self.results.insert(request, exec);
        }
    }

    fn get_response(&self, executor: &mut Executor, request: HttpRequest) -> Result<Rc<CurlOutput>, Rc<ExecError>> {
        debug!("Fetching url -> {:?}", request);
        let output = self.results[&request].run(executor)?;
        Ok(output)
    }
}

impl CurlCommand {

    /// Pieces together the command to run.
    fn curl_cmd(&self, request: &HttpRequest, print_body: bool) -> Command {
        let mut cmd = Command::new("curl");
        cmd.arg("-XGET").arg("--max-time").arg("5");

        if print_body {
            cmd.arg("-i");
        }
        else {
            cmd.arg("--head");
        }

        if let Some(ua) = &self.user_agent {
            cmd.arg("--user-agent").arg(&ua);
        }
        else {
            cmd.arg("--user-agent").arg("specsheet");
        }

        for (header, value) in &request.headers {
            cmd.arg("-H").arg(format!("{}: {}", header, value));
        }

        cmd.arg(&request.url);
        cmd
    }
}


/// The **curl output** encapsulates the output lines of an
/// invoked `CurlCommand`.
#[derive(Debug)]
pub struct CurlOutput {
    first_line: Rc<str>,
    response_header_lines: Vec<Rc<str>>,
    response_body_lines: Vec<Rc<str>>,
}

impl CommandOutput for CurlOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;

        let mut iter = lines.into_iter();

        let first_line = iter.next().unwrap();

        let mut response_header_lines = Vec::new();
        while let Some(line) = iter.next() {
            if line.is_empty() {
                break;
            }
            else {
                response_header_lines.push(line);
            }
        }

        let mut response_body_lines = Vec::new();
        while let Some(line) = iter.next() {
            response_body_lines.push(line);
        }

        Ok(Self { first_line, response_header_lines, response_body_lines })
    }
}

impl HttpResponse for CurlOutput {
    fn status(&self) -> Option<i32> {
        let caps = HTTP_VERSION.captures(&self.first_line)?;
        Some(caps[1].parse().unwrap())
    }

    fn content_type(&self) -> Option<&str> {
        self.header("Content-Type")
    }

    fn encoding(&self) -> Option<&str> {
        self.header("Content-Encoding")
    }

    fn location(&self) -> Option<&str> {
        self.header("Location")
    }

    fn header(&self, header_name: &str) -> Option<&str> {
        // HTTP headers are case-insensitive:
        // https://www.w3.org/Protocols/rfc2616/rfc2616-sec4.html#sec4.2

        for line in &self.response_header_lines {
            let colon = match line.find(':') {
                Some(i) => i,
                None    => continue,
            };

            if line[.. colon].eq_ignore_ascii_case(header_name) {
                return Some(line[colon + 1 ..].trim())
            }
        }

        None
    }

    fn body(&self) -> Vec<u8> {
        let mut v = Vec::new();
        for line in &self.response_body_lines {
            v.extend(line.as_bytes());
            v.extend(b"\n");
        }
        v
    }
}

static HTTP_VERSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?x) ^ HTTP / \d \. \d \s (\d+)").unwrap()
});


// Things to parse in the curl -v output:
//
//
// # Connected flag
//
// ```
// * Connected to bsago.me (68.183.255.189) port 443 (#0)
// ```
//
// # Certificate checking
//
// ```
// * Server certificate:
// *  subject: CN=bsago.me
// *  start date: Oct 11 00:08:40 2019 GMT
// *  expire date: Jan  9 00:08:40 2020 GMT
// *  subjectAltName: host "bsago.me" matched cert's "bsago.me"
// *  issuer: C=US; O=Let's Encrypt; CN=Let's Encrypt Authority X3
// *  SSL certificate verify ok.
// ```
//
// # Request headers plus newline
//
// ```
// > GET / HTTP/1.1
// > Host: bsago.me
// > User-Agent: specsheet
// > Accept: */*
// >
// ```
//
// # Response headers plus a newline
//
// ```
// < HTTP/1.1 200 OK
// < Server: nginx
// < Date: Sun, 22 Dec 2019 21:10:47 GMT
// < Content-Type: text/html;charset=utf-8
// < Content-Length: 2715
// < Connection: keep-alive
// < Vary: Accept-Encoding
// < Cache-Control: public, max-age=300
// < Content-Security-Policy: default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self'; font-src 'self'; connect-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'; block-all-mixed-content; report-uri /csp;
// < Etag: "a79aa830ec"
// < Feature-Policy: 'none'
// < Referrer-Policy: same-origin
// < X-Content-Type-Options: nosniff
// < Strict-Transport-Security: max-age=31536000
// <
// ```
//
// # The body
