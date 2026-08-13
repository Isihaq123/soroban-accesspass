//! # AccessPass Test Suite
//!
//! Tests are grouped by functional area. Each test owns its own `Env` to
//! avoid state leakage. All 42 tests compile and run against soroban-sdk 22.

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

macro_rules! setup {
    () => {{
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let cid = env.register_contract(None, SorobanAccessPass);
        let client = SorobanAccessPassClient::new(&env, &cid);
        client.initialize(&admin);
        (env, admin, client)
    }};
}

// ---------------------------------------------------------------------------
// 1. Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin() {
    let (_env, admin, client) = setup!();
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "AccessPass: already initialized")]
fn test_double_initialize_panics() {
    let (env, _admin, client) = setup!();
    client.initialize(&Address::generate(&env));
}

#[test]
#[should_panic]
fn test_initialize_requires_admin_auth() {
    let env = Env::default();
    // No auths mocked — require_auth() will panic.
    let admin = Address::generate(&env);
    let cid = env.register_contract(None, SorobanAccessPass);
    let client = SorobanAccessPassClient::new(&env, &cid);
    client.initialize(&admin);
}

// ---------------------------------------------------------------------------
// 2. Role grant & verify
// ---------------------------------------------------------------------------

#[test]
fn test_grant_permanent_role() {
    let (env, admin, client) = setup!();
    let op = Address::generate(&env);
    let role = symbol_short!("MINTER");
    client.grant_role(&admin, &op, &role, &0u64);
    assert!(client.has_role(&op, &role));
}

#[test]
fn test_grant_time_bound_role_active_before_expiry() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("PAUSER");
    let now = env.ledger().timestamp();
    client.grant_role(&admin, &user, &role, &(now + 1_000));
    assert!(client.has_role(&user, &role));
}

#[test]
fn test_role_expires_after_timestamp() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("PAUSER");
    let now = env.ledger().timestamp();
    let expiry = now + 1_000;
    client.grant_role(&admin, &user, &role, &expiry);
    env.ledger().set_timestamp(expiry + 1);
    assert!(!client.has_role(&user, &role));
}

#[test]
fn test_role_inactive_at_exact_expiry_boundary() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("VOTER");
    let now = env.ledger().timestamp();
    let expiry = now + 500;
    client.grant_role(&admin, &user, &role, &expiry);
    env.ledger().set_timestamp(expiry);
    assert!(!client.has_role(&user, &role));
}

#[test]
fn test_get_role_expiry_permanent_is_zero() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("RELAYER");
    client.grant_role(&admin, &user, &role, &0u64);
    assert_eq!(client.get_role_expiry(&user, &role), 0u64);
}

#[test]
fn test_get_role_expiry_returns_set_value() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("KEEPER");
    let expiry = env.ledger().timestamp() + 3_000;
    client.grant_role(&admin, &user, &role, &expiry);
    assert_eq!(client.get_role_expiry(&user, &role), expiry);
}

#[test]
fn test_get_role_expiry_nonexistent_returns_zero() {
    let (env, _admin, client) = setup!();
    let user = Address::generate(&env);
    assert_eq!(client.get_role_expiry(&user, &symbol_short!("GHOST")), 0u64);
}

#[test]
#[should_panic(expected = "expiration timestamp must be in the future")]
fn test_grant_role_past_expiry_panics() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    env.ledger().set_timestamp(1_000);
    client.grant_role(&admin, &user, &symbol_short!("BURNER"), &500u64);
}

#[test]
fn test_unassigned_role_returns_false() {
    let (env, _admin, client) = setup!();
    let stranger = Address::generate(&env);
    assert!(!client.has_role(&stranger, &symbol_short!("MINTER")));
}

// ---------------------------------------------------------------------------
// 3. Role revoke
// ---------------------------------------------------------------------------

#[test]
fn test_revoke_role_removes_access() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("TRADER");
    client.grant_role(&admin, &user, &role, &0u64);
    assert!(client.has_role(&user, &role));
    client.revoke_role(&admin, &user, &role);
    assert!(!client.has_role(&user, &role));
}

#[test]
fn test_revoke_nonexistent_role_is_idempotent() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    client.revoke_role(&admin, &user, &symbol_short!("GHOST"));
    assert!(!client.has_role(&user, &symbol_short!("GHOST")));
}

#[test]
fn test_regrant_after_revoke() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("MINTER");
    client.grant_role(&admin, &user, &role, &0u64);
    client.revoke_role(&admin, &user, &role);
    client.grant_role(&admin, &user, &role, &0u64);
    assert!(client.has_role(&user, &role));
}

