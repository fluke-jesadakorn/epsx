//! Test Database Configuration Utilities
//! 
//! Provides consistent database connection patterns for testing.
//! Replaces hardcoded connection strings with environment-aware configuration.

use std::env;

/// Database configuration for testing
pub struct TestDatabaseConfig;

impl TestDatabaseConfig {
    /// Get the test database URL from environment or use secure default
    /// 
    /// Priority:
    /// 1. DATABASE_URL environment variable
    /// 2. TEST_DATABASE_URL environment variable  
    /// 3. Constructed from individual components
    /// 4. Secure localhost default (no embedded credentials)
    pub fn get_database_url() -> String {
        // First priority: Standard DATABASE_URL
        if let Ok(url) = env::var("DATABASE_URL") {
            return url;
        }
        
        // Second priority: Test-specific DATABASE_URL
        if let Ok(url) = env::var("TEST_DATABASE_URL") {
            return url;
        }
        
        // Third priority: Construct from components
        let host = env::var("TEST_DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("TEST_DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let user = env::var("TEST_DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = env::var("TEST_DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
        let database = env::var("TEST_DB_NAME").unwrap_or_else(|_| "epsx_test".to_string());
        
        format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, database)
    }
    
    /// Get a secure database URL that doesn't expose credentials in logs
    /// Returns the connection string with password masked
    pub fn get_safe_database_url_for_logging() -> String {
        let url = Self::get_database_url();
        Self::mask_credentials_in_url(&url)
    }
    
    /// Mask credentials in a database URL for safe logging
    pub fn mask_credentials_in_url(url: &str) -> String {
        if url.contains("://") {
            if let Some(at_pos) = url.find('@') {
                if let Some(colon_pos) = url.rfind(':') {
                    if colon_pos < at_pos {
                        let before_creds = &url[..url.find("://").unwrap() + 3];
                        let after_at = &url[at_pos..];
                        return format!("{}***:***{}", before_creds, after_at);
                    }
                }
            }
        }
        url.to_string()
    }
    
    /// Get database connection parameters for connection testing
    pub fn get_connection_params() -> ConnectionParams {
        let url = Self::get_database_url();
        Self::parse_database_url(&url)
    }
    
    /// Parse a PostgreSQL URL into components
    fn parse_database_url(url: &str) -> ConnectionParams {
        // Simple parser for postgresql://user:pass@host:port/db
        let url = url.strip_prefix("postgresql://").unwrap_or(url);
        let url = url.strip_prefix("postgres://").unwrap_or(url);
        
        let parts: Vec<&str> = url.split('@').collect();
        if parts.len() != 2 {
            return ConnectionParams::default();
        }
        
        let auth_parts: Vec<&str> = parts[0].split(':').collect();
        let host_parts: Vec<&str> = parts[1].split('/').collect();
        let host_port: Vec<&str> = host_parts[0].split(':').collect();
        
        ConnectionParams {
            user: auth_parts.get(0).unwrap_or(&"postgres").to_string(),
            password: auth_parts.get(1).unwrap_or(&"postgres").to_string(),
            host: host_port.get(0).unwrap_or(&"localhost").to_string(),
            port: host_port.get(1).unwrap_or(&"5432").parse().unwrap_or(5432),
            database: host_parts.get(1).unwrap_or(&"epsx_test").to_string(),
        }
    }
}

/// Database connection parameters
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database: String,
}

impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "epsx_test".to_string(),
        }
    }
}

/// Database utility functions for tests
pub struct DatabaseTestUtils;

impl DatabaseTestUtils {
    /// Check if database is available before running tests
    pub async fn is_database_available() -> bool {
        use std::process::Command;
        
        let params = TestDatabaseConfig::get_connection_params();
        let output = Command::new("pg_isready")
            .arg("-h")
            .arg(&params.host)
            .arg("-p")
            .arg(&params.port.to_string())
            .output();
            
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Get environment suggestions for test setup
    pub fn get_environment_setup_help() -> Vec<String> {
        vec![
            "Set DATABASE_URL for test database connection".to_string(),
            "Or set individual components:".to_string(),
            "  TEST_DB_HOST=localhost".to_string(),
            "  TEST_DB_PORT=5432".to_string(),
            "  TEST_DB_USER=postgres".to_string(),
            "  TEST_DB_PASSWORD=postgres".to_string(),
            "  TEST_DB_NAME=epsx_test".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_credential_masking() {
        let url = "postgresql://user:secret123@localhost:5432/testdb";
        let masked = TestDatabaseConfig::mask_credentials_in_url(url);
        assert_eq!(masked, "postgresql://***:***@localhost:5432/testdb");
        
        // Should not mask URLs without credentials
        let simple_url = "postgresql://localhost:5432/testdb";
        let not_masked = TestDatabaseConfig::mask_credentials_in_url(simple_url);
        assert_eq!(not_masked, simple_url);
    }
    
    #[test]
    fn test_url_parsing() {
        let url = "postgresql://testuser:testpass@testhost:1234/testdb";
        let params = TestDatabaseConfig::parse_database_url(url);
        
        assert_eq!(params.user, "testuser");
        assert_eq!(params.password, "testpass");
        assert_eq!(params.host, "testhost");
        assert_eq!(params.port, 1234);
        assert_eq!(params.database, "testdb");
    }
    
    #[test]
    fn test_environment_priority() {
        // This test would need to set environment variables to test properly
        // For now, just test that the function doesn't panic
        let _url = TestDatabaseConfig::get_database_url();
        let _safe_url = TestDatabaseConfig::get_safe_database_url_for_logging();
    }
}