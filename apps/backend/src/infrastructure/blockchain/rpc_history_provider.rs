use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use crate::domain::payment::repository_ports::{TransactionHistoryProvider, TransactionHistoryInfo};
use crate::infrastructure::blockchain::event_parser::parse_payment_event;
use crate::domain::shared_kernel::app_error::AppError;

pub struct RpcTransactionHistoryProvider {
    provider: Arc<Provider<Http>>,
    contract_address: H160,
    event_topic: H256,
}

impl RpcTransactionHistoryProvider {
    pub fn new(rpc_url: String, contract_address: String) -> Result<Self, AppError> {
        let provider = Provider::<Http>::try_from(&rpc_url)
            .map_err(|e| AppError::infrastructure_error(format!("Failed to create provider: {}", e)))?;

        let contract_address = contract_address.parse::<H160>()
            .map_err(|e| AppError::infrastructure_error(format!("Invalid contract address: {}", e)))?;

        // PaymentReceived event topic0
        let event_topic = H256::from_slice(&hex::decode(
            "a7f9e7f4f9c6e7e3d8b3a2f1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1"
        ).unwrap_or_default());

        Ok(Self {
            provider: Arc::new(provider),
            contract_address,
            event_topic,
        })
    }
}

#[async_trait]
impl TransactionHistoryProvider for RpcTransactionHistoryProvider {
    async fn get_history(
        &self,
        wallet_address: &str,
        _page: u32,
        _per_page: u32,
    ) -> Result<(Vec<TransactionHistoryInfo>, u64), String> {
        let wallet_addr = wallet_address.parse::<H160>()
            .map_err(|e| format!("Invalid wallet address: {}", e))?;

        // In RPC mode, we search for logs where the first indexed param (user) is the wallet
        let filter = Filter::new()
            .address(self.contract_address)
            .topic0(self.event_topic)
            .topic1(wallet_addr)
            .from_block(0); // For development/RPC, we search from start or a reasonable depth

        let logs = self.provider.get_logs(&filter).await
            .map_err(|e| format!("Failed to fetch logs: {}", e))?;

        let mut history = Vec::new();
        for log in logs {
            if let Ok(event) = parse_payment_event(&log) {
                history.push(TransactionHistoryInfo {
                    tx_hash: event.transaction_hash,
                    amount: event.amount.to_string().parse().unwrap_or(0.0),
                    currency: "USDT".to_string(), // Default or detect from token_address
                    status: "Confirmed".to_string(),
                    timestamp: event.timestamp,
                    from_address: event.user_address,
                    to_address: self.contract_address.to_string(),
                    block_number: event.block_number,
                    plan_name: Some(format!("Context #{} (type={})", event.context_id, event.context_type)),
                });
            }
        }

        // RPC doesn't easily support pagination for logs without custom indexing
        // For simplicity in development, return all matches found in the range
        let total = history.len() as u64;
        
        // Sort by timestamp desc
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok((history, total))
    }
}
