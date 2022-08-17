use hakoniwa::{Error, ExecutorResultStatus, Sandbox, SandboxPolicy, Stdio};

fn main() -> Result<(), Error> {
    let policy = SandboxPolicy::from_str(
        r#"
mount_new_tmpfs = true
mount_new_devfs = true
mounts = [
  { source = "/bin"  , target = "/bin"  },
  { source = "/lib"  , target = "/lib"  },
  { source = "/lib64", target = "/lib64"},
  { source = "/usr"  , target = "/usr"  },
]

[env]
TERM = {{ os_env "TERM" }}
    "#,
    )?;

    let mut sandbox = Sandbox::new();
    sandbox.with_policy(policy);

    // Capture stdout into Executor#stdout_data.
    let prog = "echo";
    let argv = vec![prog, "Hako!"];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor.run();
    assert_eq!(result.status, ExecutorResultStatus::Ok);
    assert_eq!(String::from_utf8_lossy(executor.stdout_data()), "Hako!\n");
    assert_eq!(String::from_utf8_lossy(executor.stderr_data()), "");

    // Inherit stdout from parent.
    let prog = "echo";
    let argv = vec![prog, "Hako!"];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor.stdout(Stdio::inherit_stdout()).run();
    assert_eq!(result.status, ExecutorResultStatus::Ok);
    assert_eq!(String::from_utf8_lossy(executor.stdout_data()), "");
    assert_eq!(String::from_utf8_lossy(executor.stderr_data()), "");

    // Capture stderr into Executor#stderr_data.
    let prog = "command404";
    let argv = vec![prog];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor.run();
    assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
    assert_eq!(String::from_utf8_lossy(executor.stdout_data()), "");
    assert!(String::from_utf8_lossy(executor.stderr_data()).contains("command not found"));

    // Inherit stderr from parent.
    let prog = "command404";
    let argv = vec![prog];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor.stderr(Stdio::inherit_stderr()).run();
    assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
    assert_eq!(String::from_utf8_lossy(executor.stdout_data()), "");
    assert_eq!(String::from_utf8_lossy(executor.stderr_data()), "");

    Ok(())
}
