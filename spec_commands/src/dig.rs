//! The `dig` command
//!
//! # Sample output
//!
//! The output is basically a description of the underlying DNS packets.
//! It includes blank lines and commented lines.
//!
//! ```text
//! $ dig -t txt cheese.singles
//!
//! ; <<>> DiG 9.10.6 <<>> -t txt cheese.singles
//! ;; global options: +cmd
//! ;; Got answer:
//! ;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 36382
//! ;; flags: qr rd ra; QUERY: 1, ANSWER: 0, AUTHORITY: 1, ADDITIONAL: 1
//!
//! ;; OPT PSEUDOSECTION:
//! ; EDNS: version: 0, flags:; udp: 512
//! ;; QUESTION SECTION:
//! ;cheese.singles.			IN	TXT
//!
//! ;; AUTHORITY SECTION:
//! cheese.singles.		900	IN	SOA	ns-487.awsdns-60.com. awsdns-hostmaster.amazon.com. 1 7200 900 1209600 86400
//!
//! ;; Query time: 97 msec
//! ;; SERVER: 192.168.1.1#53(192.168.1.1)
//! ;; WHEN: Sat Oct 05 08:05:53 BST 2019
//! ;; MSG SIZE  rcvd: 124
//!
//! ```


use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;

use log::*;

use spec_checks::dns::{RunDns, Request, Nameserver};
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **dig command** that runs the `dig` binary.
#[derive(Debug)]
pub struct DigCommand {
    results: BTreeMap<Request, Exec<DigOutput>>,
    timeout: Option<Duration>,
    default_nameserver: Option<String>,
}

impl DigCommand {

    /// Creates a new command to run `dig`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let results = BTreeMap::new();
        let timeout = global_options.duration("dns.timeout");
        let default_nameserver = global_options.key_value("dns.nameserver");
        Self { results, timeout, default_nameserver }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().flat_map(|e| e.1.into_command())
    }
}

impl RunDns for DigCommand {
    fn prime(&mut self, request: &Request) {
        if ! self.results.contains_key(request) {
            debug!("Priming dig command with {:?}", request);
            let exec = Exec::actual(dig_cmd(request));
            self.results.insert(request.clone(), exec);
        }
    }

    fn get_values(&self, executor: &mut Executor, request: &Request) -> Result<Vec<Rc<str>>, Rc<ExecError>> {
        debug!("Finding dns records -> {:?}", request);
        let output = self.results[request].run(executor)?;
        Ok(output.clone_lines())
    }
}

fn dig_cmd(request: &Request) -> Command {
    let mut cmd = Command::new("dig");
    cmd.arg("+short");

    if let Nameserver::ByIP(ref ip) = request.nameserver {
        cmd.arg(format!("@{}", ip));
    }

    cmd.arg("-t").arg(format!("{:?}", request.rtype));
    cmd.arg(&request.domain);
    cmd
}


/// The **dig output** encapsulates the output lines of an
/// invoked `DigCommand`.
#[derive(Debug)]
pub struct DigOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for DigOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl DigOutput {
    fn clone_lines(&self) -> Vec<Rc<str>> {
        self.lines.clone()
    }
}
