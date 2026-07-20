.PHONY: all arm64 x64 macos-arm64 macos-x64 clean install

# ── Build targets ──

all: arm64 x64

arm64:
	cargo build --release --target aarch64-unknown-linux-musl
	@mkdir -p dist
	@cp target/aarch64-unknown-linux-musl/release/angles dist/angles-linux-arm64
	@echo "✅ dist/angles-linux-arm64"

x64:
	cargo build --release --target x86_64-unknown-linux-musl
	@mkdir -p dist
	@cp target/x86_64-unknown-linux-musl/release/angles dist/angles-linux-x64
	@echo "✅ dist/angles-linux-x64"

macos-arm64:
	cargo build --release --target aarch64-apple-darwin
	@mkdir -p dist
	@cp target/aarch64-apple-darwin/release/angles dist/angles-macos-arm64
	@echo "✅ dist/angles-macos-arm64"

macos-x64:
	cargo build --release --target x86_64-apple-darwin
	@mkdir -p dist
	@cp target/x86_64-apple-darwin/release/angles dist/angles-macos-x64
	@echo "✅ dist/angles-macos-x64"

# ── Setup cross-compilation toolchains ──

setup-arm64:
	rustup target add aarch64-unknown-linux-musl
	@echo "Install musl cross-compiler:"
	@echo "  Ubuntu/Debian: sudo apt install gcc-aarch64-linux-gnu"
	@echo "  or:            docker run --rm -v $$PWD:/home cimg/rust:1.85 cargo build --release --target aarch64-unknown-linux-musl"

setup-x64:
	rustup target add x86_64-unknown-linux-musl
	@echo "Install musl cross-compiler:"
	@echo "  Ubuntu/Debian: sudo apt install gcc-x86-64-linux-gnu"

setup-macos:
	rustup target add aarch64-apple-darwin
	rustup target add x86_64-apple-darwin
	@echo "macOS targets can only be built on macOS with Xcode"

# ── Docker cross-compile (no local toolchain needed) ──

docker-arm64:
	docker run --rm -v "$(PWD)":/home -w /home \
		ghcr.io/cross-rs/aarch64-unknown-linux-musl:latest \
		cargo build --release --target aarch64-unknown-linux-musl
	@mkdir -p dist
	@cp target/aarch64-unknown-linux-musl/release/angles dist/angles-linux-arm64
	@echo "✅ dist/angles-linux-arm64"

docker-x64:
	docker run --rm -v "$(PWD)":/home -w /home \
		ghcr.io/cross-rs/x86_64-unknown-linux-musl:latest \
		cargo build --release --target x86_64-unknown-linux-musl
	@mkdir -p dist
	@cp target/x86_64-unknown-linux-musl/release/angles dist/angles-linux-x64
	@echo "✅ dist/angles-linux-x64"

# ── Package for release ──

package: all
	@cd dist && \
		tar czf angles-linux-arm64.tar.gz angles-linux-arm64 && \
		tar czf angles-linux-x64.tar.gz angles-linux-x64 && \
		cd ..
	@echo "✅ dist/angles-linux-arm64.tar.gz"
	@echo "✅ dist/angles-linux-x64.tar.gz"

# ── Install locally ──

install:
	cargo install --path .
	@echo "✅ angles installed to ~/.cargo/bin/angles"

clean:
	cargo clean
	rm -rf dist
