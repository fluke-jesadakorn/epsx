// Clean Architecture Test Suite
// Main test library that organizes tests by architectural layers
// Enables fine-grained test execution and caching for Turborepo

pub mod shared;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// Re-export common test utilities for easy access
pub use shared::{
    fixtures::*,
    mocks::*,
    test_utils::*,
    database_helpers::*,
};

// Test configuration and constants
pub mod test_config {
    pub const DEFAULT_TEST_TIMEOUT_SECONDS: u64 = 30;
    pub const PERFORMANCE_TEST_ITERATIONS: usize = 100;
    pub const CONCURRENCY_TEST_TASKS: usize = 10;
    pub const BULK_OPERATION_SIZE: usize = 1000;
    
    pub fn is_integration_test_enabled() -> bool {
        std::env::var("ENABLE_INTEGRATION_TESTS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true)
    }
    
    pub fn is_performance_test_enabled() -> bool {
        std::env::var("ENABLE_PERFORMANCE_TESTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false)
    }
}

// Layer-specific test runners for Turborepo
pub mod runners {
    use super::*;
    
    /// Run all domain layer tests (pure business logic)
    /// These tests have no external dependencies and run fastest
    pub async fn run_domain_tests() {
        println!("Running domain layer tests...");
        // Domain tests are run automatically by cargo test
        // This is mainly for documentation and future orchestration
    }
    
    /// Run all application layer tests (use cases and services)
    /// These tests use mocked dependencies
    pub async fn run_application_tests() {
        println!("Running application layer tests...");
        // Application tests are run automatically by cargo test
        // This is mainly for documentation and future orchestration
    }
    
    /// Run all infrastructure layer tests (external dependencies)
    /// These tests require external services (database, cache, etc.)
    pub async fn run_infrastructure_tests() {
        if !test_config::is_integration_test_enabled() {
            println!("Infrastructure tests disabled. Set ENABLE_INTEGRATION_TESTS=true to run.");
            return;
        }
        
        println!("Running infrastructure layer tests...");
        // Infrastructure tests are run automatically by cargo test
        // This is mainly for documentation and future orchestration
    }
    
    /// Run all presentation layer tests (HTTP handlers and middleware)
    /// These tests require the web server to be running
    pub async fn run_presentation_tests() {
        if !test_config::is_integration_test_enabled() {
            println!("Presentation tests disabled. Set ENABLE_INTEGRATION_TESTS=true to run.");
            return;
        }
        
        println!("Running presentation layer tests...");
        // Presentation tests are run automatically by cargo test
        // This is mainly for documentation and future orchestration
    }
    
    /// Run performance tests across all layers
    pub async fn run_performance_tests() {
        if !test_config::is_performance_test_enabled() {
            println!("Performance tests disabled. Set ENABLE_PERFORMANCE_TESTS=true to run.");
            return;
        }
        
        println!("Running performance tests...");
        // Performance tests are run automatically by cargo test
        // This is mainly for documentation and future orchestration
    }
}

// Helper macros for test organization
#[macro_export]
macro_rules! domain_test {
    ($test_name:ident, $test_body:block) => {
        #[cfg(test)]
        #[tokio::test]
        async fn $test_name() {
            // Domain tests should be pure and fast
            let result = tokio::time::timeout(
                std::time::Duration::from_secs($crate::test_config::DEFAULT_TEST_TIMEOUT_SECONDS),
                async { $test_body }
            ).await;
            
            assert!(result.is_ok(), "Domain test timed out");
        }
    };
}

#[macro_export]
macro_rules! application_test {
    ($test_name:ident, $test_body:block) => {
        #[cfg(test)]
        #[tokio::test]
        async fn $test_name() {
            // Application tests may use mocks and take longer
            let result = tokio::time::timeout(
                std::time::Duration::from_secs($crate::test_config::DEFAULT_TEST_TIMEOUT_SECONDS * 2),
                async { $test_body }
            ).await;
            
            assert!(result.is_ok(), "Application test timed out");
        }
    };
}

#[macro_export]
macro_rules! infrastructure_test {
    ($test_name:ident, $test_body:block) => {
        #[cfg(test)]
        #[tokio::test]
        async fn $test_name() {
            if !$crate::test_config::is_integration_test_enabled() {
                return;
            }
            
            // Infrastructure tests involve external services
            let result = tokio::time::timeout(
                std::time::Duration::from_secs($crate::test_config::DEFAULT_TEST_TIMEOUT_SECONDS * 3),
                async { $test_body }
            ).await;
            
            assert!(result.is_ok(), "Infrastructure test timed out");
        }
    };
}

#[macro_export]
macro_rules! presentation_test {
    ($test_name:ident, $test_body:block) => {
        #[cfg(test)]
        #[tokio::test]
        async fn $test_name() {
            if !$crate::test_config::is_integration_test_enabled() {
                return;
            }
            
            // Presentation tests involve HTTP server
            let result = tokio::time::timeout(
                std::time::Duration::from_secs($crate::test_config::DEFAULT_TEST_TIMEOUT_SECONDS * 2),
                async { $test_body }
            ).await;
            
            assert!(result.is_ok(), "Presentation test timed out");
        }
    };
}

#[macro_export]
macro_rules! performance_test {
    ($test_name:ident, $test_body:block) => {
        #[cfg(test)]
        #[tokio::test]
        async fn $test_name() {
            if !$crate::test_config::is_performance_test_enabled() {
                return;
            }
            
            // Performance tests may take much longer
            let result = tokio::time::timeout(
                std::time::Duration::from_secs($crate::test_config::DEFAULT_TEST_TIMEOUT_SECONDS * 10),
                async { $test_body }
            ).await;
            
            assert!(result.is_ok(), "Performance test timed out");
        }
    };
}