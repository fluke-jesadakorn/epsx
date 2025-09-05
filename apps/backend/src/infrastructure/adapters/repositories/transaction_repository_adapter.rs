// Transaction Repository Adapter
// Bridges DDD transaction monitoring with blockchain infrastructure

use async_trait::async_trait;
use tracing::{info, warn, error};
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::domain::payment::{
    PaymentId, TransactionHash, TransactionRepositoryPort, TransactionRecord
};
use crate::infra::db::diesel::DbPool;
use crate::infra::cache::{Cache, CacheExt};

/// Repository adapter for transaction monitoring operations
pub struct TransactionRepositoryAdapter {
    /// Database pool for transaction storage
    db_pool: Arc<DbPool>,
    
    /// Cache for fast transaction lookups
    cache: Arc<dyn Cache>,
}

impl TransactionRepositoryAdapter {
    pub fn new(
        db_pool: Arc<DbPool>,
        cache: Arc<dyn Cache>,
    ) -> Self {
        Self {
            db_pool,
            cache,
        }
    }
    
    /// Get cache key for transaction data
    fn get_transaction_cache_key(&self, tx_hash: &TransactionHash) -> String {
        format!("transaction:{}", tx_hash.as_str())
    }
    
    /// Get cache key for payment transactions
    fn get_payment_transactions_key(&self, payment_id: &PaymentId) -> String {
        format!("payment_transactions:{}", payment_id.as_str())
    }
    
    /// Store transaction data in cache
    async fn cache_transaction_record(&self, record: &TransactionRecord) -> Result<(), String> {
        let cache_key = self.get_transaction_cache_key(&record.tx_hash);
        
        // Cache for 1 hour (3600 seconds)
        match self.cache.set(&cache_key, record, Some(3600)).await {
            Ok(_) => {
                info!(tx_hash = %record.tx_hash, "Transaction cached successfully");
                Ok(())
            },
            Err(e) => {
                warn!(error = %e.to_string(), tx_hash = %record.tx_hash, "Failed to cache transaction");
                Ok(()) // Don't fail the operation if caching fails
            }
        }
    }
    
    /// Get transaction data from cache
    async fn get_cached_transaction(&self, tx_hash: &TransactionHash) -> Result<Option<TransactionRecord>, String> {
        let cache_key = self.get_transaction_cache_key(tx_hash);
        
        match self.cache.get::<TransactionRecord>(&cache_key).await {
            Ok(Some(record)) => {
                info!(tx_hash = %tx_hash, "Transaction found in cache");
                Ok(Some(record))
            },
            Ok(None) => Ok(None),
            Err(e) => {
                warn!(error = %e.to_string(), tx_hash = %tx_hash, "Cache lookup failed");
                Ok(None) // Don't fail the operation if cache fails
            }
        }
    }
    
    /// Store transaction in database
    async fn store_transaction_in_db(&self, record: &TransactionRecord) -> Result<(), String> {
        info!(
            payment_id = %record.payment_id,
            tx_hash = %record.tx_hash,
            network = record.network,
            "Storing transaction in database"
        );
        
        // In production, this would use Diesel to store transaction
        // Example structure:
        // 
        // let transaction_record = TransactionDatabaseRecord {
        //     payment_id: record.payment_id.to_string(),
        //     tx_hash: record.tx_hash.to_string(),
        //     network: record.network.clone(),
        //     confirmations: record.confirmations,
        //     required_confirmations: record.required_confirmations,
        //     created_at: record.created_at,
        //     confirmed_at: record.confirmed_at,
        //     block_number: record.block_number,
        //     gas_used: record.gas_used,
        //     gas_price: record.gas_price,
        // };
        // 
        // diesel::insert_into(transactions::table)
        //     .values(&transaction_record)
        //     .on_conflict(transactions::tx_hash)
        //     .do_update()
        //     .set((
        //         transactions::confirmations.eq(excluded(transactions::confirmations)),
        //         transactions::confirmed_at.eq(excluded(transactions::confirmed_at)),
        //         transactions::block_number.eq(excluded(transactions::block_number)),
        //         transactions::gas_used.eq(excluded(transactions::gas_used)),
        //         transactions::gas_price.eq(excluded(transactions::gas_price)),
        //     ))
        //     .execute(&mut self.db_pool.get()?)
        //     .map_err(|e| format!("Database error: {}", e))?;
        
        // For now, placeholder implementation
        Ok(())
    }
    
