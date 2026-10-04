.PHONY: all build clean daemon module

all: build
build: daemon module

daemon:
	cargo build --locked --release --manifest-path src/daemon/Cargo.toml

module:
	$(MAKE) -C src/module

clean:
	cargo clean --manifest-path src/daemon/Cargo.toml
	$(MAKE) -C src/module clean
