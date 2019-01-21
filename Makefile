docker-built := .dev/docker-built
docker-dev-image := dev/pm

help:
	@echo
	@echo "Available tasks:"
	@echo "    shell: Run a bash shell inside a Rust Docker container."
	@echo

$(docker-built): Dockerfile
	docker build $(maybe-no-cache) --tag $(docker-dev-image) --target build .
	touch $@

shell: $(docker-built)
	mkdir -p .dev/cargo-registry
	xauth nlist :0 | sed -e 's/^..../ffff/' | xauth -f .dev/xauthority nmerge -
	docker run -it --rm -w /workdir -v $$(pwd)/pm:/workdir \
	-e DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix -e XAUTHORITY=/xauthority -v $$(pwd)/.dev/xauthority:/xauthority \
	-v $$(pwd)/.dev/cargo-registry:/root/.cargo/registry \
	$(docker-dev-image) bash
