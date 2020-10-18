//! Command-line option parsing.

use std::ffi::OsStr;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use log::*;

use spec_checks::read::{Rewrites, Rewrite};

use crate::commands::GlobalOptions;
use crate::doc::DocumentPaths;
use crate::filter::{Filter, TagsFilter, TypesFilter, RunningOrder};
use crate::input::Inputs;
use crate::output::{OutputFormat, UseColours};
use crate::side::{SideProcess, StartupWait, KillSignal};
use crate::terminal_ui::{ShownLines, ExpandLevel};


/// The **options** contains the entirety of the parsed user input from the
/// command-line and the environment variables.
#[derive(PartialEq, Debug)]
pub struct Options {
    pub mode: RunningMode,
    pub output: OutputFormat,
    pub inputs: Inputs,
    pub filter: Filter,
    pub rewrites: Rewrites,
}

/// Specsheet runs in a **mode**, which determines how much it does.
#[derive(PartialEq, Debug)]
pub enum RunningMode {

    /// Run each file of tests completely.
    Run(CheckingOptions, EndingOptions),

    /// Run in continual mode.
    Continual(CheckingOptions),

    /// Don’t run any checks, just validate each input file’s syntax.
    SyntaxCheckOnly,

    /// Don’t run any checks, just list the commands that would have been
    /// executed.
    ListCommandsOnly(GlobalOptions),

    /// Don’t run any checks, just list the ones that would have been ran.
    ListChecksOnly,

    /// Don’t run any checks, just list the tags defined in the documets.
    ListTagsOnly,
}

/// Options for running checks, which are used in both normal and continual mode.
#[derive(PartialEq, Debug)]
pub struct CheckingOptions {
    pub delay: Delay,
    pub global_options: GlobalOptions,
    pub directory: RunningDirectory,
    pub process: Option<SideProcess>,
}

/// Options for what to do after all the checks have been run, which is only
/// used in normal mode.
#[derive(PartialEq, Debug)]
pub struct EndingOptions {
    pub perform_analysis: bool,
    pub result_documents: DocumentPaths,
}

/// The **delay** determines how long to wait between running two checks.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Delay {

    /// Sleep for the given delay between two checks.
    Wait(Duration),

    /// Run the second check immediately after the first.
    RunInstantly,
}

/// Which directory checks should be run from.
#[derive(PartialEq, Debug)]
pub enum RunningDirectory {

    /// Run commands from the directory the input check file is in. This
    /// changes directory on every input file, and resets it at the end.
    CheckDirectory,

    /// Run commands from a specific directory. This changes directory once,
    /// for the duration of the program.
    OtherDirectory(PathBuf),
}


impl Options {

