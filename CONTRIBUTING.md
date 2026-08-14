# Contributing to Soroban AccessPass

Thank you for your interest in contributing. This document explains how to get
involved, submit changes, and maintain the quality bar expected of a
production-grade security primitive.

---

## Getting Started

### Prerequisites

| Tool | Install |
|---|---|
| Rust (stable) | `rustup toolchain install stable` |
| `wasm32` target | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | `cargo install --locked stellar-cli --features opt` |
| `cargo-expand` (optional) | `cargo install cargo-expand` |

### First build

```bash
git clone https://github.com/abdulazizishaq212-prog/soroban-accesspass.git
cd soroban-accesspass
make build   # wasm build
make test    # unit tests
make lint    # clippy
```

---

## Contribution Workflow

1. **Open an issue first** for non-trivial changes so the approach can be
   agreed before you invest time writing code.
2. Fork the repository and create a branch: `feat/my-feature` or `fix/my-bug`.
3. Make your changes following the coding standards below.
4. Run the full verification suite locally:

   ```bash
   make fmt-check
   make lint
   make test
   make build
   ```

5. Update `CHANGELOG.md` under `[Unreleased]`.
6. Open a pull request using the provided template.

---

## Coding Standards

- All `pub fn` entries must have `///` documentation including `# Arguments`
  and `# Panics` sections where applicable.
- Every new storage key must be added to the `DataKey` enum with a doc comment.
- Every state-mutating function must emit at least one on-ledger event.
- Every persistent storage write must be accompanied by `extend_ttl`.
- Every persistent storage read must bump TTL if the entry is found.
- Panic messages must start with `"AccessPass: "` and go through a dedicated
  `err_*` helper function.
- New functions must have corresponding unit tests covering the happy path,
  at least one negative (panic) case, and event emission.

---

## Security Issues

**Do not open a public GitHub issue for security vulnerabilities.**
Please follow the process described in [SECURITY.md](SECURITY.md).

---

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
All participants are expected to uphold its standards.
