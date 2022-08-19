use hakoniwa::{Error, ExecutorResultStatus, Sandbox, SandboxPolicy, Stdio};

fn main() -> Result<(), Error> {
    let mut sandbox = Sandbox::new();
    sandbox.with_policy(SandboxPolicy::from_str(
        r#"
mounts = [
  { source = "/bin"  , target = "/bin"  },
  { source = "/lib"  , target = "/lib"  },
  { source = "/lib64", target = "/lib64"},
  { source = "/usr"  , target = "/usr"  },
]
    "#,
    )?);

    // Capture stdout/stderr into ExecutorResult#stdout/stderr.
    let prog = "command404";
    let argv = vec![prog];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor.run();
    assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
    assert_eq!(result.exit_code, None);
    assert_eq!(String::from_utf8_lossy(&result.stdout), "");
    assert!(String::from_utf8_lossy(&result.stderr).contains("command not found"));

    // Inherit stdout/stderr from parent.
    let prog = "command404";
    let argv = vec![prog];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .run();
    assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
    assert_eq!(result.exit_code, None);
    assert_eq!(String::from_utf8_lossy(&result.stdout), "");
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
