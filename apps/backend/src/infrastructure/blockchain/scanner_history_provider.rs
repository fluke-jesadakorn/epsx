use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::payment::repository_ports::{TransactionHistoryProvider, TransactionHistoryInfo};

use chrono::{DateTime, Utc};

pub struct ScannerTransactionHistoryProvider {
    api_key: String,
    contract_address: String,
    event_topic: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct BscScanResponse {
    status: String,
    message: String,
    result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BscScanLog {
    address: String,
    topics: Vec<String>,
    data: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "timeStamp")]
    time_stamp: String,
    #[serde(rename = "gasPrice")]
    gas_price: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    #[serde(rename = "logIndex")]
    log_index: String,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "transactionIndex")]
    transaction_index: String,
}

impl ScannerTransactionHistoryProvider {
    pub fn new(api_key: String, contract_address: String) -> Self {
        let event_topic = "0xa7f9e7f4f9c6e7e3d8b3a2f1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1".to_string();
        let is_mainnet = std::env::var("BLOCKCHAIN_NETWORK")
            .unwrap_or_default()
            .eq_ignore_ascii_case("mainnet");
        let base_url = if is_mainnet {
            "https://api.bscscan.com/api"
        } else {
            "https://api-testnet.bscscan.com/api"
        }.to_string();
        Self {
            api_key,
            contract_address,
            event_topic,
            base_url,
        }
    }
}

#[async_trait]
impl TransactionHistoryProvider for ScannerTransactionHistoryProvider {
    async fn get_history(
        &self,
        wallet_address: &str,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<TransactionHistoryInfo>, u64), String> {
        let client = reqwest::Client::new();
        
        // Format wallet address for topic1 (pad to 32 bytes)
        let wallet_topic = format!("0x000000000000000000000000{}", &wallet_address[2..].to_lowercase());

        let url = format!(
            "{}?module=logs&action=getLogs&fromBlock=0&toBlock=latest&address={}&topic0={}&topic1={}&page={}&offset={}&apikey={}",
            self.base_url,
            self.contract_address,
            self.event_topic,
            wallet_topic,
            page,
            per_page,
            self.api_key
        );

        let resp = client.get(&url).send().await
            .map_err(|e| format!("Failed to call BscScan: {}", e))?
            .json::<BscScanResponse>().await
            .map_err(|e| format!("Failed to parse BscScan response: {}", e))?;

        if resp.status != "1" && resp.message != "No logs found" {
            return Err(format!("BscScan error: {}", resp.message));
        }

        let mut history = Vec::new();
        if let serde_json::Value::Array(logs) = resp.result {
            for log_val in logs {
                let log: BscScanLog = serde_json::from_value(log_val)
                    .map_err(|e| format!("Invalid log format: {}", e))?;
                
                let block_number = u64::from_str_radix(log.block_number.trim_start_matches("0x"), 16)
                    .unwrap_or(0);
                let timestamp_secs = i64::from_str_radix(log.time_stamp.trim_start_matches("0x"), 16)
                    .unwrap_or(0);
                
                let timestamp = DateTime::<Utc>::from_timestamp(timestamp_secs, 0)
                    .unwrap_or_else(Utc::now);

                let amount_wei = log.data.trim_start_matches("0x");
                let amount = u128::from_str_radix(amount_wei, 16)
                    .map(|a| (a as f64) / 1e18) // Assuming 18 decimals
                    .unwrap_or(0.0);

                history.push(TransactionHistoryInfo {
                    amount,
                    currency: "BNB".to_string(), // Native token
                    status: "completed".to_string(),
                    tx_hash: log.transaction_hash,
                    timestamp,
                    block_number,
                    plan_name: None,
                    from_address: wallet_address.to_string(),
                    to_address: self.contract_address.clone(),
                });
            }
        }

        // BscScan getLogs doesn't return total count in a simple way for logs
        // Usually you call and it returns what it matches
        let total = history.len() as u64; // Placeholder
        
        Ok((history, total))
    }
}
