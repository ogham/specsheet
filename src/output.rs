use serde_json::json;

use spec_checks::{Check, CheckResult, PassResult, FailResult};

use crate::input::{InputSource, LoadError};
use crate::results::Stats;
use crate::set::ReadError;
use crate::terminal_ui::{TerminalUI, Colours, ShownLines};


/// How to format the output data.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum OutputFormat {

    /// Format the output as plain text, optionally adding ANSI colours.
    Text(UseColours, ShownLines),

    // Print a dot per check.
    Dots,

    /// Format the entries as JSON Lines.
    JsonLines,

    /// Format the output as TAP (Test Anything Protocol).
    TAP,
}


impl OutputFormat {
    pub fn ui(self) -> Output {
        match self {
            Self::Text(uc, sl) => {
                let tui = TerminalUI { colours: uc.palette(), shown_lines: sl };
                Output::Text(tui)
            }
            Self::Dots => {
                Output::Dots
            }
            Self::JsonLines => {
                Output::JSON
            }
            Self::TAP => {
                Output::TAP { count: 0 }
            }
        }
    }
}


/// An output, which prints lines, errors, and check results.
/// It would be a trait, but I can’t make it a trait because of some weird
/// Rust reason I don’t really understand (the language got in my way).
pub enum Output {
    Text(TerminalUI),
    Dots,
    JSON,
    TAP { count: u32 },
}

impl Output {
    // ugh, this repetition

    pub fn print_file_section(&self, input_source: &InputSource) {
        match self {
            Self::Text(tui)   => tui.print_file_section(input_source),
            Self::Dots        => {/* do nothing */},
            Self::JSON        => json_print_file_section(input_source),
            Self::TAP { .. }  => tap_print_file_section(input_source),
        }
    }

    pub fn print_load_error(&self, input: &InputSource, e: LoadError) {
        match self {
            Self::Text(tui)   => tui.print_load_error(input, e),
            Self::Dots        => dots_print_load_error(),
            Self::JSON        => json_print_load_error(input, e),
            Self::TAP { .. }  => tap_print_load_error(),
        }
    }

    pub fn print_read_errors(&self, es: &[ReadError]) {
        match self {
            Self::Text(tui)   => tui.print_read_errors(es),
            Self::Dots        => dots_print_read_error(),
            Self::JSON        => json_print_read_error(es),
            Self::TAP { .. }  => tap_print_read_error(),
        }
    }

    pub fn print_check(&mut self, check: &impl Check, name: Option<&String>, results: &[CheckResult<impl PassResult, impl FailResult>]) {
        match self {
            Self::Text(tui)      => tui.print_check(check, name, results),
            Self::Dots           => dots_print_check(check, results),
            Self::JSON           => json_print_check(check, name, results),
            Self::TAP { count }  => tap_print_check(check, name, results, { *count += 1; *count }),
        }
    }

    pub fn print_stats(&self, stats: Stats) {
        match self {
            Self::Text(tui)   => tui.print_stats(stats),
            Self::JSON        => json_print_stats(stats),
            _                 => {/* do nothing */},
        }
    }

    pub fn print_end(&self) {
        match self {
            Self::Dots => println!(),
            _          => {/* do nothing */},
        }
    }
}


/// When to use colours in the output.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum UseColours {

    /// Always use colours.
    Always,

    /// Use colours if output is to a terminal; otherwise, do not.
    Automatic,

    /// Never use colours.
    Never,
}

impl UseColours {

    /// Whether we should use colours or not. This checks whether the user has
    /// overridden the colour setting, and if not, whether output is to a
    /// terminal.
    pub fn should_use_colours(self) -> bool {
        self == Self::Always || (atty::is(atty::Stream::Stdout) && self != Self::Never)
    }

    /// Creates a palette of colours depending on the user’s wishes or whether
    /// output is to a terminal.
    pub fn palette(self) -> Colours {
        if self.should_use_colours() {
            Colours::pretty()
        }
        else {
            Colours::plain()
        }
    }
}


// dots

fn dots_print_load_error() {
    print!("?");
}

fn dots_print_read_error() {
    print!("?");
}

fn dots_print_check(_check: &impl Check, results: &[CheckResult<impl PassResult, impl FailResult>]) {
    let passed = results.iter().all(CheckResult::passed);
    if passed {
        print!(".");
    }
    else {
        print!("X");
    }
}


// tap

fn tap_print_file_section(input_source: &InputSource) {
    println!("# {}", input_source);
}

fn tap_print_load_error() {
    println!("# Load error");
}

fn tap_print_read_error() {
    println!("# Load error");
}

fn tap_print_check(check: &impl Check, name: Option<&String>, results: &[CheckResult<impl PassResult, impl FailResult>], count: u32) {
    let name = name.cloned().unwrap_or_else(|| check.to_string());

    let passed = results.iter().all(CheckResult::passed);
    if passed {
        println!("ok {} - {}", count, name);
    }
    else {
        println!("fail {} - {}", count, name);

        for result in results {
            match result {
                CheckResult::Passed(message) => println!("  {}", message),
                CheckResult::Failed(message) => println!("  {}", message),
                CheckResult::CommandError(message) => println!("  {}", message),
            }
        }
    }
}


// json

fn json_print_file_section(input_source: &InputSource) {
    println!("{}", json!({
        "file": {
            "path": input_source.to_string(),
        }
    }));
}

fn json_print_load_error(input_source: &InputSource, e: LoadError) {
    println!("{}", json!({
        "load-error": {
            "path": input_source.to_string(),
            "error": e.to_string(),
        }
    }));
}

fn json_print_read_error(es: &[ReadError]) {
    println!("{}", json!({
        "read-error": {
            "errors": es.iter().map(|e| e.inner.to_string()).collect::<Vec<_>>(),
        }
    }));
}

fn json_print_check(check: &impl Check, name: Option<&String>, results: &[CheckResult<impl PassResult, impl FailResult>]) {
    let passed = results.iter().all(CheckResult::passed);

    let mut stages = Vec::new();
    for result in results {
        match result {
            CheckResult::Passed(pass) => {
                stages.push(json!({ "status": "pass", "message": pass.to_string() }));
            }

            CheckResult::Failed(fail) => {
                stages.push(json!({ "status": "fail", "message": fail.to_string() }));
            }

            CheckResult::CommandError(err) => {
                stages.push(json!({ "status": "error", "message": err.to_string() }));
            }
        }
    }

    println!("{}", json!({
        "ran-check": {
            "name": name.cloned().unwrap_or_else(|| check.to_string()),
            "passed": passed,
            "stages": stages,
        }
    }));
}

fn json_print_stats(stats: Stats) {
    println!("{}", json!({
        "stats": {
            "check-count": stats.check_count,
            "pass-count":  stats.pass_count,
            "fail-count":  stats.fail_count,
            "err-count":   stats.err_count,
        },
    }));
}
