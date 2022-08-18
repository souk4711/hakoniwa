use hakoniwa::{Error, ExecutorResultStatus, Sandbox, SandboxPolicy};

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

    // Killed in 2s.
    let prog = "sleep";
    let argv = vec![prog, "5"];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor
        .limit_as(Some(16_000_000)) // 16MB
        .limit_core(Some(0)) // no core file
        .limit_cpu(None) // inherit from parent
        .limit_fsize(Some(0)) // no output file
        .limit_nofile(Some(32)) // 32 max fd
        .limit_walltime(Some(2)) // 2 seconds
        .run();
    assert_eq!(result.status, ExecutorResultStatus::TimeLimitExceeded);
    assert_eq!(result.exit_code, Some(128 + libc::SIGKILL));

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