    /// Parses and interprets a set of options from the user’s command-line
    /// arguments.
    ///
    /// This returns an `Ok` set of options if successful and running
    /// normally, a `Help` or `Version` variant if one of those options is
    /// specified, or an error variant if there’s an invalid option or
    /// inconsistency within the options after they were parsed.
    #[allow(unused_results)]
    pub fn getopts<C>(args: C) -> OptionsResult
    where C: IntoIterator,
          C::Item: AsRef<OsStr>,
    {
        let mut opts = getopts::Options::new();

        // Meta options
        opts.optflag ("v", "version",          "show version of specsheet");
        opts.optflag ("?", "help",             "show list of command-line options");

        // Running modes
        opts.optflag ("c", "syntax-check",     "don't run, just check the syntax of the input files");
        opts.optflag ("C", "list-commands",    "don't run, just list the commands that would be executed");
        opts.optflag ("l", "list-checks",      "don't run, just list the checks that would be run");
        opts.optflag (" ", "list-tags",        "don't run, just list the tags defined in the documents");
        opts.optflag ("",  "random-order",     "run the checks in a random order");
        opts.optflag ("",  "continual",        "run the checks in continual mode");
        opts.optopt  ("",  "delay",            "amount of time to delay between checks", "DURATION");
        opts.optopt  ("",  "directory",        "directory to run the tests from", "PATH");
        opts.optopt  ("j", "threads",          "number of threads to run in parallel", "COUNT");
        opts.optmulti("O", "option",           "set a global option or override the environment", "KEY=VALUE");
        opts.optmulti("R", "rewrite",          "add a rule to rewrite values in the input documents", "THIS->THAT");
        opts.optflag ("z", "analysis",         "switch on analysis");

        // Background process options
        opts.optmulti("x", "exec",             "process to run in the background during execution", "CMD");
        opts.optopt  ("",  "exec-delay",       "wait an amount of time before running checks", "DURATION");
        opts.optopt  ("",  "exec-port",        "wait until a port becomes open before running checks", "PORT");
        opts.optopt  ("",  "exec-file",        "wait until a file exists before running checks", "PATH");
        opts.optopt  ("",  "exec-line",        "wait until the process outputs a line before running checks", "REGEX");
        opts.optopt  ("",  "exec-kill-signal", "signal to send to the background process after finishing", "SIGNAL");

        // Filtering options
        opts.optopt  ("t", "tags",             "comma-separated list of tags to run", "TAGS");
        opts.optopt  ("",  "skip-tags",        "comma-separated list of tags to skip", "TAGS");
        opts.optopt  ("T", "types",            "comma-separated list of check types to run", "TYPES");
        opts.optopt  ("",  "skip-types",       "comma-separated list of check types to skip", "TYPES");

        // Output options
        opts.optopt  ("s", "successes",        "how to show successful results", "SHOW");
        opts.optopt  ("f", "failures",         "how to show unsuccessful results", "SHOW");
        opts.optopt  ("",  "summaries",        "how to show summaries for each file", "SHOW");
        opts.optopt  ("P", "print",            "how to print the output", "FORMAT");
        opts.optopt  ("",  "color",            "when to use terminal colors",  "WHEN");
        opts.optopt  ("",  "colour",           "when to use terminal colours", "WHEN");

        // Results document options
        opts.optopt  ("",  "html-doc",         "produce an output HTML document", "PATH");
        opts.optopt  ("",  "json-doc",         "produce an output JSON document", "PATH");
        opts.optopt  ("",  "toml-doc",         "produce an output TOML document", "PATH");

        let matches = match opts.parse(args) {
            Ok(m)  => m,
            Err(e) => return OptionsResult::InvalidOptionsFormat(e),
        };

        if matches.opt_present("version") {
            OptionsResult::Version(UseColours::deduce(&matches))
        }
        else if let Some(reason) = Self::check_help(&matches) {
            OptionsResult::Help(reason, UseColours::deduce(&matches))
        }
        else {
            match Self::deduce(&matches) {
                Ok(opts) => OptionsResult::Ok(opts),
                Err(e)   => OptionsResult::InvalidOptions(e),
            }
        }
    }

    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        let mode = RunningMode::deduce(matches)?;
        let output = OutputFormat::deduce(matches)?;
        let inputs = Inputs::deduce(matches)?;
        let filter = Filter::deduce(matches);
        let rewrites = parse_rewrites(matches)?;

        Ok(Self { mode, output, inputs, filter, rewrites })
    }

    /// Check whether the given set of matches require the help text to be
    /// printed; if so, returns the reason, and if not, returns nothing.
    fn check_help(matches: &getopts::Matches) -> Option<HelpReason> {
        if matches.opt_present("help") {
            Some(HelpReason::Flag)
        }
        else if matches.free.is_empty() {
            Some(HelpReason::NoArguments)
        }
        else {
            None
        }
    }
}


impl RunningMode {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if matches.opt_present("syntax-check") {
            Ok(Self::SyntaxCheckOnly)
        }
        else if matches.opt_present("list-commands") {
            let global_options = GlobalOptions::deduce(matches)?;
            Ok(Self::ListCommandsOnly(global_options))
        }
        else if matches.opt_present("list-checks") {
            Ok(Self::ListChecksOnly)
        }
        else if matches.opt_present("list-tags") {
            Ok(Self::ListTagsOnly)
        }
        else if matches.opt_present("continual") {
            let check_opts = CheckingOptions::deduce(matches)?;
            Ok(Self::Continual(check_opts))
        }
        else {
            let check_opts = CheckingOptions::deduce(matches)?;
            let end_opts   = EndingOptions::deduce(matches)?;
            Ok(Self::Run(check_opts, end_opts))
        }
    }
}


