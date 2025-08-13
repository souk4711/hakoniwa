.PHONY: help
help:										## Print help
	@grep -E '^[a-z.A-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: test
test:										## Run test suites
	cargo test --all-features --all-targets

.PHONY: build_completions
build_completions:			## Generate SHELL autocompletions files
	./scripts/make-build-completions.sh
