# Soroban AccessPass — Drips Wave Maintainer Plan

This document describes the issue backlog strategy for `soroban-accesspass` as a
Drips Wave maintainer. It outlines the types of work that will be posted as scoped
issues across sprint cycles, ensuring a continuous stream of well-defined, contributor-ready
tasks at varying difficulty levels.

---

## Why This Project Is Ideal for Wave

`soroban-accesspass` is a **composable security primitive** — every Soroban protocol
(DAOs, AMMs, RWA platforms, bridges) that needs access control can import it instead of
building their own. This means the issue backlog naturally spans multiple domains:
contract engineering, testing, documentation, tooling, and ecosystem integration.

The modular architecture makes it easy to scope work into **self-contained issues** that
a contributor can pick up, complete, and submit within a single sprint without needing
deep context of the entire codebase.

---

## Issue Categories & Backlog

### 1. New Features (Contract Engineering)

These are the highest-value issues. Each adds a new capability to the primitive.

| Issue | Description | Difficulty |
|---|---|---|
| `feat: multi-sig admin guard` | Require M-of-N signatures to grant/revoke roles. Extends the Admin model to support threshold authorization. | Hard |
| `feat: timelock module` | Add a configurable delay between `grant_role` and when it takes effect. Admin sets a `delay_seconds` per role or globally. | Hard |
| `feat: role hierarchy / inheritance` | Define parent-child role relationships so `SUPERADMIN` implicitly grants `ADMIN` which grants `MINTER`. | Hard |
| `feat: role enumeration` | Add `get_roles_for_user(user)` and `get_users_for_role(role)` view functions backed by an indexed storage structure. | Medium |
| `feat: bulk grant/revoke` | Add `grant_roles_batch` and `revoke_roles_batch` to reduce transaction overhead for protocols assigning multiple roles at once. | Medium |
| `feat: delegation depth limit` | Add a configurable max delegation chain depth to prevent unbounded chains of delegated authority. | Medium |
| `feat: expiry extension` | Add `extend_role_expiry(caller, grantee, role, new_expiry)` so Admin can extend a role without revoking and re-granting. | Easy |
| `feat: zero-knowledge proof hook` | Add an optional `verify_proof` hook interface that protocols can implement to gate role grants with ZK credential checks. | Hard |

---

### 2. Bug Fixes

| Issue | Description | Difficulty |
|---|---|---|
| `fix: upstream soroban-env-host CI` | Track and apply fix once Stellar publishes a patched `soroban-env-host` resolving the `ed25519-dalek 3.0.0 / rand_core` incompatibility. See `.github/KNOWN_ISSUES.md`. | Easy (dependency bump) |
| `fix: revoke_delegation no-op event` | Ensure `revoke_delegation` on a non-existent entry emits no event (currently idempotent but verify no subtle state inconsistency across edge cases). | Easy |
| `fix: TTL not bumped on get_role_expiry for zero value` | When `get_role_expiry` returns `0` (permanent), the expiry key TTL is not extended. If the key was never written (non-existent role), this is a no-op. Confirm behaviour is consistent with the permanent sentinel. | Easy |

---

### 3. Testing

| Issue | Description | Difficulty |
|---|---|---|
| `test: property-based fuzzing with proptest` | Add `proptest` or `quickcheck` fuzz tests for the role-key derivation logic, expiry boundaries, and delegation resolution. | Medium |
| `test: auth enforcement without mock_all_auths` | Replace `mock_all_auths` in auth-enforcement tests with explicit `mock_auths` entries to provide stronger proof that `require_auth` calls are in place. | Medium |
| `test: TTL extension on every read` | Add tests that advance the ledger sequence number to simulate archival thresholds and verify entries are still accessible after TTL-bumping reads. | Medium |
| `test: concurrent admin transfer overwrite` | Verify that initiating a second `transfer_admin` while one is pending correctly overwrites the pending entry and emits the right events. | Easy |
| `test: delegation before role grant then grant` | Verify that pre-emptive delegation (before role is granted) correctly returns `false`, then returns `true` after the role is granted, without requiring a new delegation call. | Easy |
| `test: event topic and body assertions` | Expand event-emission tests to assert both topic vectors and body values using `env.events().all()`. | Easy |

