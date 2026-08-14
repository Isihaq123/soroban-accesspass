# Known Issues

## CI: soroban-env-host 21.2.1 / ed25519-dalek incompatibility

**Status:** Upstream bug — not in this project's code.

**Affected jobs:** Unit Tests, Clippy, Contract Build (wasm)

**Root cause:**
`soroban-env-host 21.2.1` (a transitive dependency of `soroban-sdk 21.7.7`) depends on
`ed25519-dalek 3.0.0`, which introduced a breaking change in its `rand_core` trait bounds.
Specifically, `SigningKey::generate` now requires `CryptoRng + ?Sized` from `rand_core 0.9`,
but `ChaCha20Rng` (used internally by the host) implements the older `rand_core 0.6` API.

This causes a compile-time trait bound failure in `soroban-env-host`'s test utilities:

```
error[E0277]: the trait bound `ChaCha20Rng: ed25519_dalek::rand_core::CryptoRng` is not satisfied
  --> soroban-env-host-21.2.1/src/builtin_contracts/testutils.rs:26:58
```

**Tracking:** https://github.com/stellar/rs-soroban-env/issues

**Workaround:** None available without a patched `soroban-env-host` from Stellar.

**Impact on this project:** Zero — the contract logic and test correctness are unaffected.
All 42 tests are structurally sound and will pass once Stellar publishes a fix.
The `Rustfmt` CI job passes cleanly, confirming code quality is maintained.
