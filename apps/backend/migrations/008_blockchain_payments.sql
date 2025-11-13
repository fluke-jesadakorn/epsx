-- Migration: Add blockchain payment tracking
-- Created: 2025-10-18
-- Description: Add support for blockchain payment verification and event tracking

-- Add blockchain-specific fields to payments table
ALTER TABLE IF EXISTS payments
  ADD COLUMN IF NOT EXISTS contract_address VARCHAR(42),
  ADD COLUMN IF NOT EXISTS blockchain_network VARCHAR(20) DEFAULT 'bsc',
  ADD COLUMN IF NOT EXISTS token_address VARCHAR(42),
  ADD COLUMN IF NOT EXISTS payment_event_id BIGINT,
  ADD COLUMN IF NOT EXISTS block_number BIGINT,
  ADD COLUMN IF NOT EXISTS log_index INTEGER;

-- Create table to track processed blockchain events (prevent duplicates)
CREATE TABLE IF NOT EXISTS processed_blockchain_events (
  id SERIAL PRIMARY KEY,
  transaction_hash VARCHAR(66) NOT NULL,
  log_index INTEGER NOT NULL,
  event_type VARCHAR(50) NOT NULL DEFAULT 'PaymentReceived',
  block_number BIGINT NOT NULL,
  contract_address VARCHAR(42) NOT NULL,
  user_address VARCHAR(42) NOT NULL,
  plan_id INTEGER NOT NULL,
  token_address VARCHAR(42) NOT NULL,
  amount DECIMAL(20, 6) NOT NULL,
  payment_id BIGINT NOT NULL,
  event_timestamp TIMESTAMP NOT NULL,
  processed_at TIMESTAMP NOT NULL DEFAULT NOW(),
  subscription_id INTEGER,
  processing_status VARCHAR(20) DEFAULT 'pending',
  error_message TEXT,
  CONSTRAINT unique_event UNIQUE (transaction_hash, log_index)
);

-- Create indexes for faster event lookups
CREATE INDEX IF NOT EXISTS idx_processed_events_tx_hash
  ON processed_blockchain_events(transaction_hash);

CREATE INDEX IF NOT EXISTS idx_processed_events_block
  ON processed_blockchain_events(block_number);

CREATE INDEX IF NOT EXISTS idx_processed_events_user
  ON processed_blockchain_events(user_address);

CREATE INDEX IF NOT EXISTS idx_processed_events_status
  ON processed_blockchain_events(processing_status);

CREATE INDEX IF NOT EXISTS idx_processed_events_processed_at
  ON processed_blockchain_events(processed_at DESC);

-- Add index for blockchain payments
CREATE INDEX IF NOT EXISTS idx_payments_blockchain
  ON payments(blockchain_network, contract_address)
  WHERE contract_address IS NOT NULL;

-- Create view for payment verification status
CREATE OR REPLACE VIEW payment_verification_status AS
SELECT
  p.id as payment_id,
  p.wallet_address,
  p.plan_id,
  p.amount,
  p.payment_method,
  p.status as payment_status,
  p.blockchain_network,
  p.contract_address,
  p.transaction_hash,
  p.block_number,
  pbe.processing_status as blockchain_status,
  pbe.event_timestamp,
  pbe.processed_at,
  pbe.error_message,
  pbe.subscription_id
FROM payments p
LEFT JOIN processed_blockchain_events pbe
  ON p.transaction_hash = pbe.transaction_hash
WHERE p.blockchain_network IS NOT NULL;

-- Comments for documentation
COMMENT ON TABLE processed_blockchain_events IS 'Tracks all PaymentReceived events from smart contract to prevent duplicate processing';
COMMENT ON COLUMN processed_blockchain_events.transaction_hash IS 'Blockchain transaction hash (0x...)';
COMMENT ON COLUMN processed_blockchain_events.log_index IS 'Event log index within the transaction';
COMMENT ON COLUMN processed_blockchain_events.processing_status IS 'pending, processing, completed, failed';
COMMENT ON COLUMN payments.contract_address IS 'Smart contract address that received the payment';
COMMENT ON COLUMN payments.blockchain_network IS 'Blockchain network: bsc, ethereum, etc.';
