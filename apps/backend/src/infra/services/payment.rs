use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::app::ports::services::{PayGw, PaymentServiceError, PaymentAddress, TransactionDetails, TransactionStatus};
use crate::dom::values::{UserId, Currency};

/// Configuration for different payment gateways
#[derive(Debug, Clone)]
pub struct PaymentGatewayConfig {
    pub api_key: String,
    pub api_secret: Option<String>,
    pub base_url: String,
    pub webhook_secret: Option<String>,
    pub supported_currencies: Vec<Currency>,
    pub network_configs: HashMap<String, NetworkConfig>,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: String,
    pub confirmation_count: u32,
    pub fee_currency: Currency,
    pub min_amount: Decimal,
    pub max_amount: Decimal,
}

/// Multi-gateway payment service that can handle different crypto payment providers
pub struct MultiGatewayPaymentService {
    #[allow(dead_code)]
    client: Client,
    primary_gateway: Box<dyn PayGw + Send + Sync>,
    fallback_gateways: Vec<Box<dyn PayGw + Send + Sync>>,
}

impl MultiGatewayPaymentService {
    pub fn new(
        primary_gateway: Box<dyn PayGw + Send + Sync>,
        fallback_gateways: Vec<Box<dyn PayGw + Send + Sync>>,
    ) -> Self {
        Self {
            client: Client::new(),
            primary_gateway,
            fallback_gateways,
        }
    }
}

#[async_trait]
impl PayGw for MultiGatewayPaymentService {
    async fn create_payment_address(
        &self,
        currency: &Currency,
        user_id: &UserId,
    ) -> Result<PaymentAddress, PaymentServiceError> {
        // Try primary gateway first
        match self.primary_gateway.create_payment_address(currency, user_id).await {
            Ok(address) => Ok(address),
            Err(e) => {
                tracing::warn!("Primary gateway failed: {}, trying fallbacks", e);
                
                // Try fallback gateways
                for gateway in &self.fallback_gateways {
                    if let Ok(address) = gateway.create_payment_address(currency, user_id).await {
                        return Ok(address);
                    }
                }
                
                Err(e)
            }
        }
    }

    async fn verify_transaction(
        &self,
        tx_hash: &str,
        expected_amount: Decimal,
        currency: &Currency,
    ) -> Result<TransactionDetails, PaymentServiceError> {
        // Try primary gateway first
        match self.primary_gateway.verify_transaction(tx_hash, expected_amount, currency).await {
            Ok(details) => Ok(details),
            Err(e) => {
                tracing::warn!("Primary gateway verification failed: {}, trying fallbacks", e);
                
                // Try fallback gateways
                for gateway in &self.fallback_gateways {
                    if let Ok(details) = gateway.verify_transaction(tx_hash, expected_amount, currency).await {
                        return Ok(details);
                    }
                }
                
                Err(e)
            }
        }
    }

    async fn get_exchange_rate(
        &self,
        from: &Currency,
        to: &Currency,
    ) -> Result<Decimal, PaymentServiceError> {
        self.primary_gateway.get_exchange_rate(from, to).await
    }

    async fn estimate_fees(
        &self,
        currency: &Currency,
        network: &str,
    ) -> Result<Decimal, PaymentServiceError> {
        self.primary_gateway.estimate_fees(currency, network).await
    }
}

/// CoinPayments.net gateway implementation
pub struct CoinPaymentsGateway {
    client: Client,
    config: PaymentGatewayConfig,
}

impl CoinPaymentsGateway {
    pub fn new(config: PaymentGatewayConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = std::env::var("COINPAYMENTS_API_KEY")
            .map_err(|_| "COINPAYMENTS_API_KEY environment variable not set")?;
        let api_secret = std::env::var("COINPAYMENTS_API_SECRET").ok();
        let webhook_secret = std::env::var("COINPAYMENTS_WEBHOOK_SECRET").ok();

        let config = PaymentGatewayConfig {
            api_key,
            api_secret,
            base_url: "https://www.coinpayments.net/api.php".to_string(),
            webhook_secret,
            supported_currencies: vec![
                Currency::BTC,
                Currency::ETH,
                Currency::USDT,
                Currency::BNB,
            ],
            network_configs: Self::default_network_configs(),
        };

        Ok(Self::new(config))
    }

    fn default_network_configs() -> HashMap<String, NetworkConfig> {
        let mut configs = HashMap::new();
        
        configs.insert("BTC".to_string(), NetworkConfig {
            name: "Bitcoin".to_string(),
            confirmation_count: 3,
            fee_currency: Currency::BTC,
            min_amount: Decimal::new(1, 6), // 0.000001 BTC
            max_amount: Decimal::new(100, 0), // 100 BTC
        });

        configs.insert("ETH".to_string(), NetworkConfig {
            name: "Ethereum".to_string(),
            confirmation_count: 12,
            fee_currency: Currency::ETH,
            min_amount: Decimal::new(1, 4), // 0.0001 ETH
            max_amount: Decimal::new(1000, 0), // 1000 ETH
        });

        configs.insert("USDT_ERC20".to_string(), NetworkConfig {
            name: "USDT (ERC-20)".to_string(),
            confirmation_count: 12,
            fee_currency: Currency::ETH,
            min_amount: Decimal::new(10, 0), // 10 USDT
            max_amount: Decimal::new(100000, 0), // 100,000 USDT
        });

        configs.insert("USDT_TRC20".to_string(), NetworkConfig {
            name: "USDT (TRC-20)".to_string(),
            confirmation_count: 19,
            fee_currency: Currency::TRX,
            min_amount: Decimal::new(10, 0), // 10 USDT
            max_amount: Decimal::new(100000, 0), // 100,000 USDT
        });

        configs
    }

