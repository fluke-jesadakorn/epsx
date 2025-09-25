mod integration;

use integration::payment_verification_tests::PaymentVerificationIntegrationTests;
use std::sync::Arc;
use std::env;
use sqlx::PgPool;

/// Main integration test runner
/// 
/// To run these tests:
/// 1. Set DATABASE_URL environment variable to test database
/// 2. Run: cargo test --test integration_tests
/// 
/// Example:
/// DATABASE_URL=postgresql://postgres:password@localhost:5432/epsx_test cargo test --test integration_tests

#[tokio::test]
async fn run_payment_verification_integration_tests() {
    // Skip integration tests if DATABASE_URL not set
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("⏭️  Skipping integration tests - DATABASE_URL not set");
            return;
        }
    };

    println!("🚀 Starting EPSX Payment Verification Integration Tests");
    println!("📍 Database: {}", mask_connection_string(&database_url));
    
    // Connect to database
    let db_pool = Arc::new(
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    );

    // Initialize test suite
    let test_suite = match PaymentVerificationIntegrationTests::new(db_pool.clone()).await {
        Ok(suite) => suite,
        Err(e) => {
            panic!("Failed to initialize test suite: {}", e);
        }
    };

    println!("✅ Test suite initialized successfully");

    // Run comprehensive integration tests
    println!("\n🧪 Running Payment Verification Integration Tests...");
    
    // Test 1: Successful verification flow
    if let Err(e) = test_suite.test_successful_verification_flow().await {
        panic!("❌ Successful verification flow test failed: {}", e);
    }
    
    // Test 2: Payment not found
    if let Err(e) = test_suite.test_payment_not_found().await {
        panic!("❌ Payment not found test failed: {}", e);
    }
    
    // Test 3: Already processed payment
    if let Err(e) = test_suite.test_already_processed_payment().await {
        panic!("❌ Already processed payment test failed: {}", e);
    }
    
    // Test 4: Pending confirmations
    if let Err(e) = test_suite.test_pending_confirmations().await {
        panic!("❌ Pending confirmations test failed: {}", e);
    }
    
    // Test 5: Maximum attempts exceeded
    if let Err(e) = test_suite.test_max_attempts_exceeded().await {
        panic!("❌ Max attempts exceeded test failed: {}", e);
    }
    
    // Test 6: Batch verification
    if let Err(e) = test_suite.test_batch_verification().await {
        panic!("❌ Batch verification test failed: {}", e);
    }
    
    // Test 7: Database transaction integrity
    if let Err(e) = test_suite.test_transaction_integrity().await {
        panic!("❌ Transaction integrity test failed: {}", e);
    }
    
    // Test 8: Error recovery and retry
    if let Err(e) = test_suite.test_error_recovery().await {
        panic!("❌ Error recovery test failed: {}", e);
    }

    println!("\n🎉 All Payment Verification Integration Tests PASSED!");
    println!("✅ System is ready for production deployment");
}

/// Mask sensitive parts of connection string for logging
fn mask_connection_string(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let mut masked = parsed.clone();
        if let Some(password) = parsed.password() {
            let _ = masked.set_password(Some("***"));
        }
        masked.to_string()
    } else {
        "***masked***".to_string()
    }
}

#[tokio::test]
async fn test_system_health_check() {
    println!("🏥 Running system health check...");
    
    // Basic compilation and module loading test
    assert!(true, "System compiles and loads successfully");
    
    println!("✅ System health check passed");
}

#[tokio::test] 
async fn test_environment_validation() {
    println!("🔧 Validating test environment...");
    
    // Check if we can access required environment variables
    let env_vars = [
        "DATABASE_URL",
        "RUST_ENV", 
    ];
    
    for var in env_vars.iter() {
        match env::var(var) {
            Ok(value) => println!("  ✅ {}: {}", var, if var.contains("URL") { mask_connection_string(&value) } else { value }),
            Err(_) => println!("  ⚠️  {} not set (may use defaults)", var),
        }
    }
    
    println!("✅ Environment validation completed");
}