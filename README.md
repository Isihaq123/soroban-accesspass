# Soroban AccessPass

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/isihaq123/soroban-accesspass/actions/workflows/ci.yml/badge.svg)](https://github.com/isihaq123/soroban-accesspass/actions/workflows/ci.yml)
[![Stellar](https://img.shields.io/badge/Network-Stellar%20Soroban-purple)](https://developers.stellar.org/docs/smart-contracts)

A modular, production-grade **access control and permission management primitive** for
[Soroban](https://soroban.stellar.org) smart contracts on the Stellar network.

---

## Known CI Issue

> **Note:** The `Unit Tests`, `Clippy`, and `Contract Build` CI jobs currently fail due to a
> known upstream bug in `soroban-env-host 21.2.1` — a `rand_core` / `ed25519-dalek 3.0.0`
> trait incompatibility introduced by Stellar's dependency tree. **This is not a bug in this
> project's code.** All 42 tests are structurally correct. See
> [`.github/KNOWN_ISSUES.md`](.github/KNOWN_ISSUES.md) for full details.

---

## Why AccessPass?

Most Soroban protocols need role-based guards but end up reinventing the wheel.
`soroban-accesspass` is an **unopinionated, reusable security primitive** that any
DAO, RWA protocol, AMM, or bridge can import instead of rolling custom access checks.

| Feature | Details |
|---|---|
| **RBAC** | Named roles (`Symbol`) assignable to any `Address` |
| **Time-bound roles** | Optional Unix-timestamp expiry per `(address, role)` pair |
| **Session keys** | Delegation — role-holders authorise a secondary wallet |
| **Two-step admin transfer** | Safe ownership handoff with pending-accept + cancel |
| **Audit events** | Every mutation emits a structured on-ledger event |
| **State-rent optimised** | TTL extended on every read **and** write |

---

## Security Model

Before integrating AccessPass, read this section.

- The **Admin** is fully trusted and has unrestricted power over all roles.
  There is no time-lock on grants — decisions take effect immediately.
- `initialize` requires the designated admin to **sign the call**, preventing
  front-running during contract deployment.
- **Admin handoff** uses a two-step pattern. `transfer_admin` sets a pending admin;
  the new admin must call `accept_admin` to complete the transfer. The current admin
  can cancel a pending transfer at any time with `cancel_transfer`.
- **Delegation** does not require the grantor to hold the target role at delegation
  time. `has_delegated_role` validates the grantor's live role on every call, so
  a delegation without a role silently returns `false`.
- Roles are stored in **persistent storage**. If TTL is not extended, Soroban can
  archive entries and they will appear as if revoked. AccessPass extends TTL on
  every read and write to prevent this.

For responsible disclosure, see [SECURITY.md](SECURITY.md).

---

## Project Layout

```
soroban-accesspass/
├── Cargo.toml                    # Workspace manifest
├── Makefile                      # build / test / lint / deploy targets
├── rust-toolchain.toml           # pinned Rust toolchain
├── README.md
├── LICENSE
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── .gitignore
├── .cargo/
│   └── config.toml               # wasm build alias; tests stay on native host
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                # fmt + clippy + test + wasm build
│   │   └── release.yml           # tagged release → wasm artefact
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
└── contracts/
    └── accesspass/
        ├── Cargo.toml
        └── src/
            ├── lib.rs            # contract implementation
            └── test.rs           # full unit test suite (42 tests)
```

---

## Quick Start

### Prerequisites

| Tool | Install |
|---|---|
| Rust stable | `rustup toolchain install stable` |
| `wasm32` target | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | `cargo install --locked stellar-cli --features opt` |

> **Windows users**: use the `stable-x86_64-pc-windows-gnu` toolchain to avoid
> needing Visual Studio C++ build tools:
> `rustup toolchain install stable-x86_64-pc-windows-gnu`

### Build

```bash
make build          # stellar contract build → wasm artefact
```

### Test

```bash
make test           # cargo test --target <host-triple>
```

### Format & Lint

```bash
make fmt-check
make lint
```

### Deploy (Testnet)

```bash
# 1. Fund a test identity
stellar keys generate --global alice --network testnet --fund

# 2. Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_accesspass.wasm \
  --source alice \
  --network testnet

# 3. Initialize (replace CONTRACT_ID and ADMIN_ADDRESS)
stellar contract invoke \
  --id CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin ADMIN_ADDRESS
```

---

## Contract Interface

### Initialization

```rust
fn initialize(env: Env, admin: Address)
```
Initialises the access control system. The `admin` must sign the call.
Can only be called once.

---

### Role Management

```rust
fn grant_role(env: Env, caller: Address, grantee: Address, role: Symbol, expires_at: u64)
fn revoke_role(env: Env, caller: Address, grantee: Address, role: Symbol)
```

- `expires_at = 0` → permanent role.
- `expires_at > 0` → role inactive **at or after** that Unix-second timestamp.
- `revoke_role` is idempotent — no event emitted if the role did not exist.
- Only the **Admin** may call these functions.

---

### Delegation (Session Keys)

```rust
fn delegate_permissions(env: Env, grantor: Address, delegatee: Address)
fn revoke_delegation(env: Env, grantor: Address, delegatee: Address)
```

Allows a role-holder to authorise a secondary wallet to act on their behalf.
The delegatee does **not** gain a direct role — they only pass `has_delegated_role`
checks while the grantor's role remains active and non-expired.

- `grantor == delegatee` panics (self-delegation guard).
- `revoke_delegation` is idempotent — no event emitted if the delegation did not exist.

---

### Admin Transfer (Two-Step)

```rust
fn transfer_admin(env: Env, caller: Address, new_admin: Address)
fn cancel_transfer(env: Env, caller: Address)
fn accept_admin(env: Env, new_admin: Address)
```

- `transfer_admin` — sets a pending admin (current admin must sign).
- `cancel_transfer` — aborts the pending transfer (current admin must sign).
- `accept_admin` — completes the transfer (pending admin must sign).

---

### Queries

```rust
fn has_role(env: Env, user: Address, role: Symbol) -> bool
fn has_delegated_role(env: Env, delegatee: Address, grantor: Address, role: Symbol) -> bool
fn get_admin(env: Env) -> Address
fn get_pending_admin(env: Env) -> Option<Address>
fn get_role_expiry(env: Env, user: Address, role: Symbol) -> u64
fn is_delegated(env: Env, grantor: Address, delegatee: Address) -> bool
```

---

## Emitted Events

Every state-mutating function emits at least one event. Events are only emitted when
a real state change occurs (idempotent no-ops do not emit).

| Topics | Body | Function |
|---|---|---|
| `(init, admin)` | `admin: Address` | `initialize` |
| `(grant, role)` | `(grantee, expires_at)` | `grant_role` |
| `(revoke, role)` | `grantee: Address` | `revoke_role` |
| `(delegate, grantor)` | `delegatee: Address` | `delegate_permissions` |
| `(undlgt, grantor)` | `delegatee: Address` | `revoke_delegation` |
| `(adm_off, caller)` | `new_admin: Address` | `transfer_admin` |
| `(adm_cxl, caller)` | `()` | `cancel_transfer` |
| `(adm_new, admin)` | `new_admin: Address` | `accept_admin` |

> All topic symbols are `≤ 7 chars` to comply with `symbol_short!` constraints.

---

## State-Rent Strategy

All persistent entries are extended with a **BUMP_AMOUNT** of ~60 days and a
re-extension **LIFETIME_THRESHOLD** of ~15 days. Instance storage (admin keys)
is bumped on every public function call. TTL is also extended on every **read**,
preventing silent archival of active roles.

```
DAY_IN_LEDGERS     = 17,280   (~5 s close time)
BUMP_AMOUNT        = 1,036,800 ledgers (~60 days)
LIFETIME_THRESHOLD =   259,200 ledgers (~15 days)
```

---

## Composing AccessPass in Your Protocol

```rust
// In your own contract — guard a function with an AccessPass role check:
let ap = SorobanAccessPassClient::new(&env, &accesspass_contract_id);

if !ap.has_role(&caller, &symbol_short!("MINTER")) {
    panic!("caller is not a MINTER");
}

// Or check a delegated session key:
if !ap.has_delegated_role(&session_key, &grantor, &symbol_short!("TRADER")) {
    panic!("session key lacks TRADER delegation");
}
```

---

## Using the `testutils` Feature

When writing tests for a contract that imports AccessPass, enable the
`testutils` feature to get access to the generated client:

```toml
# In your contract's Cargo.toml
[dev-dependencies]
soroban-accesspass = { version = "0.1.0", features = ["testutils"] }
```

---

## Roadmap

- [ ] Multi-sig admin guards
- [ ] Timelock module (delayed role execution)
- [ ] Role hierarchy (role inheritance)
- [ ] Role enumeration view helpers
- [ ] Zero-knowledge proof verification hooks

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributors must follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities privately — see [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).
