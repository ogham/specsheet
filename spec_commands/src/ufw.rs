//! The `ufw` command.
//!
//! # Sample output
//!
//! ```text
//! Status: active
//!
//! To                         Action      From
//! --                         ------      ----
//! 22/tcp                     ALLOW       Anywhere
//! 22/tcp (v6)                ALLOW       Anywhere (v6)
//! ```

use std::rc::Rc;

use log::*;
use once_cell::sync::Lazy;
use regex::Regex;

use spec_checks::ufw::{RunUfw, Portspec, Protocol, Rule};
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **ufw command** that runs the `ufw` binary.
#[derive(Debug)]
pub struct UfwCommand {
    exec: Option<Exec<UfwOutput>>,
}


impl UfwCommand {

    /// Creates a new command to run `ufw`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("ufw.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunUfw for UfwCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming ufw command");
            self.exec = Some(Exec::actual(ufw_list_rules_cmd()));
        }
    }

    fn find_rule(&self, executor: &mut Executor, portspec: Portspec, protocol: Protocol) -> Result<Option<Rule>, Rc<ExecError>> {
        debug!("Finding ufw rule -> {:?}/{:?}", portspec, protocol);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_rule(portspec, protocol))
    }
}

fn ufw_list_rules_cmd() -> Command {
    let mut cmd = Command::new("ufw");
    cmd.arg("status").arg("verbose");
    cmd
}


/// The **ufw output** encapsulates the output lines of an
/// invoked `UfwCommand`.
#[derive(Debug)]
pub struct UfwOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for UfwOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl UfwOutput {

    /// Searches the output lines for a rule with the given protocol and port
    /// specification, returning the rest of the fields for that rule if one
    /// is found, or nothing if none of the rules matches.
    fn find_rule(&self, portspec: Portspec, protocol: Protocol) -> Option<Rule> {
        let mut prefix = String::new();

        match portspec {
            Portspec::One(one)        => prefix.push_str(&one.to_string()),
            Portspec::Range(from, to) => prefix.push_str(&format!("{}:{}", from, to)),
        }

        match protocol {
            Protocol::TCP => prefix.push_str("/tcp"),
            Protocol::UDP => prefix.push_str("/udp"),
        }

        if let Some(line) = self.lines.iter().find(|line| line.starts_with(&prefix)) {
            let caps  = REGEX.captures(line).expect(line);

            let iface = caps.get(4).map(|s| s.as_str().to_owned());
            let allow = caps.get(6).unwrap().as_str().trim().to_owned();
            let ipv6  = caps.get(7).is_some();
            Some(Rule { iface, allow, ipv6 })
        }
        else {
            None
        }
    }
}


/// Regular expression used to extract data from a line of ufw output.
static REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r##"(?x) ^
        (\d+) (: \d+)?      # from (and to) ports
        / (tcp|udp) \s+     # the protocol
        (?:
            on \s+
            ( .+? ) \s+     # interface
        )?
        (ALLOW \s IN | ALLOW) \s+  # directive
        (.+?)                # allow from
        (?:
            \s+
            ( \( v6 \) )    # ipv6?
        )?
    $ "##).unwrap()
});



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn some_ufw_rules() {
        let lines = vec![
            Rc::from("22/tcp                     ALLOW IN    Anywhere"),
            Rc::from("60000:61000/udp            ALLOW IN    Anywhere"),
        ];

        let output = UfwOutput { lines };

        assert_eq!(Some(Rule { iface: None, allow: "Anywhere".into(), ipv6: false }), output.find_rule(Portspec::One(22),             Protocol::TCP));
        assert_eq!(Some(Rule { iface: None, allow: "Anywhere".into(), ipv6: false }), output.find_rule(Portspec::Range(60000, 61000), Protocol::UDP));

        assert_eq!(None, output.find_rule(Portspec::One(23), Protocol::TCP));
        assert_eq!(None, output.find_rule(Portspec::One(22), Protocol::UDP));
    }


    #[test]
    fn interface_and_ipv6_ufw_rules() {
        let lines = vec![
            Rc::from("8500/tcp on eth0           ALLOW IN    Anywhere"),
            Rc::from("8302/tcp on eth0           ALLOW IN    Anywhere (v6)"),
            Rc::from("60000:61000/udp            ALLOW IN    Anywhere (v6)"),
        ];

        let output = UfwOutput { lines };

        assert_eq!(Some(Rule { iface: Some("eth0".into()), allow: "Anywhere".into(), ipv6: false }), output.find_rule(Portspec::One(8500),          Protocol::TCP));
        assert_eq!(Some(Rule { iface: Some("eth0".into()), allow: "Anywhere".into(), ipv6: true }),  output.find_rule(Portspec::One(8302),           Protocol::TCP));
        assert_eq!(Some(Rule { iface: None,                allow: "Anywhere".into(), ipv6: true }),  output.find_rule(Portspec::Range(60000, 61000), Protocol::UDP));
    }
}