    async fn make_api_call(&self, command: &str, params: &serde_json::Value) -> Result<serde_json::Value, PaymentServiceError> {
        let mut form_data: HashMap<String, String> = HashMap::new();
        form_data.insert("version".to_string(), "1".to_string());
        form_data.insert("cmd".to_string(), command.to_string());
        form_data.insert("key".to_string(), self.config.api_key.clone());
        form_data.insert("format".to_string(), "json".to_string());

        // Add custom parameters
        if let serde_json::Value::Object(map) = params {
            for (key, value) in map {
                if let serde_json::Value::String(s) = value {
                    form_data.insert(key.clone(), s.clone());
                }
            }
        }

        let response = self.client
            .post(&self.config.base_url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| PaymentServiceError::ExternalError(e.to_string()))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PaymentServiceError::ExternalError(e.to_string()))?;

        if let Some(error) = json.get("error") {
            if error != "ok" {
                return Err(PaymentServiceError::ExternalError(error.to_string()));
            }
        }

        Ok(json)
    }
}

#[async_trait]
impl PayGw for CoinPaymentsGateway {
    async fn create_payment_address(
        &self,
        currency: &Currency,
        user_id: &UserId,
    ) -> Result<PaymentAddress, PaymentServiceError> {
        let currency_code = match currency {
            Currency::BTC => "BTC",
            Currency::ETH => "ETH", 
            Currency::USDT => "USDT.ERC20",
            Currency::BNB => "BNB",
            Currency::TRX => "TRX",
            _ => return Err(PaymentServiceError::InvalidCurrency(format!("{:?}", currency))),
        };

        let params = serde_json::json!({
            "currency": currency_code,
            "ipn_url": format!("https://your-domain.com/webhook/coinpayments/{}", user_id.value())
        });

        let response = self.make_api_call("get_callback_address", &params).await?;

        let result = response.get("result")
            .ok_or_else(|| PaymentServiceError::ExternalError("No result in response".to_string()))?;

        let address = result.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PaymentServiceError::AddressGenerationFailed("No address in response".to_string()))?;

