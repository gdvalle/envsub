PREFIX?=/usr/local
BIN_PATH=target/$(TARGET)/release/envsub
TARGET?=x86_64-unknown-linux-musl
RUST_DISTRIBUTION?=stable

all: test build compress

.PHONY: build
build:
	cargo build --release --target=$(TARGET)

.PHONY: test
test:
	cargo test

.PHONY: compress
compress: build
	strip -s $(BIN_PATH)
	upx --brute $(BIN_PATH)

.PHONY: bootstrap
bootstrap:
	curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$(RUST_DISTRIBUTION)
	$(HOME)/.cargo/bin/rustup target add $(TARGET)

.PHONY: install
install:
	install -D -m 0755 -t $(DESTDIR)$(PREFIX)/bin $(BIN_PATH)

.PHONY: clean
clean:
	rm -r target
