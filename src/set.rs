use std::borrow::Cow;
use std::fmt;
use std::thread::sleep;

use derive_more::{From, Display};
use log::*;

use spec_analysis::AnalysisTable;
use spec_checks::*;
use spec_checks::load::{CheckDocument, CheckEntry, Tags};
use spec_checks::read::Rewrites;
use spec_exec::Executor;

use crate::commands::Commands;
use crate::filter::{Filter, RunningOrder};
use crate::options::Delay;
use crate::output::Output;
use crate::results::{ResultsSection, ResultMessage, CheckOutput, Stats};


/// A **check set** is read from each input file.
#[derive(Debug, Default)]
pub struct CheckSet {
    checks: Vec<ReadyCheck>,
}

#[derive(Debug)]
struct ReadyCheck {
    class: LoadedCheck,
    name: Option<String>,
}

#[derive(Debug, Display, From)]
pub enum LoadedCheck {

    // command
    Cmd(cmd::CommandCheck),
    Tap(tap::TapCheck),

    // network
    Dns(dns::DnsCheck),
    Http(http::HttpCheck),
    Ping(ping::PingCheck),
    Tcp(tcp::TcpCheck),
    Udp(udp::UdpCheck),

    // remote
    Apt(apt::AptCheck),
    Defaults(defaults::DefaultsCheck),
    Fs(fs::FilesystemCheck),
    Gem(gem::GemCheck),
    Group(group::GroupCheck),
    Hash(hashes::HashCheck),
    Homebrew(homebrew::HomebrewCheck),
    HomebrewCask(homebrew_cask::HomebrewCaskCheck),
    HomebrewTap(homebrew_tap::HomebrewTapCheck),
    Npm(npm::NpmCheck),
    Systemd(systemd::SystemdCheck),
    Ufw(ufw::UfwCheck),
    User(user::UserCheck),
}


impl CheckSet {

    /// Create a new, empty check set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a file full of checks into this check set, using the filter to
    /// determine which checks to include.
    pub fn read_toml(&mut self, filter: &Filter, rewrites: &Rewrites, check_document: CheckDocument) -> Result<(), Vec<ReadError>> {

        // Work out the parent directory, because certain checks need to
        // access files relative to the file the check was in.
        //let base_directory = path.canonicalize().expect("canonicalize");
        //let base_directory = base_directory.parent().expect("parent");

        let mut errors = Vec::new();
        for (check_key, checks) in check_document {
            if ! filter.types.should_include_type(&check_key) {
                debug!("Skipping check type {}", check_key);
                continue;
            }

            for CheckEntry { inner, name, tags } in checks {
                let nothing: &[String] = &[];
                let tag_ok = match &tags {
                    Some(Tags::One(tag))    => filter.tags.should_include_tags(&[ tag ]),
                    Some(Tags::Many(tags))  => filter.tags.should_include_tags(tags),
                    None                    => filter.tags.should_include_tags(nothing),
                };

                if ! tag_ok {
                    debug!("Skipping check with tags {:?}", tags);
                    continue;
                }

                macro_rules! read_check_type {
                    ($type:path $(, $read_args:tt )*) => {
                        let type_str = <$type as Check>::TYPE;
                        if check_key == type_str {
                            debug!("Loading check {} with {:?}", type_str, inner);

                            match <$type>::read(&inner, $( $read_args )*) {
                                Ok(check) => {
                                    self.checks.push(ReadyCheck {
                                        class: LoadedCheck::from(check),
                                        name,
                                    });
                                }
                                Err(e) => {
                                    warn!("Failed to read: {:?}", e);
                                    let error = ReadError {
                                        name: type_str.into(),
                                        inner: Box::new(e),
                                    };

                                    errors.push(error);
                                }
                            }

                            continue;
                        }
                    };
                }

                // command
                read_check_type!(cmd::CommandCheck);
                read_check_type!(tap::TapCheck);

                // remote
                read_check_type!(dns::DnsCheck);
                read_check_type!(http::HttpCheck, rewrites);
                read_check_type!(ping::PingCheck);
                read_check_type!(tcp::TcpCheck);
                read_check_type!(udp::UdpCheck);

                // local
                read_check_type!(apt::AptCheck);
                read_check_type!(defaults::DefaultsCheck, rewrites);
                read_check_type!(fs::FilesystemCheck, rewrites);
                read_check_type!(gem::GemCheck);
                read_check_type!(group::GroupCheck);
                read_check_type!(hashes::HashCheck, rewrites);
                read_check_type!(homebrew_cask::HomebrewCaskCheck);
                read_check_type!(homebrew::HomebrewCheck);
                read_check_type!(homebrew_tap::HomebrewTapCheck);
                read_check_type!(npm::NpmCheck);
                read_check_type!(systemd::SystemdCheck);
                read_check_type!(ufw::UfwCheck);
                read_check_type!(user::UserCheck, rewrites);

                let error = ReadError {
                    name: check_key.clone().into(),
                    inner: Box::new(UnknownCheckType(check_key.clone())),
                };

                errors.push(error);
            }

            if filter.order == RunningOrder::Random {
                trace!("Shuffling order of checks");
                rand::seq::SliceRandom::shuffle(self.checks.as_mut_slice(), &mut rand::thread_rng());
            }
        }

        if errors.is_empty() {
            Ok(())
        }
        else {
            Err(errors)
        }
    }

