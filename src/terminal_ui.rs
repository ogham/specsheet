use ansi_term::{Style, Colour::*};
use once_cell::sync::Lazy;
use regex::Regex;

use spec_checks::{Check, CheckResult, PassResult, FailResult};

use crate::input::{InputSource, LoadError};
use crate::results::Stats;
use crate::set::ReadError;


/// The **terminal UI** handles printing stuff to the screen as
/// specsheet executes.
#[derive(PartialEq, Debug)]
pub struct TerminalUI {
    pub colours: Colours,
    pub shown_lines: ShownLines,
}


/// How progress should be presented.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct ShownLines {
    pub successes: ExpandLevel,
    pub failures:  ExpandLevel,
    pub summaries: ExpandLevel,
}

/// Whether to show individual Pass/Fail results in the output.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ExpandLevel {

    /// Don't show anything, ever.
    Hide,

    /// Show the check, but none of its check results.
    Show,

    /// Show the check, and expand its check results.
    Expanded,
}

impl TerminalUI {

    /// Print a new section based on the path to the file of checks
    /// being run. This gets executed at the start of each file.
    pub fn print_file_section(&self, input_stream: &InputSource) {
        println!("\n   {}", self.colours.file_heading.paint(&input_stream.to_string()));
    }

    /// Prints an errors that occurred while loading a file of checks.
    pub fn print_load_error(&self, input: &InputSource, e: LoadError) {
        match e {
            // For the first two, show the path so the terminal can
            // linkify it. Also it makes it seem more “official”.
            LoadError::Io(ioe) => {
                println!(" {} {} {}: {}", self.colours.question_sub.paint("?"), self.colours.error.paint("error:"), input, ioe);
            }
            LoadError::Toml(te) => {
                if let Some((line, col)) = te.line_col() {
                    println!(" {} {} {}:{}:{}: {}", self.colours.question_sub.paint("?"), self.colours.error.paint("parse error:"), input, line + 1, col, te);
                }
                else {
                    println!(" {} {} {}: {}", self.colours.question_sub.paint("?"), self.colours.error.paint("parse error:"), input, te);
                }
            }
        }
    }

    pub fn print_read_errors(&self, es: &[ReadError]) {
        // We don’t need to show the path here. Read errors are the most
        // common type of error a user will encounter, and they’re printed at
        // the top of the section so the path is right there, and we don’t
        // have a line/column number.

        for err in es {
            println!(" {} {} {} {}", self.colours.question_sub.paint("?"), self.colours.error.paint("read error:"), self.colours.question_sub.paint(&format!("[{}]", err.name)), err.inner);
        }
    }

    /// Print an individual check and its results to the screen. This
    /// gets executed after a check has been run.
    pub fn print_check(&self, check: &impl Check, name: Option<&String>, results: &[CheckResult<impl PassResult, impl FailResult>]) {

        // Make text in ‘single smart quotes’ bold for the terminal
        let check = name.cloned().unwrap_or_else(|| check.to_string());
        let check = SMART_QUOTES.replace_all(&check, "\x1B[1m$1\x1b[0m");

        let passed = results.iter().all(CheckResult::passed);

        if passed {
            if self.shown_lines.successes == ExpandLevel::Hide {
                return;
            }

            println!(" {} {}", self.colours.tick.paint("✔"), check);
        }
        else {
            if self.shown_lines.failures == ExpandLevel::Hide {
                return;
            }

            println!(" {} {}", self.colours.cross.paint("✘"), check);
        }

        for result in results {
            if passed {
                if self.shown_lines.successes == ExpandLevel::Expanded {
                    self.print_result(&result);
                    self.print_output(&result);
                }
            }
            else {
                if self.shown_lines.failures == ExpandLevel::Expanded {
                    self.print_result(&result);
                    self.print_output(&result);
                }
            }
        }
    }