impl CheckingOptions {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        let delay = Delay::deduce(matches)?;
        let global_options = GlobalOptions::deduce(matches)?;
        let directory = RunningDirectory::deduce(matches);
        let process = SideProcess::deduce(matches);
        Ok(Self { delay, global_options, directory, process })
    }
}


impl Delay {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if let Some(duration) = matches.opt_str("delay") {
            let d = parse_delay(&duration)?;
            Ok(Self::Wait(d))
        }
        else {
            Ok(Self::RunInstantly)
        }
    }
}


impl OutputFormat {
    pub fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if let Some(format) = matches.opt_str("print") {
            Ok(match &*format {
                "ansi"       => Self::Text(UseColours::deduce(matches), ShownLines::deduce(matches)?),
                "dots"       => Self::Dots,
                "json-lines" => Self::JsonLines,
                "tap"        => Self::TAP,
                _            => return Err(OptionsError::InvalidOutputFormat(format.clone())),
            })
        }
        else {
            Ok(Self::Text(UseColours::deduce(matches), ShownLines::deduce(matches)?))
        }
    }
}


impl ShownLines {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        let successes = ExpandLevel::deduce(matches, "successes")?.unwrap_or(ExpandLevel::Show);
        let failures  = ExpandLevel::deduce(matches, "failures")?.unwrap_or(ExpandLevel::Expanded);
        let summaries = ExpandLevel::deduce(matches, "summaries")?.unwrap_or(ExpandLevel::Show);
        Ok(Self { successes, failures, summaries })
    }
}


impl ExpandLevel {
    fn deduce(matches: &getopts::Matches, key: &'static str) -> Result<Option<Self>, OptionsError> {
        if let Some(option) = matches.opt_str(key) {
            Ok(Some(match &*option {
                "hide"   | "hidden"    => Self::Hide,
                "show"   | "shown"     => Self::Show,
                "expand" | "expanded"  => Self::Expanded,
                _                      => return Err(OptionsError::InvalidExpandLevel(option.clone()))
            }))
        }
        else {
            Ok(None)
        }
    }
}


impl UseColours {
    pub fn deduce(matches: &getopts::Matches) -> Self {
        match matches.opt_str("color").or_else(|| matches.opt_str("colour")).unwrap_or_default().as_str() {
            "automatic" | "auto" | ""  => Self::Automatic,
            "always"    | "yes"        => Self::Always,
            "never"     | "no"         => Self::Never,
            otherwise => {
                warn!("Unknown colour setting {:?}", otherwise);
                Self::Automatic
            },
        }
    }
}


impl Inputs {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if matches.free.is_empty() {
            unreachable!()  // dealt with in check_help
        }
        else if matches.free[0] == "-" {
            Ok(Self::Stdin)
        }
        else {
            let paths = matches.free.iter().map(PathBuf::from).collect();
            Ok(Self::Files(paths))
        }
    }
}


impl Filter {
    fn deduce(matches: &getopts::Matches) -> Self {
        Self {
            tags: TagsFilter::deduce(matches),
            types: TypesFilter::deduce(matches),
            order: RunningOrder::deduce(matches),
        }
    }
}


impl TagsFilter {
    fn deduce(matches: &getopts::Matches) -> Self {
        let mut tf = Self::default();

        if let Some(tags) = matches.opt_str("tags") {
            tf.tags.extend(tags.split(',').map(String::from))
        }

        if let Some(skip_tags) = matches.opt_str("skip-tags") {
            tf.skip_tags.extend(skip_tags.split(',').map(String::from))
        }

        tf
    }
}


impl TypesFilter {
    fn deduce(matches: &getopts::Matches) -> Self {
        let mut tf = Self::default();

        if let Some(types) = matches.opt_str("types") {
            tf.types.extend(types.split(',').map(String::from))
        }

        if let Some(skip_types) = matches.opt_str("skip-types") {
            tf.skip_types.extend(skip_types.split(',').map(String::from))
        }

        tf
    }
}


impl RunningOrder {
    fn deduce(matches: &getopts::Matches) -> Self {
        if matches.opt_present("random-order") {
            Self::Random
        }
        else {
            Self::ByType
        }
    }
}


impl GlobalOptions {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        use std::collections::btree_map::BTreeMap;

        let mut map = BTreeMap::new();

