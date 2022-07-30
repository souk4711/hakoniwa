use nix::{
    sys::signal::Signal,
    sys::wait::{waitpid, WaitStatus},
    unistd::{fork, ForkResult},
};
use std::time::{Duration, Instant};

use crate::child_process;
use crate::fs;

#[derive(Default)]
enum Status {
    #[default]
    Unset,
    Ok,                  // ok
    SandboxFailure,      // sandbox exec failure
    TimeLimitExceeded,   // time limit execeeded
    OutputLimitExceeded, // output limit exceeded
    Violation,           // syscall violation
    Signaled,            // terminated with a signal
}

#[derive(Default)]
pub struct ExecutorResult {
    status: Status,
    reason: String,               // more info about the status
    exit_code: Option<i32>,       // exit code or signal number that caused an exit
    start_time: Option<Instant>,  // when process started
    finish_time: Option<Instant>, // when process finished
    real_time: Option<Duration>,  // wall time used
}

impl ExecutorResult {
    fn new() -> ExecutorResult {
        ExecutorResult {
            ..Default::default()
        }
    }
}

pub struct Executor {
    pub(crate) prog: String,
    pub(crate) argv: Vec<String>,
}

impl Executor {
    pub fn new<T: AsRef<str>>(prog: &str, argv: &[T]) -> Executor {
        let executor = Executor {
            prog: prog.to_string(),
            argv: argv.iter().map(|arg| String::from(arg.as_ref())).collect(),
        };
        executor
    }

    pub fn run(&mut self) -> ExecutorResult {
        let mut result = ExecutorResult::new();
        self.prog = match fs::find_executable_in_path(&self.prog) {
            Some(path) => match path.to_str() {
                Some(path) => path.to_string(),
                None => return result,
            },
            None => {
                return Self::set_result_with_sandbox_failure(
                    result,
                    &format!("{}: command not found", self.prog),
                )
            }
        };

        result.start_time = Some(Instant::now());
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => match waitpid(child, None) {
                Ok(ws) => Self::set_result(result, Some(ws)),
                Err(err) => Self::set_result_with_sandbox_failure(result, &err.to_string()),
            },
            Ok(ForkResult::Child) => {
                child_process::run(self);
                result
            }
            Err(err) => Self::set_result_with_sandbox_failure(result, &err.to_string()),
        }
    }

    fn set_result(mut result: ExecutorResult, ws: Option<WaitStatus>) -> ExecutorResult {
        if let Some(ws) = ws {
            match ws {
                WaitStatus::Exited(_, exit_status) => {
                    result.status = Status::Ok;
                    result.exit_code = Some(exit_status);
                }
                WaitStatus::Signaled(_, signal, _) => {
                    match signal {
                        Signal::SIGKILL => result.status = Status::TimeLimitExceeded,
                        Signal::SIGXCPU => result.status = Status::TimeLimitExceeded,
                        Signal::SIGXFSZ => result.status = Status::OutputLimitExceeded,
                        Signal::SIGSYS => result.status = Status::Violation,
                        _ => result.status = Status::Signaled,
                    }
                    result.reason = format!("signal: {}", signal);
                    result.exit_code = Some(signal as i32);
                }
                _ => {}
            }
        }

        if let Some(start_time) = result.start_time {
            let finish_time = Instant::now();
            result.finish_time = Some(finish_time);
            result.real_time = Some(finish_time.duration_since(start_time));
        }

        result
    }

    fn set_result_with_sandbox_failure(mut result: ExecutorResult, reason: &str) -> ExecutorResult {
        result.status = Status::SandboxFailure;
        result.reason = reason.to_string();
        Self::set_result(result, None)
    }
}
