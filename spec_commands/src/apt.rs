//! The `apt` command.
//!
//! # Sample output
//!
//! ```text
//! $ apt list --installed
//! Listing... Done
//! accountsservice/bionic,now 0.6.45-1ubuntu1 amd64 [installed]
//! acl/bionic,now 2.2.52-3build1 amd64 [installed]
//! acpid/bionic,now 1:2.0.28-1ubuntu1 amd64 [installed]
//! adduser/bionic,now 3.116ubuntu1 all [installed]
//! apparmor/bionic-updates,bionic-security,now 2.12-4ubuntu5.1 amd64 [installed]
//! apport/now 2.20.9-0ubuntu7.5 all [installed,upgradable to: 2.20.9-0ubuntu7.6]
//! ```

use std::rc::Rc;

use log::*;

use spec_checks::apt::RunApt;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **apt command** that runs the `apt` binary.
#[derive(Debug)]
pub struct AptCommand {
    exec: Option<Exec<AptOutput>>,
}

impl AptCommand {

    /// Creates a new apt command.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("apt.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunApt for AptCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming apt command");
            self.exec = Some(Exec::actual(apt_list_installed_cmd()));
        }
    }

    fn find_package(&self, executor: &mut Executor, package_name: &str) -> Result<Option<String>, Rc<ExecError>> {
        debug!("Finding apt package -> {:?}", package_name);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_package(package_name))
    }
}

fn apt_list_installed_cmd() -> Command {
    let mut cmd = Command::new("apt");
    cmd.arg("list").arg("--installed");
    cmd
}


/// The **apt output** encapsulates the output lines of an
/// invoked `AptCommand`.
#[derive(Debug)]
pub struct AptOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for AptOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl AptOutput {

    /// Searches through the lines of output for a package with the given
    /// name, returning its version number if found.
    fn find_package(&self, package_name: &str) -> Option<String> {
        let mut prefix = String::from(package_name);
        prefix.push('/');

        if let Some(line) = self.lines.iter().find(|line| line.starts_with(&prefix)) {
            let version = line.split(' ').nth(1).unwrap();
            Some(version.into())
        }
        else {
            None
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn some_apt_packages() {
        let lines = vec![
            String::from("python3.5-minimal/xenial-updates,xenial-security,now 3.5.2-2ubuntu0~16.04.4 amd64 [installed]").into(),
            String::from("rake/xenial,now 10.5.0-2 all [installed,automatic]").into(),
        ];

        let output = AptOutput { lines };

        assert_eq!(Some("10.5.0-2".into()),               output.find_package("rake"));
        assert_eq!(Some("3.5.2-2ubuntu0~16.04.4".into()), output.find_package("python3.5-minimal"));

        assert_eq!(None, output.find_package("exa"));
        assert_eq!(None, output.find_package("python"));
        assert_eq!(None, output.find_package("python3.5"));
    }
}