        for input in matches.opt_strs("option") {
            let equals_index = match input.find('=') {
                Some(ei)  => ei,
                None      => return Err(OptionsError::InvalidGlobalSyntax(input)),
            };

            let key = input[.. equals_index].into();
            let val = input[equals_index + 1 ..].into();

            if map.contains_key(&key) {
                return Err(OptionsError::DuplicateGlobal(key));
            }
            else {
                map.insert(key, val);
            }
        }

        Ok(Self { map })
    }
}


impl RunningDirectory {
    fn deduce(matches: &getopts::Matches) -> Self {
        if let Some(directory) = matches.opt_str("directory") {
            Self::OtherDirectory(PathBuf::from(directory))
        }
        else {
            Self::CheckDirectory
        }
    }
}


impl SideProcess {
    fn deduce(matches: &getopts::Matches) -> Option<Self> {
        if let Some(shell) = matches.opt_str("exec") {
            let wait = StartupWait::deduce(matches).ok()?;
            let signal = KillSignal::deduce(matches).ok()?;
            Some(Self { shell, wait, signal })
        }
        else {
            None
        }
    }
}


impl StartupWait {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        // TODO: some way to have more than one of these apply at once
        if let Some(delay) = matches.opt_str("exec-delay") {
            let duration = parse_delay(&delay)?;
            Ok(Self::Delay(duration))
        }
        else if let Some(port) = matches.opt_str("exec-port") {
            let port_number = port.parse().map_err(|e| OptionsError::InvalidPortNumber(e, port))?;
            Ok(Self::Port(port_number))
        }
        else if let Some(path) = matches.opt_str("exec-file") {
            let path = PathBuf::from(path);
            Ok(Self::File(path))
        }
        else if let Some(regex) = matches.opt_str("exec-line") {
            // TODO: some way to check for invalid regexes early
            Ok(Self::OutputLine(regex))
        }
        else {
            Ok(Self::default())
        }
    }
}


fn parse_delay(input: &str) -> Result<Duration, OptionsError> {
    match input.parse() {
        Ok(seconds) => {
            Ok(Duration::new(seconds, 0))
        }
        Err(e) => {
            warn!("Invalid delay duration: {}", e);
            Err(OptionsError::InvalidDelay(input.into()))
        }
    }
}


impl KillSignal {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        if let Some(signal_name) = matches.opt_str("exec-kill-signal") {
            match &*signal_name {
                "int"  | "sigint"  | "2"  => Ok(Self::Int),
                "kill" | "sigkill" | "9"  => Ok(Self::Kill),
                "term" | "sigterm" | "15" => Ok(Self::Term),
                _                         => Err(OptionsError::InvalidKillSignal(signal_name)),
            }
        }
        else {
            Ok(Self::default())
        }
    }
}


impl EndingOptions {
    fn deduce(matches: &getopts::Matches) -> Result<Self, OptionsError> {
        let perform_analysis = matches.opt_present("analysis");
        let result_documents = DocumentPaths::deduce(matches);
        Ok(Self { perform_analysis, result_documents })
    }
}


impl DocumentPaths {
    fn deduce(matches: &getopts::Matches) -> Self {
        Self {
            html_path: matches.opt_str("html-doc").map(PathBuf::from),
            json_path: matches.opt_str("json-doc").map(PathBuf::from),
            toml_path: matches.opt_str("toml-doc").map(PathBuf::from),
        }
    }
}


fn parse_rewrites(matches: &getopts::Matches) -> Result<Rewrites, OptionsError> {
    let mut rewrites = Rewrites::new();

    for rewrite_rule in matches.opt_strs("rewrite") {
        let pos = match rewrite_rule.find("->") {
            Some(p)  => p,
            None     => return Err(OptionsError::InvalidRewriteRule(rewrite_rule)),
        };

        let this = &rewrite_rule[.. pos];
        let that = &rewrite_rule[pos + 2 ..];

        if this.starts_with("http://") || this.starts_with("https://") {
            rewrites.add(Rewrite::Url(this.into(), that.into()));
        }
        else if this.starts_with('/') {
            rewrites.add(Rewrite::Path(this.into(), that.into()));
        }
        else if this.starts_with('%') && that.starts_with('%') {
            rewrites.add(Rewrite::Interface(this.into(), that.into()));
        }
        else {
            return Err(OptionsError::InvalidRewriteRule(rewrite_rule));
        }
    }

    rewrites.expand_tildes();

    Ok(rewrites)
}


