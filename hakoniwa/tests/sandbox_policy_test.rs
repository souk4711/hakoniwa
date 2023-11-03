#[cfg(test)]
mod sandbox_policy_test {
    mod kiss_policy_toml {
        use hakoniwa::{ExecutorResultStatus, Sandbox, SandboxPolicy};
        use rust_embed::RustEmbed;

        #[derive(RustEmbed)]
        #[folder = "../hakoniwa-cli/src/embed"]
        pub struct Embed;

        fn sandbox() -> Sandbox {
            let mut sandbox = Sandbox::new();
            sandbox.with_policy(
                SandboxPolicy::from_str(
                    std::str::from_utf8(&Embed::get("KISS-policy.toml").unwrap().data).unwrap(),
                )
                .unwrap(),
            );
            sandbox
        }

        #[test]
        fn test_mounts_proc_mounted() {
            let mut executor = sandbox().command("ls", &["ls", "-la", "/proc/1/exe"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
            assert!(String::from_utf8_lossy(&result.stdout).contains("/bin/ls"));
        }

        #[test]
        fn test_mounts_proc_flags() {
            let mut executor = sandbox().command("findmnt", &["findmnt", "-n", "-T", "/proc"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
            assert!(String::from_utf8_lossy(&result.stdout).contains("rw,nosuid,nodev,noexec"));
        }

        #[test]
        fn test_mounts_dev_mounted() {
            let mut executor = sandbox().command("ls", &["ls", "/dev"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
            assert_eq!(
                String::from_utf8_lossy(&result.stdout),
                "null\nrandom\nurandom\nzero\n"
            );
        }

        #[test]
        fn test_mounts_dev_flags() {
            let mut executor = sandbox().command("findmnt", &["findmnt", "-n", "-T", "/dev/null"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
            assert!(String::from_utf8_lossy(&result.stdout).contains("rw,"));
        }

        #[test]
        fn test_mounts_lib_mounted() {
            let mut executor = sandbox().command("ls", &["ls", "/lib"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
        }

        #[test]
        fn test_mounts_lib_flags() {
            let mut executor = sandbox().command("findmnt", &["findmnt", "-n", "-T", "/lib"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
            assert!(String::from_utf8_lossy(&result.stdout).contains("ro,nosuid,"));
        }

        #[test]
        fn test_env() {
            let mut executor = sandbox().command("env", &["env"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
            assert!(String::from_utf8_lossy(&result.stdout).contains("TERM"));
        }
    }

    mod examples_policies {
        #[derive(rust_embed::RustEmbed)]
        #[folder = "../hakoniwa-cli/examples"]
        #[include = "**/*.toml"]
        pub struct Embed;

        use hakoniwa::SandboxPolicy;

        #[test]
        fn test_policy_format_validation() {
            let mut counter = 0;
            for filepath in Embed::iter() {
                let file = Embed::get(&filepath).unwrap();
                let data = std::str::from_utf8(&file.data).unwrap();
                let result = SandboxPolicy::from_str(data);
                assert!(
                    result.is_ok(),
                    "File {:?}: {:?}",
                    filepath,
                    result.err().unwrap()
                );
                counter += 1;
            }
            assert!(counter > 10);
        }
    }
}
