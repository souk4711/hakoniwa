# Howto - Run simple X11 application


## xterm

First create a custom policy configuration named `policy.toml` for X11 application:

```toml
mounts = [
  { source = "/bin"  , target = "/bin"  },
  { source = "/lib"  , target = "/lib"  },
  { source = "/lib64", target = "/lib64"},
  { source = "/usr"  , target = "/usr"  },
  { source = "/dev"  , target = "/dev"  },
  { source = "/tmp/.X11-unix", target = "/tmp/.X11-unix", rw = true },
  { source = {{ os_homedir "/.Xauthority" }}, target = {{ os_homedir "/.Xauthority" }} },
]

[env]
DISPLAY    = {{ os_env "DISPLAY"    }}
XAUTHORITY = {{ os_env "XAUTHORITY" }}
HOME       = {{ os_env "HOME"       }}
```

Then run:

```console
$ hakoniwa run --policy-file ./policy.toml -- /usr/bin/xterm
```

If you get an error message `Authorization required, but no authorization
protocol specified`. Try this to fix it:

```console
# To provide access to an application to the graphical server. Use `xhost -` to get things back to normal.
$ xhost +
```

More examples can be found in [hakoniwa-cli/examples](./).
