# Hakoniwa

Process isolation for Linux using namespaces, resource limits and seccomp. It
works by creating a new, completely empty, mount namespace where the root is
on a tmpfs that is invisible from the host, and will be automatically cleaned
up when the last process exits.


## Installation

### Cargo

* Install libseccomp by following [this][Install libseccomp] guide.
* Install the rust toolchain in order to have cargo installed by following
  [this][Install Rust] guide.
* Run `cargo install hakoniwa-cli`.


## Acknowledgements

* Special thanks to [bubblewrap].


## License

The CLI is licensed under the GPL-3.0-only.
The Library is licensed under the LGPL-3.0-linking-exception.


[Install libseccomp]:https://github.com/libseccomp-rs/libseccomp-rs#requirements
[Install Rust]:https://www.rust-lang.org/tools/install
[bubblewrap]:https://github.com/containers/bubblewrap