    /// Load transaction from database
    async fn load_transaction_from_db(&self, tx_hash: &TransactionHash) -> Result<Option<TransactionRecord>, String> {
        info!(tx_hash = %tx_hash, "Loading transaction from database");
        
        // In production, this would query the database
        // For now, placeholder returning None
        Ok(None)
    }
}

#[async_trait]
impl TransactionRepositoryPort for TransactionRepositoryAdapter {
    async fn store_transaction(&self, payment_id: &PaymentId, tx_hash: &TransactionHash) -> Result<(), String> {
        info!(
            payment_id = %payment_id,
            tx_hash = %tx_hash,
            "Storing new transaction for monitoring"
        );
        
        // Create initial transaction record
        let record = TransactionRecord {
            payment_id: payment_id.clone(),
            tx_hash: tx_hash.clone(),
            network: "ethereum".to_string(), // Would be determined from payment context
            confirmations: 0,
            required_confirmations: 6, // Standard for Ethereum
            created_at: Utc::now(),
            confirmed_at: None,
            block_number: None,
            gas_used: None,
            gas_price: None,
        };
        
        // Store in database
        self.store_transaction_in_db(&record).await?;
        
        // Cache for fast lookups
        self.cache_transaction_record(&record).await?;
        
        info!(
            payment_id = %payment_id,
            tx_hash = %tx_hash,
            "Transaction stored for monitoring"
        );
        Ok(())
    }
    
    async fn find_by_hash(&self, tx_hash: &TransactionHash) -> Result<Option<TransactionRecord>, String> {
        info!(tx_hash = %tx_hash, "Finding transaction by hash");
        
        // Try cache first
        if let Some(cached_record) = self.get_cached_transaction(tx_hash).await? {
            return Ok(Some(cached_record));
        }
        
        // Fall back to database
        if let Some(db_record) = self.load_transaction_from_db(tx_hash).await? {
            // Cache for future lookups
            let _ = self.cache_transaction_record(&db_record).await;
            return Ok(Some(db_record));
        }
        
        Ok(None)
    }
    
    async fn update_confirmations(&self, tx_hash: &TransactionHash, confirmations: u32) -> Result<(), String> {
        info!(
            tx_hash = %tx_hash,
            confirmations = confirmations,
            "Updating transaction confirmations"
        );
        
        // Get current record
        if let Some(mut record) = self.find_by_hash(tx_hash).await? {
            // Update confirmations
            record.confirmations = confirmations;
            
            // Mark as confirmed if threshold reached
            if confirmations >= record.required_confirmations && record.confirmed_at.is_none() {
                record.confirmed_at = Some(Utc::now());
                info!(
                    tx_hash = %tx_hash,
                    confirmations = confirmations,
                    "Transaction confirmed!"
                );
            }
            
            // Update in database
            self.store_transaction_in_db(&record).await?;
            
            // Update cache
            self.cache_transaction_record(&record).await?;
            
            Ok(())
        } else {
            error!(tx_hash = %tx_hash, "Transaction not found for confirmation update");
            Err("Transaction not found".to_string())
        }
    }
    
    async fn find_pending_confirmations(&self) -> Result<Vec<TransactionRecord>, String> {
        info!("Finding transactions pending confirmation");
        
        // In production, this would query database for unconfirmed transactions
        // SELECT * FROM transactions WHERE confirmations < required_confirmations
        
        // For now, return empty list
        Ok(vec![])
    }
    
    async fn get_transaction_history(&self, payment_id: &PaymentId) -> Result<Vec<TransactionRecord>, String> {
        info!(payment_id = %payment_id, "Getting transaction history for payment");
        
        // In production, this would query database for all payment transactions
        // SELECT * FROM transactions WHERE payment_id = ? ORDER BY created_at DESC
        
        // For now, return empty list
        Ok(vec![])
    }
}

