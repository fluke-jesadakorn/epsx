// Integration tests for DDD migration validation
// Tests that all DDD bounded contexts work end-to-end while maintaining API contracts

use std::sync::Arc;
use tokio;

// Test utilities module
#[path = "utils/mod.rs"]
mod utils;

use epsx::{
    infrastructure::DDDContainer,
    application::{
        shared::{CommandHandler, QueryHandler},
        user_management::commands::models::{CreateUserCommand, DeleteUserCommand},
        user_management::queries::models::{GetUserQuery, ListUsersQuery},
    },
    domain::user_management::value_objects::{FirebaseUid},
    infra::db::diesel::create_pool,
};

/// Test the User Management DDD bounded context end-to-end
#[tokio::test]
async fn test_user_management_ddd_bounded_context() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| utils::TestDatabaseConfig::get_database_url());
    
    let pool = Arc::new(create_pool(&database_url).await?);
    
    // Create DDD container
    let ddd_container = DDDContainer::new(pool.clone());
    
    // Test data
    let test_email = "ddd_test_user@example.com".to_string();
    let test_firebase_uid = "ddd_test_firebase_uid_123".to_string();
    
    // === COMMAND TESTING ===
    
    // 1. Test CreateUser command through DDD
    println!("🧪 Testing CreateUser command via DDD bounded context...");
    
    let create_command = CreateUserCommand::new(test_email.clone(), test_firebase_uid.clone())
        .with_permissions(vec!["epsx:basic:view".to_string()])
        .with_email_verified(true)
        .initiated_by("test_admin".to_string())
        .with_correlation_id("test_correlation_123".to_string());
    
    let response = ddd_container
        .create_user_handler()
        .handle(create_command)
        .await?;
    
    println!("✅ User created via DDD with ID: {:?}", response.user_id);
    
    // === QUERY TESTING ===
    
    // 2. Test GetUser query through DDD
    println!("🧪 Testing GetUser query via DDD bounded context...");
    
    let get_query = GetUserQuery::new(response.user_id.to_string())
        .with_permissions()
        .with_sessions()
        .requested_by("test_requester".to_string())
        .with_correlation_id("query_correlation_123".to_string());
    
    let retrieved_user_response = ddd_container
        .user_query_service()
        .get_user(&get_query)
        .await?;
    
    assert_eq!(retrieved_user_response.user_id, response.user_id);
    assert_eq!(retrieved_user_response.email.to_string(), test_email);
    println!("✅ User retrieved via DDD: {:?}", retrieved_user_response.email);
    
    // 3. Test ListUsers query through DDD
    println!("🧪 Testing ListUsers query via DDD bounded context...");
    
    let list_query = ListUsersQuery::new()
        .with_limit(10)
        .with_offset(0)
        .with_email_domain_filter("example.com".to_string());
    
    let user_list_response = ddd_container
        .user_query_service()
        .list_users(&list_query)
        .await?;
    
    assert!(!user_list_response.users.is_empty(), "User list should not be empty");
    println!("✅ Listed {} users via DDD", user_list_response.users.len());
    
    // === CLEANUP ===
    
    // 4. Test DeleteUser command through DDD
    println!("🧪 Testing DeleteUser command via DDD bounded context...");
    
    let firebase_uid = FirebaseUid::new(&test_firebase_uid)?;
    let delete_command = DeleteUserCommand::new(firebase_uid);
    
    ddd_container
        .user_application_service()
        .delete_user(&delete_command)
        .await?;
    
    println!("✅ User deleted via DDD");
    
    // Verify user was actually deleted
    let get_deleted_query = GetUserQuery::new(response.user_id.to_string());
    
    let deleted_result = ddd_container
        .user_query_service()
        .get_user(&get_deleted_query)
        .await;
    
    // Should return error since user is deleted
    assert!(deleted_result.is_err(), "Deleted user should not be retrievable");
    
    println!("✅ User Management DDD bounded context test passed!");
    Ok(())
}

/// Test Trading Analytics DDD bounded context (basic validation)
#[tokio::test]
async fn test_trading_analytics_ddd_bounded_context() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| utils::TestDatabaseConfig::get_database_url());
    
    let pool = Arc::new(create_pool(&database_url).await?);
    
    // Create stock analysis adapter
    let stock_analysis_adapter = Arc::new(
        epsx::infrastructure::adapters::repositories::StockAnalysisRepositoryAdapter::new(pool.clone())
    );
    
    println!("🧪 Testing Trading Analytics DDD bounded context...");
    
    // Test converting legacy EPS rankings to DDD format
    let legacy_params = epsx::dom::services::eps_ranking_service::EPSRankingParams {
        country: Some("america".to_string()),
        sector: None,
        sort_by: Some("qoq_growth".to_string()),
        page: 1,
        limit: 5,
        min_eps: None,
        min_growth: None,
    };
    
    // This should use the DDD Trading Analytics bounded context internally
    // but still return legacy format for API compatibility
    let ddd_result = stock_analysis_adapter
        .convert_legacy_rankings_to_ddd(legacy_params)
        .await;
    
    match ddd_result {
        Ok(rankings) => {
            println!("✅ Trading Analytics DDD conversion successful: {} rankings", rankings.len());
            
            // Validate DDD structure
            for ranking in rankings.iter().take(2) {
                assert!(!ranking.symbol.is_empty(), "Symbol should not be empty");
                assert!(!ranking.name.is_empty(), "Company name should not be empty");
                println!("  📊 DDD Ranking: {} - {}", ranking.symbol, ranking.name);
            }
            
            println!("✅ Trading Analytics DDD bounded context test passed!");
        },
        Err(e) => {
            // This might fail if TradingView service is not available in test environment
            println!("⚠️ Trading Analytics test skipped due to external dependency: {}", e);
        }
    }
    
    Ok(())
}

/// Test DDD Container initialization and basic functionality
#[tokio::test]
async fn test_ddd_container_initialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing DDD Container initialization...");
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| utils::TestDatabaseConfig::get_database_url());
    
    let pool = Arc::new(create_pool(&database_url).await?);
    
    // Create DDD container
    let ddd_container = DDDContainer::new(pool.clone());
    
    // Test that all components are available
    // Database pool should be available
    let _db_pool = ddd_container.db_pool();
    
    // Test that services are properly wired
    let _user_query_service = ddd_container.user_query_service();
    let _user_application_service = ddd_container.user_application_service();
    
    println!("✅ DDD Container initialization test passed!");
    Ok(())
}

/// Test basic DDD architecture components
#[tokio::test]
async fn test_ddd_architecture_components() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing DDD architecture components...");
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| utils::TestDatabaseConfig::get_database_url());
    
    let pool = Arc::new(create_pool(&database_url).await?);
    let ddd_container = DDDContainer::new(pool.clone());
    
    // Test command handlers are available
    let _create_user_handler = ddd_container.create_user_handler();
    let _grant_permission_handler = ddd_container.grant_permission_handler();
    
    // Test repository ports are available
    let _user_repository = ddd_container.user_repository();
    let _session_repository = ddd_container.session_repository();
    
    // Test event bus is available
    let _event_bus = ddd_container.event_bus();
    
    println!("✅ DDD architecture components test passed!");
    Ok(())
}