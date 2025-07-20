# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_hakoniwa_global_optspecs
	string join \n v/verbose q/quiet h/help V/version
end

function __fish_hakoniwa_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_hakoniwa_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_hakoniwa_using_subcommand
	set -l cmd (__fish_hakoniwa_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c hakoniwa -n "__fish_hakoniwa_needs_command" -s v -l verbose -d 'Increase logging verbosity'
complete -c hakoniwa -n "__fish_hakoniwa_needs_command" -s q -l quiet -d 'Decrease logging verbosity'
complete -c hakoniwa -n "__fish_hakoniwa_needs_command" -s h -l help -d 'Print help'
complete -c hakoniwa -n "__fish_hakoniwa_needs_command" -s V -l version -d 'Print version'
complete -c hakoniwa -n "__fish_hakoniwa_needs_command" -f -a "completion" -d 'Generate SHELL autocompletions'
complete -c hakoniwa -n "__fish_hakoniwa_needs_command" -f -a "run" -d 'Run a COMMAND in a container'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand completion" -s f -l file -d 'Output the completion to file rather than stdout' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand completion" -s v -l verbose -d 'Increase logging verbosity'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand completion" -s q -l quiet -d 'Decrease logging verbosity'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand completion" -s h -l help -d 'Print help'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l rootdir -d 'Use ROOTDIR as the mount point for the container root fs' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l rootfs -d 'Bind mount all subdirectories in ROOTFS to the container root with read-only access' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s b -l bindmount-ro -d 'Bind mount the HOST_PATH on CONTAINER_PATH with read-only access (repeatable)' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s B -l bindmount-rw -d 'Bind mount the HOST_PATH on CONTAINER_PATH with read-write access (repeatable)' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l devfs -d 'Mount new devfs on CONTAINER_PATH (repeatable)' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l tmpfs -d 'Mount new tmpfs on CONTAINER_PATH (repeatable)' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l dir -d 'Create a new dir on CONTAINER_PATH with 700 permissions (repeatable)' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l symlink -d 'Create a symbolic link on LINK_PATH pointing to the ORIGINAL_PATH (repeatable)' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l userns -d 'Configure user namespace for the container' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s u -l uidmap -d 'UID map to use for the user namespace (repeatable)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s g -l gidmap -d 'GID map to use for the user namespace (repeatable)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l user -d 'Set user/group for the container' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l hostname -d 'Set hostname for the container (implies --unshare-uts)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l network -d 'Set network mode for the container' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s e -l setenv -d 'Set an environment variable (repeatable)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s w -l workdir -d 'Bind mount the HOST_PATH on the same container path with read-write access, then run COMMAND inside it' -r -f -a "(__fish_complete_directories)"
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l limit-as -d 'Limit the maximum size of the COMMAND\'s virtual memory' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l limit-core -d 'Limit the maximum size of a core file in bytes that the COMMAND may dump' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l limit-cpu -d 'Limit the amount of CPU time that the COMMAND can consume, in seconds' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l limit-fsize -d 'Limit the maximum size in bytes of files that the COMMAND may create' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l limit-nofile -d 'Limit the maximum file descriptor number that can be opened by the COMMAND' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l limit-walltime -d 'Limit the amount of wall time that the COMMAND can consume, in seconds' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l landlock-restrict -d 'Restrict ambient rights (e.g. global filesystem access) for the process' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l landlock-fs-ro -d 'Allow to read files beneath PATH (implies --landlock-restrict=fs)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l landlock-fs-rw -d 'Allow to read-write files beneath PATH (implies --landlock-restrict=fs)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l landlock-fs-rx -d 'Allow to execute files beneath PATH (implies --landlock-restrict=fs)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l landlock-tcp-bind -d 'Allow binding a TCP socket to a local port (implies --landlock-restrict=tcp.bind)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l landlock-tcp-connect -d 'Allow connecting an active TCP socket to a remote port (implies --landlock-restrict=tcp.connect)' -r
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l seccomp -d 'Set the seccomp security profile' -r -F
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s c -l config -d 'Load configuration from a specified file, ignoring all other cli arguments' -r -F
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l unshare-all -d 'Create new CGROUP, IPC, NETWORK, UTS, ... namespaces'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l unshare-cgroup -d 'Create new CGROUP namespace'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l unshare-ipc -d 'Create new IPC namespace'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l unshare-network -d 'Create new NETWORK namespace'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l unshare-uts -d 'Create new UTS namespace'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -l allow-new-privs -d 'Set the NoNewPrivileges flag to off'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s v -l verbose -d 'Increase logging verbosity'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s q -l quiet -d 'Decrease logging verbosity'
complete -c hakoniwa -n "__fish_hakoniwa_using_subcommand run" -s h -l help -d 'Print help'
