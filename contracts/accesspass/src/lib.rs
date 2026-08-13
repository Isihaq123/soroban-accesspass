//! # Soroban AccessPass
//!
//! A modular, production-grade access control and permission management primitive
//! for Soroban smart contracts on the Stellar network.
//!
//! ## Security model
//! - The **Admin** is fully trusted and has unrestricted power over roles.
//! - There is no time-lock on role grants; Admin decisions take effect immediately.
//! - Delegation does not require the grantor to hold a role at delegation time;
//!   however, `has_delegated_role` validates the grantor's live role on every call.
//! - `initialize` requires the designated admin to sign, preventing front-running.
//! - Admin handoff uses a two-step (offer + accept) pattern — the pending admin
//!   must sign the acceptance, preventing accidental lockout.
//!
//! ## Features
//! - **RBAC**: Named roles (`Symbol`) assignable to any `Address`.
//! - **Time-bound roles**: Optional Unix-timestamp expiry per `(address, role)`.
//! - **Session keys / delegation**: Role-holders delegate to secondary wallets.
//! - **Two-step admin transfer**: Safe ownership handoff with `cancel` support.
//! - **Audit events**: Every mutating action emits a structured on-ledger event.
//! - **State-rent optimised**: TTL extended on every read *and* write.
//!
//! ## Quick start
//! ```ignore
//! client.initialize(&admin);
//! client.grant_role(&admin, &operator, &symbol_short!("MINTER"), &0u64);
//! assert!(client.has_role(&operator, &symbol_short!("MINTER")));
//! client.delegate_permissions(&operator, &session_key);
//! assert!(client.has_delegated_role(&session_key, &operator, &symbol_short!("MINTER")));
//! ```

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Rent management constants
// ---------------------------------------------------------------------------

/// Approximate ledgers per day at ~5-second close time.
const DAY_IN_LEDGERS: u32 = 17_280;

/// Extend TTL to this distance from the current ledger on every write/read.
const BUMP_AMOUNT: u32 = 60 * DAY_IN_LEDGERS; // ~60 days

/// Re-extend when remaining TTL drops below this threshold (~15 days).
const LIFETIME_THRESHOLD: u32 = 15 * DAY_IN_LEDGERS;

// ---------------------------------------------------------------------------
// Storage key schema
// ---------------------------------------------------------------------------

/// All keys written to Soroban persistent / instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Singleton — the current Administrator address.
    Admin,
    /// Singleton — pending admin address during a two-step transfer.
    PendingAdmin,
    /// `(user, role)` → `bool` — whether the role has been granted.
    HasRole(Address, Symbol),
    /// `(user, role)` → `u64` — Unix-second expiry (0 = permanent).
    RoleExpiry(Address, Symbol),
    /// `(grantor, delegatee)` → `bool` — active delegation record.
    Delegated(Address, Address),
}

// ---------------------------------------------------------------------------
// Error helpers
// (Centralised so messages are consistent and easy to update.)
// ---------------------------------------------------------------------------

#[cold]
fn err_already_initialized() -> ! {
    panic!("AccessPass: already initialized")
}

#[cold]
fn err_not_initialized() -> ! {
    panic!("AccessPass: not initialized")
}

#[cold]
fn err_unauthorized() -> ! {
    panic!("AccessPass: caller lacks Admin clearance")
}

#[cold]
fn err_expiry_in_past() -> ! {
    panic!("AccessPass: expiration timestamp must be in the future")
}

#[cold]
fn err_no_pending_transfer() -> ! {
    panic!("AccessPass: no pending admin transfer")
}

#[cold]
fn err_not_pending_admin() -> ! {
    panic!("AccessPass: caller is not the pending admin")
}