/// The result of the `Options::getopts` function.
#[derive(PartialEq, Debug)]
pub enum OptionsResult {

    /// The options were parsed successfully.
    Ok(Options),

    /// There was an error (from `getopts`) parsing the arguments.
    InvalidOptionsFormat(getopts::Fail),

    /// There was an error with the combination of options the user selected.
    InvalidOptions(OptionsError),

    /// Can’t run any checks because there’s help to display!
    Help(HelpReason, UseColours),

    /// One of the arguments was `--version`, to display the version number.
    Version(UseColours),
}

/// Something wrong with the combination of options the user has picked.
#[derive(PartialEq, Debug)]
pub enum OptionsError {

    /// The `--exec-kill-signal` argument was invalid.
    InvalidKillSignal(String),

    /// The `--exec-port` argument was invalid.
    InvalidPortNumber(std::num::ParseIntError, String),

    /// The `--delay` argument was an invalid duration.
    InvalidDelay(String),

    /// The syntax for a global option was invalid.
    InvalidGlobalSyntax(String),

    /// A global option was specified more than once.
    DuplicateGlobal(String),

    /// The `--print` argument was invalid.
    InvalidOutputFormat(String),

    /// The `--successes` or `--failures` argument was invalid.
    InvalidExpandLevel(String),

    /// A `--rewrite` rule was invalid.
    InvalidRewriteRule(String),
}

/// The reason that help is being displayed. If it’s for the `--help` flag,
/// then we shouldn’t return an error exit status.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum HelpReason {

    /// Help was requested with the `--help` flag.
    Flag,

    /// There were no files to run, so display help instead.
    NoArguments,
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKillSignal(ks)        => write!(f, "Invalid kill signal {:?}", ks),
            Self::InvalidPortNumber(err, num)  => write!(f, "Invalid port number {:?}: {}", num, err),
            Self::InvalidDelay(del)            => write!(f, "Invalid delay {:?}", del),
            Self::InvalidGlobalSyntax(arg)     => write!(f, "Invalid global option syntax for {:?}", arg),
            Self::DuplicateGlobal(name)        => write!(f, "Global option {:?} was specified twice", name),
            Self::InvalidExpandLevel(arg)      => write!(f, "Invalid expand level {:?}", arg),
            Self::InvalidOutputFormat(arg)     => write!(f, "Invalid output format {:?}", arg),
            Self::InvalidRewriteRule(arg )     => write!(f, "Invalid rewrite rule {:?}", arg),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn getopts(checks: &[&str]) -> bool {
        let opts = Options::getopts(checks);
        println!("Options: {:?}", opts);

        if let OptionsResult::Ok(_opts) = opts {
            true
        }
        else {
            false
        }
    }

    #[test]
    fn help() {
        assert_eq!(false, getopts(&[ "--help" ]));
    }

    #[test]
    fn version() {
        assert_eq!(false, getopts(&[ "--version" ]));
    }

    #[test]
    fn check() {
        assert_eq!(true, getopts(&[ "check.toml" ]));
    }

    #[test]
    fn delay_ok() {
        assert_eq!(true, getopts(&[ "checks.toml", "--delay=10" ]));
    }

    #[test]
    fn delay_not() {
        assert_eq!(false, getopts(&[ "checks.toml", "--delay=x" ]));
    }

    #[test]
    fn curl_option_ok() {
        assert_eq!(true, getopts(&[ "checks.toml", "-O", "http.localhost=8991" ]));
    }

    #[test]
    fn output_format_ok() {
        assert_eq!(true, getopts(&[ "checks.toml", "-P", "json-lines" ]));
    }

    #[test]
    fn output_format_not() {
        assert_eq!(false, getopts(&[ "checks.toml", "-P", "yaml-0bj3ctz" ]));
    }

    #[test]
    fn expand_level_ok() {
        assert_eq!(true, getopts(&[ "checks.toml", "-s", "expand" ]));
    }

    #[test]
    fn expand_level_not() {
        assert_eq!(false, getopts(&[ "checks.toml", "-s", "random" ]));
    }
}
