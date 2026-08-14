# Soroban AccessPass — Drips Wave Maintainer Plan

## Overview

`soroban-accesspass` is a modular RBAC primitive for Soroban smart contracts.
Its composable design makes it ideal for Wave: every Soroban protocol needing
access control — DAOs, AMMs, bridges, RWA platforms — can import it directly.
The architecture produces a natural stream of scoped, self-contained issues
across four work types.

---

## Issue Types

### 1. Bug Fixes
Small, well-scoped, good for first-time contributors.

- **Fix upstream CI** — apply patched `soroban-env-host` once Stellar resolves
  the `ed25519-dalek 3.0.0 / rand_core` incompatibility
- **TTL consistency** — confirm `get_role_expiry` TTL behaviour on non-existent keys
- **Event idempotency** — verify no spurious events on no-op revoke paths

### 2. New Features
Core contract extensions, medium-to-hard complexity.

- **Role expiry extension** — `extend_role_expiry()` without full revoke/re-grant
- **Bulk grant/revoke** — batch operations to reduce transaction overhead
- **Role enumeration** — `get_roles_for_user()` view function with indexed storage
- **Timelock module** — configurable delay between grant and activation
- **Multi-sig admin guard** — M-of-N threshold for admin operations
- **Role hierarchy** — parent-child role inheritance

### 3. Testing
Improves confidence and coverage, easy-to-medium difficulty.

- **Proptest fuzzing** — fuzz role-key derivation and expiry boundary logic
- **TTL archival tests** — advance ledger to simulate archival, verify reads still bump TTL
- **Auth enforcement without mock_all_auths** — use explicit `mock_auths` entries
- **Event body assertions** — assert full topic vectors and body values

### 4. Documentation & Tooling
High visibility, easy wins for contributors new to Soroban.

- **Composability guide** — how to embed AccessPass into an AMM, DAO, or bridge
- **Sequence diagrams** — Mermaid diagrams for role-grant, delegation, admin transfer flows
- **Integration example** — `examples/token-with-access` contract using AccessPass as a mint guard
- **Testnet deploy script** — complete `scripts/deploy-and-invoke.sh` with contract ID persistence
- **cargo-audit CI step** — automated vulnerability scanning on every push

---

## Sprint Roadmap

| Sprint | Focus |
|---|---|
| 1 | Fix upstream CI, auth tests, sequence diagrams |
| 2 | Proptest fuzzing, TTL tests, composability guide |
| 3 | Role enumeration, expiry extension, bulk operations |
| 4 | Examples contract, testnet scripts, crates.io publish |
| 5+ | Timelock, multi-sig admin, role hierarchy |

---

## Contributor Experience

Every issue will include:
- **Why it matters** — which protocols benefit
- **Acceptance criteria** — exact definition of done
- **Starting point** — file and function to look at first
- **Test skeleton** — a template test to fill in

Maintainer: `abdulazizishaq212@gmail.com`
Repository: https://github.com/abdulazizishaq212-prog/soroban-accesspass
