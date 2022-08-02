use libc::STDERR_FILENO;
use nix::{sys::signal::Signal, sys::wait, sys::wait::WaitStatus, unistd, unistd::ForkResult};
use std::{
    path::{Path, PathBuf},
    process,
    time::{Duration, Instant},
};

use crate::{ChildProcess, FileSystem, IDMap, Limits, Namespaces};

#[derive(Default)]
enum Status {
    #[default]
    Unset,
    Ok,                  // ok
    SandboxFailure,      // sandbox setup failure
    TimeLimitExceeded,   // time limit execeeded
    OutputLimitExceeded, // output limit exceeded
    Violation,           // syscall violation
    Signaled,            // terminated with a signal
}

type ExitCode = i32;
const ECODE_FAILURE: ExitCode = 125; // the hakoniwa command itself fails
const ECODE_CANNOT_EXECUTE: ExitCode = 126; // command invoked cannot execute
const ECODE_COMMAND_NOT_FOUND: ExitCode = 127; // "command not found"

#[derive(Default)]
pub struct ExecutorResult {
    status: Status,
    reason: String,               // more info about the status
    exit_code: Option<ExitCode>,  // exit code or signal number that caused an exit
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

#[derive(Default)]
pub struct Executor {
    pub(crate) prog: String,      // the path of the command to run
    pub(crate) argv: Vec<String>, // holds command line arguments
    pub(crate) dir: PathBuf,      // specifies the working directory of the process
    pub(crate) limits: Limits,
    pub(crate) namespaces: Namespaces,
    pub(crate) uid_mappings: IDMap, // User ID mappings for user namespaces
    pub(crate) gid_mappings: IDMap, // Group ID mappings for user namespaces
}

impl Executor {
    pub fn new<T: AsRef<str>>(prog: &str, argv: &[T]) -> Self {
        Executor {
            prog: prog.to_string(),
            argv: argv.iter().map(|arg| String::from(arg.as_ref())).collect(),
            uid_mappings: IDMap {
                container_id: 0,
                host_id: u32::from(unistd::Uid::current()),
                size: 1,
            },
            gid_mappings: IDMap {
                container_id: 0,
                host_id: u32::from(unistd::Gid::current()),
                size: 1,
            },
            ..Default::default()
        }
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn limits(&mut self, limits: Limits) -> &mut Self {
        self.limits = limits;
        self
    }

    pub fn namespaces(&mut self, namespaces: Namespaces) -> &mut Self {
        self.namespaces = namespaces;
        self
    }

    pub fn run(&mut self) -> ExecutorResult {
        let mut result = ExecutorResult::new();
        self.prog = match FileSystem::find_executable_in_path(&self.prog) {
            Some(path) => match path.to_str() {
                Some(path) => path.to_string(),
                None => unreachable!(),
            },
            None => {
                let err = format!("{}: command not found", self.prog);
                return Self::set_result_with_failure(result, &err, ECODE_COMMAND_NOT_FOUND);
            }
        };

        result.start_time = Some(Instant::now());
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => match wait::waitpid(child, None) {
                Ok(ws) => Self::set_result(result, Some(ws)),
                Err(err) => {
                    let err = err.to_string();
                    Self::set_result_with_failure(result, &err, ECODE_FAILURE)
                }
            },
            Ok(ForkResult::Child) => match ChildProcess::run(self) {
                Ok(_) => unreachable!(),
                Err(err) => {
                    let err = format!("hakoniwa: {}\n", err);
                    unistd::write(STDERR_FILENO, err.as_bytes()).ok();
                    process::exit(ECODE_CANNOT_EXECUTE);
                }
            },
            Err(err) => {
                let err = err.to_string();
                Self::set_result_with_failure(result, &err, ECODE_CANNOT_EXECUTE)
            }
        }
    }

    fn set_result(mut result: ExecutorResult, ws: Option<WaitStatus>) -> ExecutorResult {
        if let Some(ws) = ws {
            match ws {
                WaitStatus::Exited(_, exit_status) => {
                    result.status = Status::Ok;
                    result.reason = String::new();
                    result.exit_code = Some(exit_status);
                }
                WaitStatus::Signaled(_, signal, _) => {
                    result.status = match signal {
                        Signal::SIGKILL => Status::TimeLimitExceeded,
                        Signal::SIGXCPU => Status::TimeLimitExceeded,
                        Signal::SIGXFSZ => Status::OutputLimitExceeded,
                        Signal::SIGSYS => Status::Violation,
                        _ => Status::Signaled,
                    };
                    result.reason = format!("signaled: {}", signal);
                    result.exit_code = Some(128 + (signal as i32));
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

    fn set_result_with_failure(
        mut result: ExecutorResult,
        reason: &str,
        exit_code: ExitCode,
    ) -> ExecutorResult {
        result.status = Status::SandboxFailure;
        result.reason = reason.to_string();
        result.exit_code = Some(exit_code);
        Self::set_result(result, None)
    }
}