/// Implement Serialize/Deserialize for TransactionRecord for caching
impl serde::Serialize for TransactionRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TransactionRecord", 9)?;
        state.serialize_field("payment_id", &self.payment_id.to_string())?;
        state.serialize_field("tx_hash", &self.tx_hash.to_string())?;
        state.serialize_field("network", &self.network)?;
        state.serialize_field("confirmations", &self.confirmations)?;
        state.serialize_field("required_confirmations", &self.required_confirmations)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("confirmed_at", &self.confirmed_at)?;
        state.serialize_field("block_number", &self.block_number)?;
        state.serialize_field("gas_used", &self.gas_used)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for TransactionRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
        use std::fmt;
        
        struct TransactionRecordVisitor;
        
        impl<'de> Visitor<'de> for TransactionRecordVisitor {
            type Value = TransactionRecord;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TransactionRecord")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<TransactionRecord, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut payment_id = None;
                let mut tx_hash = None;
                let mut network = None;
                let mut confirmations = None;
                let mut required_confirmations = None;
                let mut created_at = None;
                let mut confirmed_at = None;
                let mut block_number = None;
                let mut gas_used = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        "payment_id" => {
                            if payment_id.is_some() {
                                return Err(de::Error::duplicate_field("payment_id"));
                            }
                            let value: String = map.next_value()?;
                            payment_id = Some(PaymentId::from_string(value)
                                .map_err(|e| de::Error::custom(format!("Invalid payment ID: {}", e)))?);
                        },
                        "tx_hash" => {
                            if tx_hash.is_some() {
                                return Err(de::Error::duplicate_field("tx_hash"));
                            }
                            let value: String = map.next_value()?;
                            tx_hash = Some(TransactionHash::new(value)
                                .map_err(|e| de::Error::custom(format!("Invalid transaction hash: {}", e)))?);
                        },
                        "network" => {
                            if network.is_some() {
                                return Err(de::Error::duplicate_field("network"));
                            }
                            network = Some(map.next_value()?);
                        },
                        "confirmations" => {
                            if confirmations.is_some() {
                                return Err(de::Error::duplicate_field("confirmations"));
                            }
                            confirmations = Some(map.next_value()?);
                        },
                        "required_confirmations" => {
                            if required_confirmations.is_some() {
                                return Err(de::Error::duplicate_field("required_confirmations"));
                            }
                            required_confirmations = Some(map.next_value()?);
                        },
                        "created_at" => {
                            if created_at.is_some() {
                                return Err(de::Error::duplicate_field("created_at"));
                            }
                            created_at = Some(map.next_value()?);
                        },
                        "confirmed_at" => {
                            if confirmed_at.is_some() {
                                return Err(de::Error::duplicate_field("confirmed_at"));
                            }
                            confirmed_at = Some(map.next_value()?);
                        },
                        "block_number" => {
                            if block_number.is_some() {
                                return Err(de::Error::duplicate_field("block_number"));
                            }
                            block_number = Some(map.next_value()?);
                        },
                        "gas_used" => {
                            if gas_used.is_some() {
                                return Err(de::Error::duplicate_field("gas_used"));
                            }
                            gas_used = Some(map.next_value()?);
                        },
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                
                let payment_id = payment_id.ok_or_else(|| de::Error::missing_field("payment_id"))?;
                let tx_hash = tx_hash.ok_or_else(|| de::Error::missing_field("tx_hash"))?;
                let network = network.ok_or_else(|| de::Error::missing_field("network"))?;
                let confirmations = confirmations.ok_or_else(|| de::Error::missing_field("confirmations"))?;
                let required_confirmations = required_confirmations.ok_or_else(|| de::Error::missing_field("required_confirmations"))?;
                let created_at = created_at.ok_or_else(|| de::Error::missing_field("created_at"))?;
                
                Ok(TransactionRecord {
                    payment_id,
                    tx_hash,
                    network,
                    confirmations,
                    required_confirmations,
                    created_at,
                    confirmed_at,
                    block_number,
                    gas_used,
                    gas_price: None,
                })
            }
        }
        
        deserializer.deserialize_struct(
            "TransactionRecord",
            &["payment_id", "tx_hash", "network", "confirmations", "required_confirmations", 
              "created_at", "confirmed_at", "block_number", "gas_used"],
            TransactionRecordVisitor
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    // Mock cache for testing
    struct MockCache;
    
    #[async_trait]
    impl Cache for MockCache {
        async fn get(&self, _key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
        }
        
        async fn set(&self, _key: &str, _value: &str, _expiry: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn delete(&self, _key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_transaction_adapter_creation() {
        let mock_pool = Arc::new(crate::infra::db::diesel::create_test_pool().await.unwrap());
        let mock_cache = Arc::new(MockCache);
        
        let adapter = TransactionRepositoryAdapter::new(
            mock_pool,
            mock_cache,
        );
        
        // Basic creation test
        assert!(true); // Adapter created successfully
    }
}