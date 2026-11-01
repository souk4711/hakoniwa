.PHONY: help
help:										## Print help
	@grep -E '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: lint
lint:										## Run lint
	cargo clippy --all-features --all-targets -- -D warnings
	cargo fmt --all -- --check

.PHONY: test
test:										## Run test suites
	cargo test --all-features --all-targets -p hakoniwa -- --nocapture
	cargo test --all-features --doc -p hakoniwa -- --nocapture
	cargo test --all-features --all-targets -p hakoniwa-cli -- --nocapture
	cargo test --all-features --doc -p hakoniwa-cli -- --nocapture

.PHONY: denycheck
denycheck:							## Run deny check
	cargo deny --all-features --log-level error check

.PHONY: hackcheck
hackcheck:							## Run hack check
	cargo hack check --feature-powerset --no-dev-deps

.PHONY: build_completions
build_completions:			## Generate SHELL autocompletions files
	./scripts/make-build-completions.sh
