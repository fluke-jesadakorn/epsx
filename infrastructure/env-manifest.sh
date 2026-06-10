#!/bin/bash
# Env var manifest — single source of truth for all environments
# Sourced by validate-env.sh to check presence + format before deploy
# Compatible with bash 3.2+ (no associative arrays)

# --- Required vars per service (no defaults in create-secrets.sh) ---

REQUIRED_POSTGRES="DB_USER DB_PASSWORD DB_NAME"

REQUIRED_REDIS="REDIS_PASSWORD"

REQUIRED_MINIO="MINIO_ROOT_USER MINIO_ROOT_PASSWORD MINIO_PUBLIC_URL"

REQUIRED_BACKEND="ENV BACKEND_URL FRONTEND_URL ADMIN_FRONTEND_URL WEB3_APP_SECRET WALLET_SIGNATURE_SECRET WEB3_SESSION_SECRET JWT_SECRET BLOCKCHAIN_NETWORK COMPANY_WALLET_MAINNET BSC_MAINNET_RPC_URL"

REQUIRED_FRONTEND="ENV BACKEND_URL FRONTEND_URL ADMIN_FRONTEND_URL WALLETCONNECT_PROJECT_ID BLOCKCHAIN_NETWORK CHAIN_ID MINIO_PUBLIC_URL OAUTH_CLIENT_ID"

# --- Format validators (parallel arrays: name + regex) ---
FORMAT_VARS=(
  BACKEND_URL
  FRONTEND_URL
  ADMIN_FRONTEND_URL
  BSC_MAINNET_RPC_URL
  MINIO_PUBLIC_URL
  COMPANY_WALLET_MAINNET
  CHAIN_ID
  BLOCKCHAIN_NETWORK
  COMPANY_WALLET_TESTNET
  BSC_TESTNET_RPC_URL
  NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET
  NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET
)
FORMAT_REGEX=(
  '^https?://.+'
  '^https?://.+'
  '^https?://.+'
  '^https?://.+'
  '^https?://.+'
  '^0x[0-9a-fA-F]{40}$'
  '^[0-9]+$'
  '^(mainnet|testnet)$'
  '^0x[0-9a-fA-F]{40}$'
  '^https?://.+'
  '^0x[0-9a-fA-F]{40}$'
  '^0x[0-9a-fA-F]{40}$'
)
