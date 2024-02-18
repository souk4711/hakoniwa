.PHONY: devcontainer prodcontainer test

devcontainer:
	docker build -f ./Dockerfile.dev . -t hakoniwa-devcontainer:latest

prodcontainer: devcontainer
	docker build -f ./Dockerfile.prod . -t hakoniwa-prodcontainer:latest

test: devcontainer
	docker run --privileged --rm -it hakoniwa-devcontainer:latest cargo test
