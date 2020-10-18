use std::fmt;
use std::rc::Rc;

use spec_exec::{Executor, ExecError};


/// The Check trait is implemented by all the possible checks.
pub trait Check: fmt::Display {

    /// The name of the table for checks of this type.
    const TYPE: &'static str;
}

/// The result of running a check part against a command’s output.
///
/// Based on [the original](https://thedailywtf.com/articles/What_Is_Truth_0x3f_).
#[derive(Debug)]
pub enum CheckResult<PASS, FAIL> {

    /// This part passed! `:)`
    Passed(PASS),

    /// This part failed. `:(`
    Failed(FAIL),

    /// A command didn’t execute as expected. `:S`
    CommandError(Rc<ExecError>),
}

impl<PASS, FAIL> CheckResult<PASS, FAIL> {

    /// Whether this result passed or not.
    /// This is used when determining whether an entire check succeeded or failed.
    pub fn passed(&self) -> bool {
        if let Self::Passed(_) = self {
            true
        }
        else {
            false
        }
    }
}


/// The trait that links checks with the commands that run them.
/// It should be implemented for all checks that run a command.
///
/// It’s used to make sure all the checks have the same function names
/// and signatures, even if that interface is not used directly.
///
/// This is a two-step process that ensures the commands listed in
/// `list-commands` are the ones actually executed.
pub trait RunCheck<CMD>: Check {

    /// The type this check produces as a successful result.
    type PASS: PassResult;

    /// The type this check produces as a failure result.
    type FAIL: FailResult;

    /// Step 1: ready the commands to run.
    fn load(&self, command: &mut CMD);

    /// Step 2: run the commands that we readied just there, using the given executor.
    fn check(&self, executor: &mut Executor, command: &CMD) -> Vec<CheckResult<Self::PASS, Self::FAIL>>;
}


/// The trait for checks that don't run a command, and examine the
/// system directly instead. It should be implemented for all checks of
/// this nature.
///
/// It’s used to make sure all the checks have the same function names
/// and signatures, even if that interface is not used directly.
pub trait BuiltInCheck<CMD>: Check {

    /// The type this check produces as a successful result.
    type PASS: PassResult;

    /// The type this check produces as a failure result.
    type FAIL: FailResult;

    /// Step 1: ready the commands to run.
    fn load(&self, command: &mut CMD);

    /// Step 2: effectively run the commands that we readied just there,
    /// using Rust code rather than an executor.
    fn check(&self, command: &CMD) -> Vec<CheckResult<Self::PASS, Self::FAIL>>;
}


/// The trait for successful check results.
pub trait PassResult: fmt::Display {

    /// Get the output of the command that was run, if any.
    fn command_output(&self) -> Option<(String, &String)> { None }
}


/// The trait for failure check results.
pub trait FailResult: fmt::Display {

    /// Get the output of the command that was run, if any.
    fn command_output(&self) -> Option<(String, &String)> { None }

    /// Get a diff between some expected output and the actual output.
    fn diff_output(&self) -> Option<(String, &String, &String)> { None }
}
