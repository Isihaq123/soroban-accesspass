#!/usr/bin/env bash
# =============================================================
# deploy-and-invoke.sh
# Deploys soroban-accesspass to Stellar Testnet and runs a
# full end-to-end demonstration of every contract function.
#
# Prerequisites:
#   stellar-cli installed  (cargo install --locked stellar-cli)
#   WASM built             (stellar contract build)
#
# Usage:
#   chmod +x scripts/deploy-and-invoke.sh
#   ./scripts/deploy-and-invoke.sh
# =============================================================

set -euo pipefail

NETWORK="testnet"
WASM="target/wasm32-unknown-unknown/release/soroban_accesspass.wasm"

echo "==> Generating identities..."
stellar keys generate --global admin  --network $NETWORK --fund || true
stellar keys generate --global alice  --network $NETWORK --fund || true
stellar keys generate --global session --network $NETWORK --fund || true

ADMIN_ADDR=$(stellar keys address admin)
ALICE_ADDR=$(stellar keys address alice)
SESSION_ADDR=$(stellar keys address session)

echo "Admin:   $ADMIN_ADDR"
echo "Alice:   $ALICE_ADDR"
echo "Session: $SESSION_ADDR"

echo ""
echo "==> Building contract..."
stellar contract build

echo ""
echo "==> Deploying contract..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm $WASM \
  --source admin \
  --network $NETWORK)

echo "Contract ID: $CONTRACT_ID"

echo ""
echo "==> Initializing..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- initialize --admin "$ADMIN_ADDR"

echo ""
echo "==> Granting MINTER role to Alice (permanent)..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- grant_role \
    --caller "$ADMIN_ADDR" \
    --grantee "$ALICE_ADDR" \
    --role MINTER \
    --expires_at 0

echo ""
echo "==> Checking Alice has MINTER..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- has_role --user "$ALICE_ADDR" --role MINTER

echo ""
echo "==> Alice delegates to session key..."
stellar contract invoke \
  --id $CONTRACT_ID --source alice --network $NETWORK \
  -- delegate_permissions \
    --grantor "$ALICE_ADDR" \
    --delegatee "$SESSION_ADDR"

echo ""
echo "==> Checking session key has delegated MINTER..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- has_delegated_role \
    --delegatee "$SESSION_ADDR" \
    --grantor "$ALICE_ADDR" \
    --role MINTER

echo ""
echo "==> Revoking MINTER from Alice..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- revoke_role \
    --caller "$ADMIN_ADDR" \
    --grantee "$ALICE_ADDR" \
    --role MINTER

echo ""
echo "==> Verifying delegated access also gone..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- has_delegated_role \
    --delegatee "$SESSION_ADDR" \
    --grantor "$ALICE_ADDR" \
    --role MINTER

echo ""
echo "==> Admin transfer (2-step)..."
stellar contract invoke \
  --id $CONTRACT_ID --source admin --network $NETWORK \
  -- transfer_admin \
    --caller "$ADMIN_ADDR" \
    --new_admin "$ALICE_ADDR"

stellar contract invoke \
  --id $CONTRACT_ID --source alice --network $NETWORK \
  -- accept_admin --new_admin "$ALICE_ADDR"

echo ""
echo "==> New admin is Alice: $(stellar contract invoke \
  --id $CONTRACT_ID --source alice --network $NETWORK \
  -- get_admin)"

echo ""
echo "All done. Contract ID: $CONTRACT_ID"
