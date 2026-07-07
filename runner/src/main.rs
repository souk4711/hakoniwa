use hakoniwa::cgroups::{Cpu, Memory, Resources};
use hakoniwa::{Container, Rlimit, Runctl};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn read_cgroup_mem_stats(pid: u32) -> serde_json::Value {
    let cgroup_str = match fs::read_to_string(format!("/proc/{pid}/cgroup")) {
        Ok(s) => s,
        Err(_) => return serde_json::json!(null),
    };

    let path = match cgroup_str.lines().find_map(|l| {
        let parts: Vec<&str> = l.splitn(3, ':').collect();
        if parts.len() == 3 && !parts[2].is_empty() {
            Some(parts[2].to_string())
        } else {
            None
        }
    }) {
        Some(p) => p,
        None => return serde_json::json!(null),
    };

    let base = PathBuf::from("/sys/fs/cgroup").join(path.trim_start_matches('/'));

    let read_u64 = |name: &str| -> Option<u64> {
        fs::read_to_string(base.join(name))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
    };

    serde_json::json!({
        "memory_current_bytes": read_u64("memory.current"),
        "memory_peak_bytes": read_u64("memory.peak"),
        "memory_max_bytes": read_u64("memory.max"),
        "swap_current_bytes": read_u64("memory.swap.current"),
        "zswap_current_bytes": read_u64("memory.zswap.current"),
    })
}

fn parse_size(s: &str) -> Result<i64, String> {
    let s = s.trim();
    let mult = if s.ends_with('G') || s.ends_with('g') {
        1_073_741_824i64
    } else if s.ends_with('M') || s.ends_with('m') {
        1_048_576i64
    } else if s.ends_with('K') || s.ends_with('k') {
        1024i64
    } else {
        1i64
    };
    let num_str = s.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.');
    num_str
        .parse::<f64>()
        .map(|v| (v * mult as f64) as i64)
        .map_err(|e| format!("bad size '{}': {}", s, e))
}