    /// Prints the number of successes and failures to the screen.
    /// This gets called after a file of checks has been run, and
    /// their totals tallied up.
    pub fn print_stats(&self, stats: Stats) {
        let successes = stats.pass_count;
        let failed = stats.fail_count;

        let total = successes + failed;

        if self.shown_lines.summaries != ExpandLevel::Hide {
            if total == 0 {
                println!("   {}", self.colours.zero.paint(format!("{}/{} successful", successes, total)))
            }
            else if failed == 0 {
                println!("   {}/{} successful", successes, total)
            }
            else {
                println!("   {}", self.colours.cross.paint(format!("{}/{} successful", successes, total)))
            }
        }
    }
}

impl TerminalUI {

    /// Prints an individual result to the screen. This gets executed
    /// when the type of result has the `Extended` level.
    fn print_result(&self, result: &CheckResult<impl PassResult, impl FailResult>) {
        match result {
            CheckResult::Passed(pass) => {
                println!("   {} {}", self.colours.tick_sub.paint("✔"), pass);
            }

            CheckResult::Failed(fail) => {
                println!("   {} {}", self.colours.cross_sub.paint("✘"), fail);
            }

            CheckResult::CommandError(err) => {
                println!("   {} {}", self.colours.question_sub.paint("?"), err);
            }
        }
    }

    fn print_output(&self, result: &CheckResult<impl PassResult, impl FailResult>) {
        match result {
            CheckResult::Passed(pass) => {
                if let Some((title, string)) = pass.command_output() {
                    println!("     {}", self.colours.output_heading.paint(title));

                    for line in string.lines() {
                        println!("     {}", line.escape_default());
                    }
                }
            }

            CheckResult::Failed(fail) => {
                if let Some((title, string)) = fail.command_output() {
                    println!("     {}", self.colours.output_heading.paint(title));

                    for line in string.lines() {
                        println!("     {}", line.escape_default());
                    }
                }
                else if let Some((title, expected, got)) = fail.diff_output() {
                    use diff::Result;

                    println!("     {}", self.colours.output_heading.paint(title));
                    for line in diff::lines(got, expected) {
                        match line {
                            Result::Left(left)   => println!("    +{}", self.colours.diff_addition.paint(&left.escape_default().collect::<String>())),
                            Result::Right(right) => println!("    -{}", self.colours.diff_removal.paint(&right.escape_default().collect::<String>())),
                            Result::Both(a, _)   => println!("     {}", a.escape_default()),
                        }
                    }
                }
            }

            CheckResult::CommandError(_err) => {
                // No diff for command errors
            }
        }
    }
}


/// A regex that detects text within ‘single smart quotes’.
static SMART_QUOTES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(‘.*?’)").unwrap()
});


/// The **colours** are used to paint the output.
#[derive(PartialEq, Debug, Default)]
pub struct Colours {

    /// The style used for outer check ticks (✔)
    pub tick: Style,

    /// The style used for inner result ticks (✔)
    pub tick_sub: Style,

    /// The style used for outer check crosses (✘)
    pub cross: Style,

    /// The style used for inner result crosses (✘)
    pub cross_sub: Style,

    /// The style used for outer file read errors (?)
    pub question: Style,

    /// The style used for inner file command failures (?)
    pub question_sub: Style,

    /// The style used for file headings
    pub file_heading: Style,

    /// The style used for highlighting the word “error”
    pub error: Style,

    pub zero: Style,

    pub output_heading: Style,
    pub diff_addition: Style,
    pub diff_removal: Style,
}

impl Colours {

    /// Create a new colour palette that has a variety of different styles
    /// defined. This is used by default.
    pub fn pretty() -> Self {
        Self {
            tick:            Green.bold(),
            tick_sub:        Green.normal(),
            cross:           Red.bold(),
            cross_sub:       Red.normal(),
            question:        Cyan.bold(),
            question_sub:    Cyan.normal(),
            file_heading:    Fixed(248).underline(),
            error:           Red.bold(),
            zero:            Yellow.bold(),
            output_heading:  Fixed(187).underline(),
            diff_addition:   Green.normal(),
            diff_removal:    Red.normal(),
        }
    }

    /// Create a new colour palette where no styles are defined, causing
    /// output to be rendered as plain text without any formatting.
    /// This is used when output is not to a terminal.
    pub fn plain() -> Self {
        Self::default()
    }
}
