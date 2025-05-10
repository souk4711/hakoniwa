#!/usr/bin/env bash


set -euo pipefail


echo_info() {
  echo -e "\e[1;34mINFO: $* \e[m"
}

echo_warn() {
  echo -e "\e[1;33mWARN: $* \e[m"
}

echo_error() {
  echo -e "\e[1;31mERROR: $* \e[m"
}


main() {
  cd "$(dirname -- "$0")/.."

  echo_info "Generating SHELL autocompletions for bash..."
  cargo run completion bash -f ./etc/bash/completions/hakoniwa

  echo_info "Generating SHELL autocompletions for fish..."
  cargo run completion fish -f ./etc/fish/vendor_completions.d/hakoniwa.fish

  echo_info "Generating SHELL autocompletions for zsh..."
  cargo run completion zsh -f ./etc/zsh/site-functions/_hakoniwa
}
main "$@"
