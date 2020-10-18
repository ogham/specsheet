//! Specsheet!

#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rustdoc)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused)]

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::map_entry)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::single_match)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::wildcard_imports)]
#![warn(clippy::clone_on_ref_ptr)]

#![allow(unsafe_code)]   // needed for libc::kill

use std::env;

use log::*;

use spec_analysis::AnalysisTable;

mod commands;
use self::commands::Commands;

mod doc;
use self::doc::{CompletedRun, CompletedSection};

mod filter;

mod input;
use self::input::InputSource;

mod logger;

mod options;
use self::options::{Options, RunningMode, RunningDirectory, OptionsResult, HelpReason};

mod output;

mod results;
use self::results::Stats;

mod set;
use self::set::CheckSet;

mod side;

mod terminal_ui;


fn main() {
    use std::process::exit;

    logger::configure(env::var_os("SPECSHEET_DEBUG"));

    match Options::getopts(env::args_os().skip(1)) {
        OptionsResult::Ok(opts) => {
            exit(run(opts));
        }

        OptionsResult::Help(help_reason, use_colours) => {
            if use_colours.should_use_colours() {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/usage.pretty.txt")));
            }
            else {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/usage.bland.txt")));
            }

            if help_reason == HelpReason::NoArguments {
                exit(exits::OPTIONS_ERROR);
            }
            else {
                exit(exits::SUCCESS);
            }
        }

        OptionsResult::Version(use_colours) => {
            if use_colours.should_use_colours() {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/version.pretty.txt")));
            }
            else {
                print!("{}", include_str!(concat!(env!("OUT_DIR"), "/version.bland.txt")));
            }

            exit(exits::SUCCESS);
        }

        OptionsResult::InvalidOptionsFormat(why) => {
            eprintln!("{}", why);
            exit(exits::OPTIONS_ERROR);
        }

        OptionsResult::InvalidOptions(e) => {
            eprintln!("{}", e);
            exit(exits::OPTIONS_ERROR);
        }
    }
}


