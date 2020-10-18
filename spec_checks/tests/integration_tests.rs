mod command;
mod local;
mod network;


// some shared imports, that get used by every test:

pub use std::fmt;
pub use std::rc::Rc;

pub use toml::{toml, map::Map};

pub use spec_checks::{RunCheck, CheckResult};
pub use spec_exec::{Executor, ExecError};


pub fn phrase<PASS, FAIL>(cr: CheckResult<PASS, FAIL>) -> String
where PASS: fmt::Display, FAIL: fmt::Display
{
    match cr {
        CheckResult::Passed(pass)      => format!("PASS {}", pass.to_string()),
        CheckResult::Failed(fail)      => format!("FAIL {}", fail.to_string()),
        CheckResult::CommandError(err) => format!("ERR  {}", err.to_string()),
    }
}