    /// Tells the commands in the input Commands set to prepare themselves
    /// based on the data that has been loaded.
    ///
    /// For commands with just one invocation (such as `apt`), this will have
    /// the command prime the Exec. For those with multiple invocations (such
    /// as `dns`), this will have the command prime all the necessary Execs.
    /// Checks with no commands (such as `fs`) have nothing done to them.
    pub fn prime_commands(&self, commands: &mut Commands) {
        for c in &self.checks {
            match &c.class {
                LoadedCheck::Cmd(c)           => c.load(&mut commands.shell),
                LoadedCheck::Tap(c)           => c.load(&mut commands.shell),

                LoadedCheck::Dns(c)           => c.load(&mut commands.dig),
                LoadedCheck::Http(c)          => c.load(&mut commands.curl),
                LoadedCheck::Ping(c)          => c.load(&mut commands.ping),
                LoadedCheck::Tcp(c)           => c.load(&mut commands.net),
                LoadedCheck::Udp(c)           => c.load(&mut commands.net),

                LoadedCheck::Apt(c)           => c.load(&mut commands.apt),
                LoadedCheck::Defaults(c)      => c.load(&mut commands.defaults),
                LoadedCheck::Fs(c)            => c.load(&mut commands.files),
                LoadedCheck::Gem(c)           => c.load(&mut commands.gem),
                LoadedCheck::Group(c)         => c.load(&mut commands.passwd),
                LoadedCheck::Hash(c)          => c.load(&mut commands.hash),
                LoadedCheck::Homebrew(c)      => c.load(&mut commands.brew),
                LoadedCheck::HomebrewCask(c)  => c.load(&mut commands.brew_cask),
                LoadedCheck::HomebrewTap(c)   => c.load(&mut commands.brew_tap),
                LoadedCheck::Npm(c)           => c.load(&mut commands.npm),
                LoadedCheck::Systemd(c)       => c.load(&mut commands.systemctl),
                LoadedCheck::Ufw(c)           => c.load(&mut commands.ufw),
                LoadedCheck::User(c)          => c.load(&mut commands.passwd),
            }
        }
    }

    /// Runs all the checks in this set in type order, running external
    /// programs using the `Executor` from commands in the `Commands` set, and
    /// printing results out to the `TerminalUI`.
    pub fn run_all<'set>(&'set self, executor: &mut Executor, commands: &mut Commands, ui: &mut Output, delay: Delay, table: Option<&mut AnalysisTable<'set, LoadedCheck>>) -> ResultsSection {
        let mut check_outputs = Vec::new();
        let mut first = true;

        for ready_check in &self.checks {
            if let Delay::Wait(duration) = delay {
                if first {
                    sleep(duration);
                }
                else {
                    first = false;
                }
            }

            let check_output = run_base_check(&ready_check, executor, commands, ui);

            if let Some(&mut ref mut table) = table {
                let properties = match ready_check.class {
                    LoadedCheck::Fs(ref c)     => c.properties(),
                    LoadedCheck::User(ref c)   => c.properties(),
                    LoadedCheck::Group(ref c)  => c.properties(),
                    _                          => Vec::new(),
                };

                table.add(&ready_check.class, properties.into_iter(), check_output.passed);
            }

            check_outputs.push(check_output);
        }

        let mut totals = Stats::default();
        for check_output in &check_outputs {
            if check_output.passed {
                totals.pass_count += 1;
            }
            else {
                totals.fail_count += 1;
            }
        }

        ResultsSection { check_outputs, totals }
    }

    pub fn run_continual_batch(&mut self, executor: &mut Executor, commands: &mut Commands, ui: &mut Output, order: RunningOrder, delay: Delay) {
        if order == RunningOrder::Random {
            trace!("Shuffling order of all checks");
            rand::seq::SliceRandom::shuffle(self.checks.as_mut_slice(), &mut rand::thread_rng());
        }

        for ready_check in &self.checks {
            run_base_check(ready_check, executor, commands, ui);

            if let Delay::Wait(duration) = delay {
                sleep(duration);
            }
        }
    }

    /// Whether this set has no checks in it. Empty check files are usually a
    /// mistake, and should be warned about, rather than being classified as
    /// “100% successful (0/0)”.
    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }

    /// Formats each check in the set as a string containing their check type
    /// name and description, and returns them as a vector.
    pub fn list_checks(self) -> Vec<String> {
        self.checks.into_iter()
            .map(|e| format!("[{}] {}", e.class.name(), e.class))
            .collect()
    }
}


