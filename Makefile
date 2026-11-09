.PHONY: all build clean daemon module

all: build
build: daemon module

daemon:
	cargo build --locked --release --manifest-path daemon/Cargo.toml

module:
	$(MAKE) -C module

clean:
	cargo clean --manifest-path daemon/Cargo.toml
	$(MAKE) -C module clean