fn help() -> ! {
    eprintln!(
        "Usage: hakoniwa-runner [OPTIONS] [--] <command> [args...]

Limit options (applied as rlimit + cgroup where possible):
  --mem-limit <SIZE>      Max virtual memory per process (e.g. 256M, 1G, 512K)
  --mem-cgroup <SIZE>     Hard cgroup memory limit (e.g. 256M, requires cgroups)
  --cpu-limit <SECONDS>   Max CPU time in seconds (rlimit RLIMIT_CPU)
  --wall-limit <SECONDS>  Max wall clock time in seconds
  --nofile-limit <N>      Max open file descriptors
  --cgroup-cpus <QUOTA>   Cgroup CPU quota, e.g. 0.5 = half a core, 2 = two cores
  --cgroup-mem-swap <SZ>  Cgroup memory+swap limit (e.g. 512M)

Output: always prints JSON metrics to stdout after the program exits.
"
    );
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut raw: &[String] = &args[1..];
    if raw.first().map(|s| s == "--").unwrap_or(false) {
        raw = &raw[1..];
    }

    // Parse flags
    let mut mem_limit: Option<u64> = None; // rlimit AS
    let mut mem_cgroup: Option<i64> = None; // cgroup memory
    let mut mem_swap: Option<i64> = None;
    let mut cpu_limit: Option<u64> = None; // rlimit CPU seconds
    let mut wall_limit_sec: Option<f64> = None;
    let mut nofile_limit: Option<u64> = None;
    let mut cpu_quota: Option<i64> = None; // cgroup CPU quota

    let mut i = 0;
    while i < raw.len() {
        let a = &raw[i];
        if a == "--" {
            i += 1;
            break;
        }
        match a.as_str() {
            "--mem-limit" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                mem_limit = Some(parse_size(&raw[i]).unwrap_or_else(|_| help()) as u64);
            }
            "--mem-cgroup" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                mem_cgroup = Some(parse_size(&raw[i]).unwrap_or_else(|_| help()));
            }
            "--cpu-limit" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                cpu_limit = Some(raw[i].parse::<f64>().unwrap_or_else(|_| help()).max(1.0) as u64);
            }
            "--wall-limit" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                wall_limit_sec = Some(raw[i].parse::<f64>().unwrap_or_else(|_| help()).max(0.1));
            }
            "--nofile-limit" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                nofile_limit = Some(raw[i].parse().unwrap_or_else(|_| help()));
            }
            "--cgroup-cpus" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                let val: f64 = raw[i].parse::<f64>().unwrap_or_else(|_| help()).max(0.01);
                let q = (val * 100_000.0) as i64;
                cpu_quota = Some(q);
            }
            "--cgroup-mem-swap" => {
                i += 1;
                if i >= raw.len() {
                    help();
                }
                mem_swap = Some(parse_size(&raw[i]).unwrap_or_else(|_| help()));
            }
            "--help" | "-h" => {
                help();
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {s}");
                help();
            }
            _ => break,
        }
        i += 1;
    }
    let positional = &raw[i..];

    if positional.is_empty() {
        eprintln!("Error: no command specified");
        help();
    }

    let program = &positional[0];
    let cmd_args: Vec<&str> = positional[1..].iter().map(|s| s.as_str()).collect();

    let start = Instant::now();

    let mut container = Container::new();
    container.rootfs("/").expect("rootfs failed");

    // Enable metrics
    container.runctl(Runctl::GetProcPidSmapsRollup);
    container.runctl(Runctl::GetProcPidStatus);

    // rlimit memory (AS limit)
    if let Some(limit) = mem_limit {
        container.setrlimit(Rlimit::As, limit, limit);
    }

    // rlimit CPU
    if let Some(secs) = cpu_limit {
        container.setrlimit(Rlimit::Cpu, secs, secs);
    }

    // rlimit nofile
    if let Some(n) = nofile_limit {
        container.setrlimit(Rlimit::Nofile, n, n);
    }

    // cgroup resources
    let has_cgroup_cpu = cpu_quota.is_some();
    let has_cgroup_mem = mem_cgroup.is_some() || mem_swap.is_some();

    if has_cgroup_cpu || has_cgroup_mem {
        let mut res = Resources::default();

        if let Some(q) = cpu_quota {
            let mut cpu = Cpu::default();
            cpu.quota(q);
            cpu.period(100_000);
            res.cpu(cpu);
        }

        if has_cgroup_mem {
            let mut mem = Memory::default();
            if let Some(m) = mem_cgroup {
                mem.limit(m);
            }
            if let Some(s) = mem_swap {
                mem.swap(s);
            }
            res.memory(mem);
        }

        container.cgroups_resources(res);
    }

    let mut cmd = container.command(program);
    for arg in &cmd_args {
        cmd.arg(arg);
    }

    // Use hakoniwa's built-in wall timeout — it kills via PTRACE and
    // still captures proc_status/smaps_rollup before the internal
    // process exits.
    if let Some(secs) = wall_limit_sec {
        cmd.wait_timeout(secs as u64);
    }

    // Use spawn so we can kill on wall limit (as backup)
    let mut child = cmd.spawn().expect("failed to spawn in sandbox");
    let child_pid = child.id();

    // Read cgroup memory stats while the process is still alive
    let cgroup_mem = read_cgroup_mem_stats(child_pid);

    let output = child.wait_with_output().expect("failed to wait for child");
    let elapsed = start.elapsed();

    let status = output.status;
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    let limited = wall_limit_sec
        .map(|w| elapsed.as_secs_f64() >= w)
        .unwrap_or(false);

    let metrics = serde_json::json!({
        "exit": {
            "code": status.code,
            "reason": status.reason,
            "success": status.success(),
            "exit_code": status.exit_code
        },
        "limits": {
            "mem_limit_kb": mem_limit.map(|v| v / 1024),
            "mem_cgroup_bytes": mem_cgroup,
            "cpu_limit_sec": cpu_limit,
            "wall_limit_sec": wall_limit_sec,
            "nofile_limit": nofile_limit,
            "cgroup_cpu_quota_us": cpu_quota,
            "wall_time_exceeded": limited
        },
        "cgroup_memory": cgroup_mem,
        "internal_rusage": status.rusage.as_ref().map(|r| serde_json::json!({
            "real_time_sec": r.real_time.as_secs_f64(),
            "user_time_sec": r.user_time.as_secs_f64(),
            "system_time_sec": r.system_time.as_secs_f64(),
            "max_rss_kb": r.max_rss
        })),
        "proc_status": status.proc_pid_status.as_ref().map(|s| serde_json::json!({
            "name": s.name,
            "vmpeak_kb": s.vmpeak,
            "vmsize_kb": s.vmsize,
            "vmhwm_kb": s.vmhwm,
            "vmrss_kb": s.vmrss,
            "vmdata_kb": s.vmdata,
            "vmstk_kb": s.vmstk,
            "vmexe_kb": s.vmexe,
            "vmlib_kb": s.vmlib,
            "vmpte_kb": s.vmpte,
            "vmswap_kb": s.vmswap,
            "rssanon_kb": s.rssanon,
            "rssfile_kb": s.rssfile,
            "rssshmem_kb": s.rssshmem
        })),
        "smaps_rollup": status.proc_pid_smaps_rollup.as_ref().map(|s| serde_json::json!({
            "rss_kb": s.rss,
            "shared_clean_kb": s.shared_clean,
            "shared_dirty_kb": s.shared_dirty,
            "private_clean_kb": s.private_clean,
            "private_dirty_kb": s.private_dirty,
            "pss_kb": s.pss,
            "pss_dirty_kb": s.pss_dirty,
            "pss_anon_kb": s.pss_anon,
            "pss_file_kb": s.pss_file,
            "pss_shmem_kb": s.pss_shmem
        })),
        "wall_time_sec": elapsed.as_secs_f64(),
        "stdout": stdout_str.to_string(),
        "stderr": stderr_str.to_string()
    });

    eprintln!("{}", serde_json::to_string_pretty(&metrics).unwrap());

    if limited {
        std::process::exit(124);
    }
    std::process::exit(status.code);
}