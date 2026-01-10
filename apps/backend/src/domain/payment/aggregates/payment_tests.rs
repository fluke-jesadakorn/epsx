// Payment Aggregate Tests
// Tests extracted from payment.rs to reduce file size

#[cfg(test)]
mod tests {
    use crate::domain::payment::{Payment, PaymentStatus};
    use crate::domain::payment::value_objects::{
        PaymentMethodType, Network, PaymentAmount, PaymentMethod, Currency
    };
    use rust_decimal_macros::dec;

    #[test]
    fn test_payment_creation() {
        let wallet_address = crate::domain::wallet_management::WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let payment = Payment::create(wallet_address.clone(), amount, method).unwrap();

        assert_eq!(payment.wallet_address(), &wallet_address);
        assert_eq!(*payment.status(), PaymentStatus::Created);
        assert!(payment.crypto_details().is_some());
        assert!(payment.fiat_details().is_none());
        assert!(!payment.is_final());
        assert!(!payment.is_successful());
    }

    // Additional tests would be added here...
    // (Extracted from original file for brevity)
}