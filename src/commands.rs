use std::collections::BTreeMap;
use std::rc::Rc;

use spec_commands::*;
use spec_exec::{Exec, Command, CommandOutput, ExitReason};


/// The **command set** contain the commands that hold the Execs.
#[derive(Debug)]
pub struct Commands {
    pub apt:        apt::AptCommand,
    pub brew:       brew::BrewCommand,
    pub brew_cask:  brew_cask::BrewCaskCommand,
    pub brew_tap:   brew_tap::BrewTapCommand,
    pub curl:       curl::CurlCommand,
    pub defaults:   defaults::DefaultsCommand,
    pub dig:        dig::DigCommand,
    pub files:      files::FilesystemNonCommand,
    pub gem:        gem::GemCommand,
    pub hash:       hash::HashCommand,
    pub net:        net::NetNonCommand,
    pub npm:        npm::NpmCommand,
    pub passwd:     passwd::PasswdNonCommand,
    pub ping:       ping::PingCommand,
    pub shell:      shell::ShellCommand,
    pub systemctl:  systemctl::SystemctlCommand,
    pub ufw:        ufw::UfwCommand,
}

impl Commands {

    /// Creates a new set of commands from the given set of overrides.
    /// Each command assembles its own Execs based on the overrides.
    pub fn from_global_options(global_options: &GlobalOptions) -> Option<Self> {
        Some(Self {
            apt:        apt::AptCommand::create(global_options),
            brew:       brew::BrewCommand::create(global_options),
            brew_cask:  brew_cask::BrewCaskCommand::create(global_options),
            brew_tap:   brew_tap::BrewTapCommand::create(global_options),
            curl:       curl::CurlCommand::create(global_options)?,
            defaults:   defaults::DefaultsCommand::create(global_options),
            dig:        dig::DigCommand::create(global_options),
            files:      files::FilesystemNonCommand::create(global_options),
            gem:        gem::GemCommand::create(global_options),
            hash:       hash::HashCommand::create(global_options),
            net:        net::NetNonCommand::create(global_options),
            npm:        npm::NpmCommand::create(global_options),
            passwd:     passwd::PasswdNonCommand::create(global_options),
            ping:       ping::PingCommand::create(global_options),
            shell:      shell::ShellCommand::create(global_options),
            systemctl:  systemctl::SystemctlCommand::create(global_options),
            ufw:        ufw::UfwCommand::create(global_options),
        })
    }

    /// Iterates through all the command types, returning a vector of
    /// the Command values that have been loaded. This is presented to
    /// the user as the list of commands that would have been run.
    pub fn list_commands(self) -> Vec<Command> {
        let mut commands = Vec::new();
        commands.extend(self.apt.commands());
        commands.extend(self.brew.commands());
        commands.extend(self.brew_cask.commands());
        commands.extend(self.brew_tap.commands());
        commands.extend(self.curl.commands());
        commands.extend(self.defaults.commands());
        commands.extend(self.dig.commands());
        commands.extend(self.files.commands());
        commands.extend(self.gem.commands());
        commands.extend(self.hash.commands());
        commands.extend(self.net.commands());
        commands.extend(self.npm.commands());
        commands.extend(self.passwd.commands());
        commands.extend(self.ping.commands());
        commands.extend(self.shell.commands());
        commands.extend(self.systemctl.commands());
        commands.extend(self.ufw.commands());
        commands
    }
}


/// The set of global options are created from user input.
#[derive(PartialEq, Debug, Default)]
pub struct GlobalOptions {
    pub map: BTreeMap<String, String>,
}

impl spec_commands::GlobalOptions for GlobalOptions {
    fn key_value(&self, key_name: &'static str) -> Option<String> {
        self.map.get(key_name).cloned()
    }

    fn key_prefix_values(&self, key_prefix: &'static str) -> BTreeMap<String, String> {
        self.map.iter()
            .filter(|e| e.0.starts_with(key_prefix))
            .map(|e| (e.0.clone(), e.1.clone()))
            .collect()
    }

    fn duration(&self, key_name: &'static str) -> Option<std::time::Duration> {
        if self.map.contains_key(key_name) {
            todo!("Duration not done yet");
        }

        None
    }

    fn command<T: CommandOutput>(&self, key_name: &'static str) -> Option<Exec<T>> {
        if let Some(data) = self.map.get(key_name) {
            let lines = data.lines().map(Rc::from).collect::<Vec<_>>();
            let object = T::interpret_command_output(lines, ExitReason::Status(0)).ok()?;  // todo: complain to the user here
            Some(Exec::predetermined(key_name, object))
        }
        else {
            None
        }
    }
}

