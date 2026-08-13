# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [0.1.0] — 2025-01-01

### Added
- `initialize(admin)` — deploy access control system with a root Admin.
  Requires the admin to sign, preventing front-running.
- `grant_role(caller, grantee, role, expires_at)` — Admin-only role assignment
  with optional Unix-timestamp expiry (`0` = permanent).
- `revoke_role(caller, grantee, role)` — Admin-only role revocation.
  Idempotent; no event emitted for a non-existent role.
- `delegate_permissions(grantor, delegatee)` — Session-key delegation pattern.
  Guards against self-delegation.
- `revoke_delegation(grantor, delegatee)` — Revoke an active delegation.
  Idempotent; no event emitted for a non-existent delegation.
- `transfer_admin(caller, new_admin)` — Initiate two-step Admin handoff.
- `cancel_transfer(caller)` — Admin can abort a pending handoff.
- `accept_admin(new_admin)` — Complete the two-step Admin handoff.
- `has_role(user, role)` — Direct role check with expiry validation.
- `has_delegated_role(delegatee, grantor, role)` — Delegated role check.
- `get_admin()` — Returns current Admin address.
- `get_pending_admin()` — Returns pending Admin address (or `None`).
- `get_role_expiry(user, role)` — Returns expiry timestamp (`0` = permanent).
- `is_delegated(grantor, delegatee)` — Checks delegation existence.
- Full audit event emission on every state-mutating function.
- TTL extended on every read **and** write to prevent state archival.
- GitHub Actions CI (fmt, clippy, test, wasm build) and release workflow.
- Makefile with `build`, `test`, `fmt`, `lint`, `clean`, `deploy-testnet` targets.

[Unreleased]: https://github.com/soroban-accesspass/soroban-accesspass/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/soroban-accesspass/soroban-accesspass/releases/tag/v0.1.0