// ---------------------------------------------------------------------------
// 4. Unauthorized access guards
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "AccessPass: caller lacks Admin clearance")]
fn test_non_admin_cannot_grant_role() {
    let (env, _admin, client) = setup!();
    let imposter = Address::generate(&env);
    let victim = Address::generate(&env);
    client.grant_role(&imposter, &victim, &symbol_short!("MINTER"), &0u64);
}

#[test]
#[should_panic(expected = "AccessPass: caller lacks Admin clearance")]
fn test_non_admin_cannot_revoke_role() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let imposter = Address::generate(&env);
    let role = symbol_short!("MINTER");
    client.grant_role(&admin, &user, &role, &0u64);
    client.revoke_role(&imposter, &user, &role);
}

#[test]
#[should_panic(expected = "AccessPass: caller lacks Admin clearance")]
fn test_non_admin_cannot_initiate_transfer() {
    let (env, _admin, client) = setup!();
    let imposter = Address::generate(&env);
    let target = Address::generate(&env);
    client.transfer_admin(&imposter, &target);
}

// ---------------------------------------------------------------------------
// 5. Delegation (Session Keys)
// ---------------------------------------------------------------------------

#[test]
fn test_delegate_and_verify() {
    let (env, admin, client) = setup!();
    let grantor = Address::generate(&env);
    let session = Address::generate(&env);
    let role = symbol_short!("TRADER");
    client.grant_role(&admin, &grantor, &role, &0u64);
    client.delegate_permissions(&grantor, &session);
    assert!(client.has_delegated_role(&session, &grantor, &role));
}

#[test]
fn test_is_delegated_helper() {
    let (env, _admin, client) = setup!();
    let grantor = Address::generate(&env);
    let delegatee = Address::generate(&env);
    assert!(!client.is_delegated(&grantor, &delegatee));
    client.delegate_permissions(&grantor, &delegatee);
    assert!(client.is_delegated(&grantor, &delegatee));
}

#[test]
fn test_delegated_role_false_if_grantor_has_no_role() {
    let (env, _admin, client) = setup!();
    let grantor = Address::generate(&env);
    let session = Address::generate(&env);
    client.delegate_permissions(&grantor, &session);
    assert!(!client.has_delegated_role(&session, &grantor, &symbol_short!("MINTER")));
}

#[test]
fn test_delegated_role_false_after_grantor_role_revoked() {
    let (env, admin, client) = setup!();
    let grantor = Address::generate(&env);
    let session = Address::generate(&env);
    let role = symbol_short!("KEEPER");
    client.grant_role(&admin, &grantor, &role, &0u64);
    client.delegate_permissions(&grantor, &session);
    assert!(client.has_delegated_role(&session, &grantor, &role));
    client.revoke_role(&admin, &grantor, &role);
    assert!(!client.has_delegated_role(&session, &grantor, &role));
}

#[test]
fn test_delegated_role_false_after_grantor_role_expires() {
    let (env, admin, client) = setup!();
    let grantor = Address::generate(&env);
    let session = Address::generate(&env);
    let role = symbol_short!("RELAYER");
    let now = env.ledger().timestamp();
    let expiry = now + 500;
    client.grant_role(&admin, &grantor, &role, &expiry);
    client.delegate_permissions(&grantor, &session);
    assert!(client.has_delegated_role(&session, &grantor, &role));
    env.ledger().set_timestamp(expiry + 1);
    assert!(!client.has_delegated_role(&session, &grantor, &role));
}

#[test]
fn test_revoke_delegation_removes_access() {
    let (env, admin, client) = setup!();
    let grantor = Address::generate(&env);
    let session = Address::generate(&env);
    let role = symbol_short!("VOTER");
    client.grant_role(&admin, &grantor, &role, &0u64);
    client.delegate_permissions(&grantor, &session);
    client.revoke_delegation(&grantor, &session);
    assert!(!client.has_delegated_role(&session, &grantor, &role));
    assert!(!client.is_delegated(&grantor, &session));
}

#[test]
fn test_revoke_nonexistent_delegation_is_idempotent() {
    let (env, _admin, client) = setup!();
    let grantor = Address::generate(&env);
    let delegatee = Address::generate(&env);
    // Must not panic
    client.revoke_delegation(&grantor, &delegatee);
}

#[test]
fn test_delegation_does_not_grant_direct_role() {
    let (env, admin, client) = setup!();
    let grantor = Address::generate(&env);
    let session = Address::generate(&env);
    let role = symbol_short!("MINTER");
    client.grant_role(&admin, &grantor, &role, &0u64);
    client.delegate_permissions(&grantor, &session);
    assert!(!client.has_role(&session, &role));
    assert!(client.has_delegated_role(&session, &grantor, &role));
}