fn run(options: Options) -> i32 {
    use spec_exec::Executor;

    let Options { mode, inputs, filter, rewrites, output } = options;
    debug!("Mode -> {:#?}", mode);
    debug!("Input files -> {:#?}", inputs);
    debug!("Filter -> {:#?}", filter);
    debug!("Rewrites -> {:#?}", rewrites);
    debug!("Output -> {:#?}", output);

	let mut ui = output.ui();
    let mut file_errored = false;
    let mut checks_have_failed = false;

    match mode {
        RunningMode::Run(check_opts, end_opts) => {
            let mut executor = Executor::new();
            let mut commands = Commands::from_global_options(&check_opts.global_options).expect("Invalid overrides");

            let here = env::current_dir().expect("current_dir");
            let here = here.canonicalize().expect("canonicalize");

            if let RunningDirectory::OtherDirectory(other_dir) = &check_opts.directory {
                debug!("Changing directory to specified directory -> {:?}", other_dir);
                env::set_current_dir(other_dir).expect("set_current_dir to other_dir");
            }

            let mut side_child = None;
            if let Some(side_process) = &check_opts.process {
                let pid = side_process.start();
                debug!("Process started -> {}", pid);
                side_child = Some(pid);
            }

            let mut sections = Vec::new();
            for input_source in inputs {

                // TODO: this table should be shared between all input sources,
                // and only analysed at the end.
                // I tried to do this but the lifetimes get all screwy
                let mut analysis_table = None;
                if end_opts.perform_analysis {
                    analysis_table = Some(AnalysisTable::new());
                }

                // Printing the file section when the only input is
                // stdin just takes up space
                if ! input_source.is_stdin() {
                    ui.print_file_section(&input_source);
                }

                let check_document = match input_source.load() {
                    Ok(cd) => cd,
                    Err(e) => {
                        ui.print_load_error(&input_source, e);
                        file_errored = true;
                        continue;
                    }
                };

                let mut checks = CheckSet::new();
                match checks.read_toml(&filter, &rewrites, check_document) {
                    Ok(()) => {},
                    Err(es) => {
                        ui.print_read_errors(&es);
                        file_errored = true;
                    }
                }

                if let RunningDirectory::CheckDirectory = &check_opts.directory {
                    if let InputSource::File(path) = &input_source {
                        let base_directory = path.canonicalize().expect("canonicalize");
                        let base_directory = base_directory.parent().expect("parent");
                        debug!("Changing directory to check directory -> {:?}", base_directory);
                        env::set_current_dir(base_directory).expect("set_current_dir");
                    }
                }

                checks.prime_commands(&mut commands);
                let section = checks.run_all(&mut executor, &mut commands, &mut ui, check_opts.delay, analysis_table.as_mut());

                ui.print_stats(section.totals);

                if section.failed() {
                    checks_have_failed = true;
                }

				let completed_section = CompletedSection { input: input_source, results: section };
                sections.push(completed_section);

                debug!("Changing to original directory -> {:?}", here);
                env::set_current_dir(&here).expect("set_current_dir to here");

                if let Some(table) = analysis_table {
                    let corals = table.resolve_correlations();

                    if corals.is_empty() {
                        println!("No correlations detected.");
                    }
                    else {
                        println!("\nAnalysis:");
                        for correlation in corals {
                            println!("- Failures {} (×{}, with 0 successes)", correlation.property, correlation.count);
                        }
                    }
                }
            }

            if let (Some(side_child), Some(side_handle)) = (check_opts.process, side_child) {
                side_child.stop(side_handle).expect("stop");
            }

            ui.print_end();


            let commands = executor.to_commands();

            let mut totals = Stats::default();
            for section in &sections {
                totals += section.results.totals;
            }

            let run = CompletedRun { sections, commands: commands.collect(), totals };
            match end_opts.result_documents.write(run) {
                Ok(()) => {
                    debug!("Output documents written OK.");
                }
                Err(e) => {
                    eprintln!("Error writing output document: {}", e);
                    file_errored = true;
                }
            }
        }

        RunningMode::Continual(check_opts) => {
            // One check set for all input files.
            let mut checks = CheckSet::new();

            for input_source in inputs {
                let check_document = match input_source.load() {
                    Ok(cd) => cd,
                    Err(e) => {
                        ui.print_load_error(&input_source, e);
                        file_errored = true;
                        continue;
                    }
                };

                match checks.read_toml(&filter, &rewrites, check_document) {
                    Ok(()) => {},
                    Err(es) => {
                        ui.print_read_errors(&es);
                        file_errored = true;
                    }
                }
            }

            if file_errored {
                return exits::FILE_ERROR;
            }

            loop {
                let mut executor = Executor::new();
                let mut commands = Commands::from_global_options(&check_opts.global_options).expect("Invalid overrides");

                checks.prime_commands(&mut commands);
                checks.run_continual_batch(&mut executor, &mut commands, &mut ui, filter.order, check_opts.delay);
            }
        }

        RunningMode::SyntaxCheckOnly => {
            for input_source in inputs {
                let check_document = match input_source.load() {
                    Ok(cd) => cd,
                    Err(e) => {
                        ui.print_load_error(&input_source, e);
                        file_errored = true;
                        continue;
                    }
                };

                let mut checks = CheckSet::new();
                match checks.read_toml(&filter, &rewrites, check_document) {
                    Ok(()) => {
                        if checks.is_empty() {
                            println!("{} contains no checks", input_source);
                        }
                        else {
                            println!("{} syntax OK", input_source);
                        }
                    }
                    Err(es) => {
                        ui.print_read_errors(&es);
                        file_errored = true;
                    }
                }
            }
        }

        RunningMode::ListCommandsOnly(global_options) => {
            let mut checks = CheckSet::new();

            for input_source in inputs {
                let check_document = match input_source.load() {
                    Ok(cd) => cd,
                    Err(e) => {
                        ui.print_load_error(&input_source, e);
                        file_errored = true;
                        continue;
                    }
                };

                match checks.read_toml(&filter, &rewrites, check_document) {
                    Ok(()) => {},
                    Err(es) => {
                        ui.print_read_errors(&es);
                        file_errored = true;
                    }
                }
            }

            let mut commands = Commands::from_global_options(&global_options).expect("Invalid overrides");

            checks.prime_commands(&mut commands);
            for command in commands.list_commands() {
                println!("{:?}", command);
            }
        }

        RunningMode::ListChecksOnly => {
            for input_source in inputs {
                ui.print_file_section(&input_source);

                let check_document = match input_source.load() {
                    Ok(cd) => cd,
                    Err(e) => {
                        ui.print_load_error(&input_source, e);
                        file_errored = true;
                        continue;
                    }
                };

                let mut checks = CheckSet::new();
                match checks.read_toml(&filter, &rewrites, check_document) {
                    Ok(()) => {},
                    Err(es) => ui.print_read_errors(&es),
                }

                for check in checks.list_checks() {
                    println!("{}", check);
                }
            }
        }

        RunningMode::ListTagsOnly => {
            use std::collections::BTreeSet;
            use spec_checks::load::Tags;

            let mut all_tags = BTreeSet::new();
            for input_source in inputs {
                let check_document = match input_source.load() {
                    Ok(cd) => cd,
                    Err(e) => {
                        ui.print_load_error(&input_source, e);
                        file_errored = true;
                        continue;
                    }
                };

                for check in check_document.values().flatten() {
                    if let Some(tags) = &check.tags {
                        match tags {
                            Tags::One(one)   => { all_tags.insert(one.clone()); },
                            Tags::Many(many) => { all_tags.extend(many.iter().cloned()); },
                        }
                    }
                }
            }

            if all_tags.is_empty() {
                warn!("There are no tags to list!");
            }

            for tag in all_tags {
                println!("{}", tag);
            }
        }
    }

    if file_errored {
        exits::FILE_ERROR
    }
    else if checks_have_failed {
        exits::CHECKS_HAVE_FAILED
    }
    else {
        exits::SUCCESS
    }
}


mod exits {

    /// Exit code for when everything turned out OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when at least one check failed.
    pub const CHECKS_HAVE_FAILED: i32 = 1;

    /// Exit code for when there was a TOML syntax or read error loading one
    /// or more of the input files, or writing a check document.
    pub const FILE_ERROR: i32 = 2;

    /// Exit code for when the command-line options were invalid.
    pub const OPTIONS_ERROR: i32 = 3;
}