#[cold]
fn err_self_delegation() -> ! {
    panic!("AccessPass: grantor and delegatee must be different addresses")
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct SorobanAccessPass;

#[contractimpl]
impl SorobanAccessPass {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialises the AccessPass system with a root Administrator.
    ///
    /// The `admin` address **must sign** this call, preventing front-running by
    /// a third party during contract deployment.  Can only be called once.
    ///
    /// # Arguments
    /// * `admin` – The address that will hold Administrator privileges.
    pub fn initialize(env: Env, admin: Address) {
        // Require the designated admin to authorise — prevents front-running.
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            err_already_initialized();
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        Self::bump_instance(&env);

        env.events()
            .publish((symbol_short!("init"), symbol_short!("admin")), admin);
    }

    // -----------------------------------------------------------------------
    // Role management
    // -----------------------------------------------------------------------

    /// Grants a named role to `grantee`.  Only the Admin may call this.
    ///
    /// # Arguments
    /// * `caller`     – Must be the current Admin (must sign).
    /// * `grantee`    – Address receiving the role.
    /// * `role`       – Short symbol identifying the role (≤ 7 chars).
    /// * `expires_at` – Unix-second timestamp when the role expires.
    ///                  Pass `0` for a permanent (non-expiring) role.
    pub fn grant_role(env: Env, caller: Address, grantee: Address, role: Symbol, expires_at: u64) {
        caller.require_auth();
        Self::assert_admin(&env, &caller);

        let now = env.ledger().timestamp();
        if expires_at > 0 && expires_at <= now {
            err_expiry_in_past();
        }

        let role_key = DataKey::HasRole(grantee.clone(), role.clone());
        let expiry_key = DataKey::RoleExpiry(grantee.clone(), role.clone());

        env.storage().persistent().set(&role_key, &true);
        env.storage().persistent().set(&expiry_key, &expires_at);
        env.storage()
            .persistent()
            .extend_ttl(&role_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
        env.storage()
            .persistent()
            .extend_ttl(&expiry_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        env.events()
            .publish((symbol_short!("grant"), role), (grantee, expires_at));
    }

    /// Revokes a previously granted role from `grantee`.  Only the Admin may call this.
    ///
    /// Idempotent — revoking a role that does not exist is a no-op (no event emitted
    /// for a non-existent revocation).
    ///
    /// # Arguments
    /// * `caller`  – Must be the current Admin (must sign).
    /// * `grantee` – Address losing the role.
    /// * `role`    – Role to revoke.
    pub fn revoke_role(env: Env, caller: Address, grantee: Address, role: Symbol) {
        caller.require_auth();
        Self::assert_admin(&env, &caller);

        let role_key = DataKey::HasRole(grantee.clone(), role.clone());

        // Only write (and emit) if the role actually existed.
        let existed = env
            .storage()
            .persistent()
            .get::<_, bool>(&role_key)
            .unwrap_or(false);

        if existed {
            let expiry_key = DataKey::RoleExpiry(grantee.clone(), role.clone());
            env.storage().persistent().remove(&role_key);
            env.storage().persistent().remove(&expiry_key);

            env.events()
                .publish((symbol_short!("revoke"), role), grantee);
        }
    }

    // -----------------------------------------------------------------------
    // Delegation (Session Keys)
    // -----------------------------------------------------------------------

    /// Allows a role-holder to delegate execution rights to a secondary wallet
    /// (the **session-key** pattern).
    ///
    /// `has_delegated_role` checks that the grantor holds the relevant role at
    /// query time, so delegation of a role the grantor does not yet hold is
    /// permitted but will return `false` until the role is granted.
    ///
    /// # Arguments
    /// * `grantor`   – Address delegating their rights (must sign).
    /// * `delegatee` – Secondary wallet receiving delegated access.
    ///
    /// # Panics
    /// Panics if `grantor == delegatee`.
    pub fn delegate_permissions(env: Env, grantor: Address, delegatee: Address) {
        grantor.require_auth();

        if grantor == delegatee {
            err_self_delegation();
        }

        let delegation_key = DataKey::Delegated(grantor.clone(), delegatee.clone());
        env.storage().persistent().set(&delegation_key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&delegation_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        env.events()
            .publish((symbol_short!("delegate"), grantor), delegatee);
    }

    /// Revokes an active delegation from `grantor` to `delegatee`.
    ///
    /// Idempotent — revoking a delegation that does not exist is a no-op
    /// (no event emitted for a non-existent revocation).
    ///
    /// # Arguments
    /// * `grantor`   – Address revoking delegation (must sign).
    /// * `delegatee` – Secondary wallet losing delegated access.
    pub fn revoke_delegation(env: Env, grantor: Address, delegatee: Address) {
        grantor.require_auth();

        let delegation_key = DataKey::Delegated(grantor.clone(), delegatee.clone());

        let existed = env
            .storage()
            .persistent()
            .get::<_, bool>(&delegation_key)
            .unwrap_or(false);

        if existed {
            env.storage().persistent().remove(&delegation_key);

            env.events()
                .publish((symbol_short!("undlgt"), grantor), delegatee);
        }
    }

    // -----------------------------------------------------------------------
    // Admin transfer (two-step handoff)
    // -----------------------------------------------------------------------

    /// Initiates a two-step Admin transfer.
    ///
    /// The offered address must call `accept_admin` to complete the handoff.
    /// Only the current Admin may initiate a transfer.
    ///
    /// # Arguments
    /// * `caller`    – Must be the current Admin (must sign).
    /// * `new_admin` – Address that will be offered Admin rights.
    pub fn transfer_admin(env: Env, caller: Address, new_admin: Address) {
        caller.require_auth();
        Self::assert_admin(&env, &caller);

        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        Self::bump_instance(&env);

        env.events()
            .publish((symbol_short!("adm_off"), caller), new_admin);
    }

    /// Cancels a pending Admin transfer.
    ///
    /// Only the current Admin may cancel.  This prevents the contract from
    /// being stuck with an unresolvable pending transfer if a wrong address
    /// was supplied to `transfer_admin`.
    ///
    /// # Arguments
    /// * `caller` – Must be the current Admin (must sign).
    pub fn cancel_transfer(env: Env, caller: Address) {
        caller.require_auth();
        Self::assert_admin(&env, &caller);

        env.storage().instance().remove(&DataKey::PendingAdmin);
        Self::bump_instance(&env);

        env.events().publish((symbol_short!("adm_cxl"), caller), ());
    }

    /// Completes the two-step Admin transfer.
    ///
    /// Must be called by the exact address passed to `transfer_admin`.
    ///
    /// # Arguments
    /// * `new_admin` – Must match the address set by `transfer_admin` (must sign).
    pub fn accept_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();

        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .unwrap_or_else(|| err_no_pending_transfer());

        if new_admin != pending {
            err_not_pending_admin();
        }

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        Self::bump_instance(&env);

        env.events().publish(
            (symbol_short!("adm_new"), symbol_short!("admin")),
            new_admin,
        );
    }

    // -----------------------------------------------------------------------
    // Read-only queries
    // -----------------------------------------------------------------------

    /// Returns `true` if `user` directly holds `role` and it has not expired.
    pub fn has_role(env: Env, user: Address, role: Symbol) -> bool {
        Self::check_direct_role(&env, &user, &role)
    }

    /// Returns `true` if `delegatee` can act on behalf of `grantor` for `role`.
    ///
    /// Both conditions must hold:
    /// 1. An active delegation from `grantor` → `delegatee` exists.
    /// 2. `grantor` currently holds `role` and it has not expired.
    pub fn has_delegated_role(
        env: Env,
        delegatee: Address,
        grantor: Address,
        role: Symbol,
    ) -> bool {
        let delegation_key = DataKey::Delegated(grantor.clone(), delegatee.clone());

        let is_delegated = env
            .storage()
            .persistent()
            .get::<_, bool>(&delegation_key)
            .unwrap_or(false);

        if !is_delegated {
            return false;
        }

        // Bump TTL on the delegation entry to prevent archival.
        env.storage()
            .persistent()
            .extend_ttl(&delegation_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        Self::check_direct_role(&env, &grantor, &role)
    }

    /// Returns the current Admin address.
    ///
    /// # Panics
    /// Panics if the contract has not been initialised.
    pub fn get_admin(env: Env) -> Address {
        Self::bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| err_not_initialized())
    }

    /// Returns the pending Admin address, or `None` if no transfer is in progress.
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        Self::bump_instance(&env);
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Returns the Unix-second expiry for a `(user, role)` pair.
    ///
    /// Returns `0` if the role is permanent or does not exist.
    pub fn get_role_expiry(env: Env, user: Address, role: Symbol) -> u64 {
        let expiry_key = DataKey::RoleExpiry(user, role);

        let val = env
            .storage()
            .persistent()
            .get::<_, u64>(&expiry_key)
            .unwrap_or(0);

        if val > 0 {
            env.storage()
                .persistent()
                .extend_ttl(&expiry_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
        }

        val
    }

    /// Returns `true` if an active delegation from `grantor` → `delegatee` exists.
    ///
    /// Note: this does **not** verify that `grantor` currently holds any role.
    pub fn is_delegated(env: Env, grantor: Address, delegatee: Address) -> bool {
        let delegation_key = DataKey::Delegated(grantor, delegatee);

        let val = env
            .storage()
            .persistent()
            .get::<_, bool>(&delegation_key)
            .unwrap_or(false);

        if val {
            env.storage()
                .persistent()
                .extend_ttl(&delegation_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
        }

        val
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Validates that `user` directly holds `role` and it has not expired.
    /// Also bumps TTL on both storage entries to prevent silent archival.
    fn check_direct_role(env: &Env, user: &Address, role: &Symbol) -> bool {
        let role_key = DataKey::HasRole(user.clone(), role.clone());

        let exists = env
            .storage()
            .persistent()
            .get::<_, bool>(&role_key)
            .unwrap_or(false);

        if !exists {
            return false;
        }

        // Keep the role entry alive.
        env.storage()
            .persistent()
            .extend_ttl(&role_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        let expiry_key = DataKey::RoleExpiry(user.clone(), role.clone());
        let expires_at = env
            .storage()
            .persistent()
            .get::<_, u64>(&expiry_key)
            .unwrap_or(0);

        // Keep the expiry entry alive.
        env.storage()
            .persistent()
            .extend_ttl(&expiry_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        // A non-zero expiry that has been reached means the role is inactive.
        if expires_at > 0 && env.ledger().timestamp() >= expires_at {
            return false;
        }

        true
    }

    /// Asserts that `caller` is the current Admin.  Panics otherwise.
    fn assert_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| err_not_initialized());

        if caller != &admin {
            err_unauthorized();
        }
    }

    /// Bumps instance storage TTL.  Called on every public function entry and
    /// after every instance storage write to prevent archival.
    fn bump_instance(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);
    }
}

#[cfg(test)]
mod test;
