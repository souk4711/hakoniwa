# Hakoniwa

Process isolation for Linux using namespaces, resource limits and seccomp. It
works by creating a new, completely empty, mount namespace where the root is
on a tmpfs that is invisible from the host, and will be automatically cleaned
up when the last process exits. You can then use a policy configuration file or
commandline options to construct the root filesystem and process environment
and command to run in the namespace.


## Installation

### Cargo

  * Install the rust toolchain in order to have cargo installed by following
    [this][Install Rust] guide.
  * Run `cargo install hakoniwa-cli`.


## Usage

### CLI

### Rust Library


## Acknowledgements

  * Special thanks to [bubblewrap].


## License

Licensed under either of

  * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


[Install Rust]:https://www.rust-lang.org/tools/install
[bubblewrap]:https://github.com/containers/bubblewrap