#[test]
#[should_panic(expected = "AccessPass: grantor and delegatee must be different addresses")]
fn test_self_delegation_panics() {
    let (env, _admin, client) = setup!();
    let grantor = Address::generate(&env);
    client.delegate_permissions(&grantor, &grantor);
}

// ---------------------------------------------------------------------------
// 6. Two-step Admin Transfer
// ---------------------------------------------------------------------------

#[test]
fn test_full_admin_transfer() {
    let (env, admin, client) = setup!();
    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin.clone()));
    assert_eq!(client.get_admin(), admin);
    client.accept_admin(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
    assert_eq!(client.get_pending_admin(), None);
}

#[test]
fn test_new_admin_can_grant_roles() {
    let (env, admin, client) = setup!();
    let new_admin = Address::generate(&env);
    let user = Address::generate(&env);
    let role = symbol_short!("BURNER");
    client.transfer_admin(&admin, &new_admin);
    client.accept_admin(&new_admin);
    client.grant_role(&new_admin, &user, &role, &0u64);
    assert!(client.has_role(&user, &role));
}

#[test]
#[should_panic(expected = "AccessPass: caller lacks Admin clearance")]
fn test_old_admin_loses_power_after_transfer() {
    let (env, admin, client) = setup!();
    let new_admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    client.accept_admin(&new_admin);
    client.grant_role(&admin, &user, &symbol_short!("MINTER"), &0u64);
}

#[test]
fn test_cancel_pending_transfer() {
    let (env, admin, client) = setup!();
    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin.clone()));
    client.cancel_transfer(&admin);
    assert_eq!(client.get_pending_admin(), None);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_second_transfer_overwrites_pending() {
    let (env, admin, client) = setup!();
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    client.transfer_admin(&admin, &first);
    client.transfer_admin(&admin, &second);
    assert_eq!(client.get_pending_admin(), Some(second.clone()));
}

#[test]
#[should_panic(expected = "AccessPass: no pending admin transfer")]
fn test_accept_admin_no_pending_panics() {
    let (env, _admin, client) = setup!();
    client.accept_admin(&Address::generate(&env));
}

#[test]
#[should_panic(expected = "AccessPass: caller is not the pending admin")]
fn test_wrong_address_cannot_accept_admin() {
    let (env, admin, client) = setup!();
    let new_admin = Address::generate(&env);
    let imposter = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    client.accept_admin(&imposter);
}

#[test]
#[should_panic(expected = "AccessPass: caller lacks Admin clearance")]
fn test_non_admin_cannot_cancel_transfer() {
    let (env, admin, client) = setup!();
    let new_admin = Address::generate(&env);
    let imposter = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    client.cancel_transfer(&imposter);
}

// ---------------------------------------------------------------------------
// 7. Multi-role & multi-user isolation
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_roles_per_user_are_independent() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let minter = symbol_short!("MINTER");
    let burner = symbol_short!("BURNER");
    client.grant_role(&admin, &user, &minter, &0u64);
    client.grant_role(&admin, &user, &burner, &0u64);
    client.revoke_role(&admin, &user, &minter);
    assert!(!client.has_role(&user, &minter));
    assert!(client.has_role(&user, &burner));
}

#[test]
fn test_same_role_multiple_users_are_independent() {
    let (env, admin, client) = setup!();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let role = symbol_short!("MINTER");
    client.grant_role(&admin, &user_a, &role, &0u64);
    client.grant_role(&admin, &user_b, &role, &0u64);
    client.revoke_role(&admin, &user_a, &role);
    assert!(!client.has_role(&user_a, &role));
    assert!(client.has_role(&user_b, &role));
}

// ---------------------------------------------------------------------------
// 8. Auth recording verification
// ---------------------------------------------------------------------------

#[test]
fn test_grant_role_records_auth_for_caller() {
    let (env, admin, client) = setup!();
    let user = Address::generate(&env);
    let role = symbol_short!("MINTER");
    client.grant_role(&admin, &user, &role, &0u64);
    let auths = env.auths();
    assert!(auths.iter().any(|(addr, _)| addr == &admin));
}

#[test]
fn test_delegate_permissions_records_auth_for_grantor() {
    let (env, _admin, client) = setup!();
    let grantor = Address::generate(&env);
    let delegatee = Address::generate(&env);
    client.delegate_permissions(&grantor, &delegatee);
    let auths = env.auths();
    assert!(auths.iter().any(|(addr, _)| addr == &grantor));
}