        Ok(PaymentAddress {
            address: address.to_string(),
            currency: currency.clone(),
            network: currency_code.to_string(),
            qr_code_url: None, // CoinPayments doesn't provide QR codes directly
        })
    }

    async fn verify_transaction(
        &self,
        tx_hash: &str,
        expected_amount: Decimal,
        currency: &Currency,
    ) -> Result<TransactionDetails, PaymentServiceError> {
        let params = serde_json::json!({
            "txid": tx_hash
        });

        let response = self.make_api_call("get_tx_info", &params).await?;
        
        let result = response.get("result")
            .ok_or_else(|| PaymentServiceError::TransactionNotFound(tx_hash.to_string()))?;

        let amount_str = result.get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PaymentServiceError::ExternalError("No amount in transaction".to_string()))?;

        let amount = amount_str.parse::<Decimal>()
            .map_err(|_| PaymentServiceError::ExternalError("Invalid amount format".to_string()))?;

        let status_code = result.get("status")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let status = match status_code {
            0 => TransactionStatus::Pending,
            1 | 100 => TransactionStatus::Confirmed,
            _ => TransactionStatus::Failed,
        };

        let confirmations = result.get("confirms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // Verify amount matches
        if (amount - expected_amount).abs() > Decimal::new(1, 8) { // Allow small precision differences
            return Err(PaymentServiceError::AmountMismatch {
                expected: expected_amount,
                actual: amount,
            });
        }

        Ok(TransactionDetails {
            tx_hash: tx_hash.to_string(),
            amount,
            currency: currency.clone(),
            confirmations,
            timestamp: chrono::Utc::now(), // CoinPayments might provide actual timestamp
            status,
        })
    }

    async fn get_exchange_rate(
        &self,
        from: &Currency,
        to: &Currency,
    ) -> Result<Decimal, PaymentServiceError> {
        let from_code = format!("{:?}", from);
        let to_code = format!("{:?}", to);

        let params = serde_json::json!({
            "accepted": "1"
        });

        let response = self.make_api_call("rates", &params).await?;
        
        let rates = response.get("result")
            .ok_or_else(|| PaymentServiceError::ExternalError("No rates in response".to_string()))?;

        let from_rate = rates.get(&from_code)
            .and_then(|v| v.get("rate_btc"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Decimal>().ok())
            .ok_or_else(|| PaymentServiceError::ExternalError(format!("Rate not found for {}", from_code)))?;

        let to_rate = rates.get(&to_code)
            .and_then(|v| v.get("rate_btc"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Decimal>().ok())
            .ok_or_else(|| PaymentServiceError::ExternalError(format!("Rate not found for {}", to_code)))?;

        if to_rate.is_zero() {
            return Err(PaymentServiceError::ExternalError("Division by zero in exchange rate".to_string()));
        }

        Ok(from_rate / to_rate)
    }

    async fn estimate_fees(
        &self,
        currency: &Currency,
        _network: &str,
    ) -> Result<Decimal, PaymentServiceError> {
        // For CoinPayments, fees are typically network-dependent
        match currency {
            Currency::BTC => Ok(Decimal::new(10000, 8)), // ~0.0001 BTC
            Currency::ETH => Ok(Decimal::new(5, 3)), // ~0.005 ETH
            Currency::USDT => Ok(Decimal::new(5, 0)), // ~5 USDT (ERC-20 fee)
            Currency::BNB => Ok(Decimal::new(1, 3)), // ~0.001 BNB
            Currency::TRX => Ok(Decimal::new(10, 0)), // ~10 TRX
            _ => Err(PaymentServiceError::InvalidCurrency(format!("{:?}", currency))),
        }
    }
}

/// Mock payment gateway for testing
pub struct MockPaymentGateway {
    pub should_fail: bool,
    pub generated_addresses: std::sync::Arc<std::sync::Mutex<Vec<PaymentAddress>>>,
}

impl MockPaymentGateway {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            generated_addresses: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn new_failing() -> Self {
        Self {
            should_fail: true,
            generated_addresses: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl PayGw for MockPaymentGateway {
    async fn create_payment_address(
        &self,
        currency: &Currency,
        user_id: &UserId,
    ) -> Result<PaymentAddress, PaymentServiceError> {
        if self.should_fail {
            return Err(PaymentServiceError::AddressGenerationFailed("Mock failure".to_string()));
        }

        let address = PaymentAddress {
            address: format!("mock_address_{}_{}", currency.to_string().to_lowercase(), user_id.value()),
            currency: currency.clone(),
            network: "mock_network".to_string(),
            qr_code_url: Some("https://mock.com/qr.png".to_string()),
        };

        self.generated_addresses.lock().unwrap().push(address.clone());
        Ok(address)
    }

    async fn verify_transaction(
        &self,
        tx_hash: &str,
        expected_amount: Decimal,
        currency: &Currency,
    ) -> Result<TransactionDetails, PaymentServiceError> {
        if self.should_fail {
            return Err(PaymentServiceError::TransactionNotFound(tx_hash.to_string()));
        }

        Ok(TransactionDetails {
            tx_hash: tx_hash.to_string(),
            amount: expected_amount,
            currency: currency.clone(),
            confirmations: 6,
            timestamp: chrono::Utc::now(),
            status: TransactionStatus::Confirmed,
        })
    }

    async fn get_exchange_rate(
        &self,
        _from: &Currency,
        _to: &Currency,
    ) -> Result<Decimal, PaymentServiceError> {
        if self.should_fail {
            return Err(PaymentServiceError::ExternalError("Mock failure".to_string()));
        }

        Ok(Decimal::new(1, 0)) // 1:1 rate for simplicity
    }

    async fn estimate_fees(
        &self,
        _currency: &Currency,
        _network: &str,
    ) -> Result<Decimal, PaymentServiceError> {
        if self.should_fail {
            return Err(PaymentServiceError::ExternalError("Mock failure".to_string()));
        }

        Ok(Decimal::new(1, 2)) // 0.01 mock fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::UserId;

    #[tokio::test]
    async fn test_mock_payment_gateway() {
        let gateway = MockPaymentGateway::new();
        let user_id = UserId::new("test_user").unwrap();
        let currency = Currency::BTC;

        let address = gateway.create_payment_address(&currency, &user_id).await.unwrap();
        assert!(address.address.contains("btc"));
        assert!(address.address.contains("test_user"));

        let tx_details = gateway.verify_transaction(
            "mock_tx_hash",
            Decimal::new(100, 2), // 1.00
            &currency
        ).await.unwrap();

        assert_eq!(tx_details.amount, Decimal::new(100, 2));
        assert_eq!(tx_details.status, TransactionStatus::Confirmed);
    }

    #[tokio::test]
    async fn test_multi_gateway_fallback() {
        let failing_gateway = Box::new(MockPaymentGateway::new_failing());
        let working_gateway = Box::new(MockPaymentGateway::new());
        
        let multi_gateway = MultiGatewayPaymentService::new(
            failing_gateway,
            vec![working_gateway]
        );

        let user_id = UserId::new("test_user").unwrap();
        let currency = Currency::ETH;

        // Should fallback to working gateway
        let address = multi_gateway.create_payment_address(&currency, &user_id).await.unwrap();
        assert!(address.address.contains("eth"));
    }
}