fn run_base_check(ready_check: &ReadyCheck, executor: &mut Executor, commands: &mut Commands, ui: &mut Output) -> CheckOutput {
    macro_rules! results_to_output {
        ($c:expr, $name:expr, $results:expr) => {{
            let results = $results;
            ui.print_check($c, $name, &results);

            let passed = results.iter().all(CheckResult::passed);
            let message = $c.to_string();

            let results = results.iter().map(|e| {
                match e {
                    CheckResult::Passed(pass)       => ResultMessage::Passed(pass.to_string()),
                    CheckResult::Failed(fail)       => ResultMessage::Failed(fail.to_string()),
                    CheckResult::CommandError(err)  => ResultMessage::Error(err.to_string()),
                }
            }).collect();

            CheckOutput { passed, results, message }
        }}
    }

    let name = ready_check.name.as_ref();

    match &ready_check.class {
        LoadedCheck::Cmd(c)           => results_to_output!(c, name, c.check(executor, &commands.shell)),
        LoadedCheck::Tap(c)           => results_to_output!(c, name, c.check(executor, &commands.shell)),

        LoadedCheck::Dns(c)           => results_to_output!(c, name, c.check(executor, &commands.dig)),
        LoadedCheck::Http(c)          => results_to_output!(c, name, c.check(executor, &commands.curl)),
        LoadedCheck::Ping(c)          => results_to_output!(c, name, c.check(executor, &commands.ping)),
        LoadedCheck::Tcp(c)           => results_to_output!(c, name, c.check(&commands.net)),
        LoadedCheck::Udp(c)           => results_to_output!(c, name, c.check(&commands.net)),

        LoadedCheck::Apt(c)           => results_to_output!(c, name, c.check(executor, &commands.apt)),
        LoadedCheck::Defaults(c)      => results_to_output!(c, name, c.check(executor, &commands.defaults)),
        LoadedCheck::Fs(c)            => results_to_output!(c, name, c.check(&commands.files)),
        LoadedCheck::Gem(c)           => results_to_output!(c, name, c.check(executor, &commands.gem)),
        LoadedCheck::Group(c)         => results_to_output!(c, name, c.check(&commands.passwd)),
        LoadedCheck::Hash(c)          => results_to_output!(c, name, c.check(executor, &commands.hash)),
        LoadedCheck::Homebrew(c)      => results_to_output!(c, name, c.check(executor, &commands.brew)),
        LoadedCheck::HomebrewCask(c)  => results_to_output!(c, name, c.check(executor, &commands.brew_cask)),
        LoadedCheck::HomebrewTap(c)   => results_to_output!(c, name, c.check(executor, &commands.brew_tap)),
        LoadedCheck::Npm(c)           => results_to_output!(c, name, c.check(executor, &commands.npm)),
        LoadedCheck::Systemd(c)       => results_to_output!(c, name, c.check(executor, &commands.systemctl)),
        LoadedCheck::Ufw(c)           => results_to_output!(c, name, c.check(executor, &commands.ufw)),
        LoadedCheck::User(c)          => results_to_output!(c, name, c.check(&commands.passwd)),
    }
}


/// An error that occurs during reading, when the checks have complained about
/// the schema or format of one or more tables in the input data.
pub struct ReadError {

    /// The name of the table that had an error.
    pub name: Cow<'static, str>,

    /// The error that caused reading to fail.
    pub inner: Box<dyn fmt::Display>,
}


#[derive(Debug)]
pub struct UnknownCheckType(String);

impl fmt::Display for UnknownCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown check type {:?}", self.0)
    }
}

impl LoadedCheck {
    fn name(&self) -> &'static str {
        match self {

            // command
            Self::Cmd(_)           => cmd::CommandCheck::TYPE,
            Self::Tap(_)           => tap::TapCheck::TYPE,

            // network
            Self::Dns(_)           => dns::DnsCheck::TYPE,
            Self::Http(_)          => http::HttpCheck::TYPE,
            Self::Ping(_)          => ping::PingCheck::TYPE,
            Self::Tcp(_)           => tcp::TcpCheck::TYPE,
            Self::Udp(_)           => udp::UdpCheck::TYPE,

            // local
            Self::Apt(_)           => apt::AptCheck::TYPE,
            Self::Defaults(_)      => defaults::DefaultsCheck::TYPE,
            Self::Fs(_)            => fs::FilesystemCheck::TYPE,
            Self::Gem(_)           => gem::GemCheck::TYPE,
            Self::Group(_)         => group::GroupCheck::TYPE,
            Self::Hash(_)          => hashes::HashCheck::TYPE,
            Self::Homebrew(_)      => homebrew::HomebrewCheck::TYPE,
            Self::HomebrewCask(_)  => homebrew_cask::HomebrewCaskCheck::TYPE,
            Self::HomebrewTap(_)   => homebrew_tap::HomebrewTapCheck::TYPE,
            Self::Npm(_)           => npm::NpmCheck::TYPE,
            Self::Systemd(_)       => systemd::SystemdCheck::TYPE,
            Self::Ufw(_)           => ufw::UfwCheck::TYPE,
            Self::User(_)          => user::UserCheck::TYPE,
        }
    }
}
