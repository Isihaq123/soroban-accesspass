.PHONY: build test fmt fmt-check lint clean deploy-testnet

# Detect the host triple so tests compile to native (not wasm).
HOST := $(shell rustc -vV 2>/dev/null | grep "^host:" | cut -d' ' -f2)

# ── Contract build (wasm) ─────────────────────────────────────────────────
build:
	stellar contract build

# ── Unit tests (native host) ──────────────────────────────────────────────
# Tests must compile to the host target because they link against std.
# The .cargo/config.toml alias sets the default to wasm32, so we must
# explicitly override it here with --target $(HOST).
test:
	cargo test --target $(HOST)

# ── Format ─────────────────────────────────────────────────────────────────
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# ── Lint ───────────────────────────────────────────────────────────────────
lint:
	cargo clippy --target $(HOST) --all-targets -- -D warnings

# ── Clean ──────────────────────────────────────────────────────────────────
clean:
	cargo clean

# ── Deploy to Testnet ──────────────────────────────────────────────────────
# Usage: make deploy-testnet IDENTITY=alice
IDENTITY ?= alice
NETWORK   ?= testnet
WASM      := target/wasm32-unknown-unknown/release/soroban_accesspass.wasm

deploy-testnet: build
	stellar contract deploy \
		--wasm $(WASM) \
		--source $(IDENTITY) \
		--network $(NETWORK)