---

### 4. Documentation

| Issue | Description | Difficulty |
|---|---|---|
| `docs: add architecture sequence diagrams` | Create Mermaid sequence diagrams showing the role-grant flow, delegation flow, and two-step admin transfer. Add to `docs/` folder and link from README. | Easy |
| `docs: composability guide` | Write a `docs/COMPOSABILITY.md` showing step-by-step how to embed AccessPass into an AMM, a DAO, and a bridge contract. Include working Rust code snippets. | Medium |
| `docs: testutils feature flag guide` | Document how downstream projects enable the `testutils` feature in their own test suites to get access to the generated client. | Easy |
| `docs: gas cost estimates` | Profile and document approximate instruction counts for each public function using `stellar contract invoke --cost`. | Medium |

---

### 5. Tooling & Developer Experience

| Issue | Description | Difficulty |
|---|---|---|
| `tooling: examples/ integration contract` | Add an `examples/token-with-access` directory containing a minimal ERC-20-style token contract that uses `soroban-accesspass` as its minting guard. Must compile and have tests. | Medium |
| `tooling: testnet deployment script` | Complete `scripts/deploy-and-invoke.sh` with idempotent deployment, contract ID persistence to `.stellar/contract-ids.json`, and full end-to-end invocation of every function. | Easy |
| `tooling: justfile` | Add a `justfile` (using the `just` command runner, popular in the Soroban ecosystem) with all standard targets as an alternative to `make`. | Easy |
| `tooling: cargo-audit CI step` | Add `cargo audit` as a CI job to catch known vulnerabilities in the dependency tree on every push. | Easy |
| `tooling: wasm size badge` | Add a CI step that reports the compiled wasm size as a badge in the README using shields.io. | Easy |

---

### 6. Ecosystem Integration

| Issue | Description | Difficulty |
|---|---|---|
| `ecosystem: OpenZeppelin-style audit checklist` | Create a `docs/AUDIT_CHECKLIST.md` that protocol developers can follow when integrating AccessPass, covering trust assumptions, initialization order, and upgrade paths. | Medium |
| `ecosystem: crates.io publish` | Prepare the crate for `crates.io` publication — verify metadata, run `cargo publish --dry-run`, resolve any issues. | Easy |
| `ecosystem: compatibility matrix` | Document which versions of `soroban-sdk`, `stellar-cli`, and Stellar Protocol are tested and supported. | Easy |

---

## Sprint Cycle Strategy

| Sprint | Focus | Issues |
|---|---|---|
| Sprint 1 | Stabilise CI, fix upstream dependency | `fix: upstream soroban-env-host CI`, `test: auth enforcement`, `docs: sequence diagrams` |
| Sprint 2 | Expand test coverage | `test: proptest fuzzing`, `test: TTL extension`, `test: event assertions` |
| Sprint 3 | High-value features | `feat: role enumeration`, `feat: expiry extension`, `feat: bulk grant/revoke` |
| Sprint 4 | Ecosystem readiness | `tooling: examples/`, `tooling: testnet scripts`, `docs: composability guide`, `ecosystem: crates.io` |
| Sprint 5+ | Advanced features | `feat: timelock module`, `feat: multi-sig admin`, `feat: role hierarchy` |

---

## Contributor Onboarding

Every issue posted will include:

1. **Context** — why the feature/fix matters and which protocols benefit
2. **Acceptance criteria** — exact definition of done (tests required, docs required)
3. **Estimated effort** — hours, not story points
4. **Starting point** — which file and function to look at first
5. **Test template** — a skeleton test to fill in

This ensures contributors can start within minutes, not hours.

---

## Contact

Maintainer: `abdulazizishaq212@gmail.com`
Repository: https://github.com/abdulazizishaq212-prog/soroban-accesspass
