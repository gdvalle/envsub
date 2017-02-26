PREFIX?=/usr/local
BIN_PATH=target/$(TARGET)/release/envsub
TARGET?=x86_64-unknown-linux-musl
RUST_DISTRIBUTION?=stable

.PHONY: build compress bootstrap install clean

all: test build compress

build:
	cargo build --release --target=$(TARGET)

test:
	echo Hello, %NAME% | NAME=User cargo run -- NAME
	cargo test

compress: build
	strip -s $(BIN_PATH)
	upx --brute $(BIN_PATH)

bootstrap:
	curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$(RUST_DISTRIBUTION)
	$(HOME)/.cargo/bin/rustup target add $(TARGET)

install:
	install -D -m 0755 -t $(DESTDIR)$(PREFIX)/bin $(BIN_PATH)

clean:
	rm -r target
