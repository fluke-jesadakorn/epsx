//! Load tests for Casbin authorization system
//! Tests system behavior under sustained high load conditions

use epsx::dom::services::casbin_service::CasbinService;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub concurrent_users: usize,
    pub requests_per_user: usize,
    pub test_duration: Duration,
    pub ramp_up_duration: Duration,
    pub think_time: Duration,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 100,
            requests_per_user: 1000,
            test_duration: Duration::from_secs(60),
            ramp_up_duration: Duration::from_secs(10),
            think_time: Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Default)]
pub struct LoadTestResults {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub requests_per_second: f64,
    pub cache_hit_ratio: f64,
}

pub struct CasbinLoadTester {
    casbin_service: Arc<CasbinService>,
    config: LoadTestConfig,
}

impl CasbinLoadTester {
    pub async fn new(config: LoadTestConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("LOAD_TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_load_test_db".to_string());
        
        let pool = PgPool::connect(&database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        let casbin_service = Arc::new(CasbinService::new(pool).await?);
        
        // Setup test data
        Self::setup_test_data(&casbin_service).await?;
        
        Ok(Self {
            casbin_service,
            config,
        })
    }
    
    async fn setup_test_data(service: &CasbinService) -> Result<(), Box<dyn std::error::Error>> {
        println!("Setting up load test data...");
        
        // Create role hierarchy
        service.add_role_for_user("admin_user", "admin").await?;
        service.add_role_for_user("moderator_user", "moderator").await?;
        
        // Create test users
        for i in 0..1000 {
            let user_type = match i % 4 {
                0 => "admin",
                1 => "moderator", 
                2 => "premium_user",
                _ => "basic_user",
            };
            service.add_role_for_user(&format!("load_test_user_{}", i), user_type).await?;
        }
        
        // Create diverse policies
        let resources = vec![
            "/api/v1/users", "/api/v1/admin", "/api/v1/trading", "/api/v1/analytics",
            "/api/v1/premium", "/api/v1/reports", "/api/v1/settings", "/api/v1/notifications"
        ];
        
        let actions = vec!["GET", "POST", "PUT", "DELETE"];
        
        for resource in &resources {
            for action in &actions {
                service.add_policy("admin", resource, action).await?;
                
                if !action.eq(&"DELETE") {
                    service.add_policy("moderator", resource, action).await?;
                }
                
                if action.eq(&"GET") {
                    service.add_policy("premium_user", resource, action).await?;
                    
                    if resource.contains("trading") || resource.contains("users") {
                        service.add_policy("basic_user", resource, action).await?;
                    }
                }
            }
        }
        
        println!("Load test data setup complete");
        Ok(())
    }
    
    pub async fn run_load_test(&self) -> Result<LoadTestResults, Box<dyn std::error::Error>> {
        println!("Starting load test with {} concurrent users for {:?}", 
                 self.config.concurrent_users, self.config.test_duration);
        
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_users));
        let mut response_times = Vec::new();
        let mut successful_requests = 0u64;
        let mut failed_requests = 0u64;
        
        let test_start = Instant::now();
        let test_end = test_start + self.config.test_duration;
        
        let mut handles = vec![];
        
        // Spawn concurrent user tasks
        for user_id in 0..self.config.concurrent_users {
            let service = self.casbin_service.clone();
            let semaphore = semaphore.clone();
            let config = self.config.clone();
            
            // Stagger user start times for ramp-up
            let start_delay = Duration::from_millis(
                (config.ramp_up_duration.as_millis() as u64 * user_id as u64) / self.config.concurrent_users as u64
            );
            
            let handle = tokio::spawn(async move {
                sleep(start_delay).await;
                
                let mut user_response_times = vec![];
                let mut user_successful = 0u64;
                let mut user_failed = 0u64;
                
                while Instant::now() < test_end {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    let request_start = Instant::now();
                    
                    // Simulate realistic request pattern
                    let (user, resource, action) = Self::generate_request_pattern(user_id);
                    
                    match service.enforce(&user, &resource, &action).await {
                        Ok(_) => {
                            user_successful += 1;
                            user_response_times.push(request_start.elapsed());
                        }
                        Err(_) => {
                            user_failed += 1;
                        }
                    }
                    
                    // Think time between requests
                    sleep(config.think_time).await;
                }
                
                (user_response_times, user_successful, user_failed)
            });
            
            handles.push(handle);
        }
        
        // Collect results from all users
        for handle in handles {
            let (user_times, user_success, user_fail) = handle.await?;
            response_times.extend(user_times);
            successful_requests += user_success;
            failed_requests += user_fail;
        }
        
        let total_duration = test_start.elapsed();
        let total_requests = successful_requests + failed_requests;
        
        // Calculate statistics
        response_times.sort();
        let average_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<Duration>() / response_times.len() as u32
        } else {
            Duration::ZERO
        };
        
        let min_response_time = response_times.first().copied().unwrap_or(Duration::ZERO);
        let max_response_time = response_times.last().copied().unwrap_or(Duration::ZERO);
        
        let p95_index = (response_times.len() as f64 * 0.95) as usize;
        let p95_response_time = response_times.get(p95_index).copied().unwrap_or(Duration::ZERO);
        
        let p99_index = (response_times.len() as f64 * 0.99) as usize;
        let p99_response_time = response_times.get(p99_index).copied().unwrap_or(Duration::ZERO);
        
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();
        
        // Get cache statistics
        let cache_stats = self.casbin_service.cache_stats().await;
        let cache_hit_ratio = if cache_stats.total_entries > 0 {
            cache_stats.active_entries as f64 / cache_stats.total_entries as f64
        } else {
            0.0
        };
        
        let results = LoadTestResults {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time,
            min_response_time,
            max_response_time,
            p95_response_time,
            p99_response_time,
            requests_per_second,
            cache_hit_ratio,
        };
        
        Self::print_results(&results);
        
        Ok(results)
    }
    
    fn generate_request_pattern(user_id: usize) -> (String, String, String) {
        let user = format!("load_test_user_{}", user_id);
        
        let resources = vec![
            "/api/v1/users", "/api/v1/trading", "/api/v1/analytics", "/api/v1/premium",
            "/api/v1/admin", "/api/v1/reports", "/api/v1/settings"
        ];
        
        let actions = vec!["GET", "POST", "PUT", "DELETE"];
        
        // Weighted selection to simulate realistic usage patterns
        let resource = match user_id % 10 {
            0..=5 => "/api/v1/trading", // 60% trading requests
            6..=7 => "/api/v1/users",   // 20% user requests  
            8 => "/api/v1/analytics",   // 10% analytics
            _ => "/api/v1/premium",     // 10% premium features
        };
        
        let action = match user_id % 10 {
            0..=6 => "GET",    // 70% read requests
            7..=8 => "POST",   // 20% create requests
            9 => "PUT",        // 10% update requests
            _ => "DELETE",     // Very few delete requests
        };
        
        (user, resource.to_string(), action.to_string())
    }
    
    fn print_results(results: &LoadTestResults) {
        println!("\n=== Load Test Results ===");
        println!("Total Requests: {}", results.total_requests);
        println!("Successful Requests: {}", results.successful_requests);
        println!("Failed Requests: {}", results.failed_requests);
        println!("Success Rate: {:.2}%", (results.successful_requests as f64 / results.total_requests as f64) * 100.0);
        println!("Requests per Second: {:.2}", results.requests_per_second);
        println!("\nResponse Time Statistics:");
        println!("  Average: {:?}", results.average_response_time);
        println!("  Min: {:?}", results.min_response_time);
        println!("  Max: {:?}", results.max_response_time);
        println!("  95th Percentile: {:?}", results.p95_response_time);
        println!("  99th Percentile: {:?}", results.p99_response_time);
        println!("\nCache Performance:");
        println!("  Hit Ratio: {:.2}%", results.cache_hit_ratio * 100.0);
        println!("========================\n");
    }
    
    pub async fn run_stress_test(&self) -> Result<LoadTestResults, Box<dyn std::error::Error>> {
        println!("Running stress test - gradually increasing load...");
        
        let mut best_rps = 0.0;
        let mut stress_config = self.config.clone();
        
        // Gradually increase concurrent users
        for concurrent_users in [50, 100, 200, 500, 1000, 2000] {
            stress_config.concurrent_users = concurrent_users;
            stress_config.test_duration = Duration::from_secs(30); // Shorter for stress test
            
            println!("Testing with {} concurrent users...", concurrent_users);
            
            let tester = CasbinLoadTester {
                casbin_service: self.casbin_service.clone(),
                config: stress_config.clone(),
            };
            
            let results = tester.run_load_test().await?;
            
            if results.requests_per_second > best_rps {
                best_rps = results.requests_per_second;
            }
            
            // Check if system is showing signs of stress
            if results.failed_requests as f64 / results.total_requests as f64 > 0.05 {
                println!("System showing stress at {} concurrent users (>5% failure rate)", concurrent_users);
                break;
            }
            
            if results.p95_response_time > Duration::from_millis(1000) {
                println!("System showing stress at {} concurrent users (P95 > 1s)", concurrent_users);
                break;
            }
        }
        
        println!("Best throughput achieved: {:.2} requests/second", best_rps);
        
        // Return final test result
        self.run_load_test().await
    }
}

#[tokio::test]
async fn test_normal_load() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoadTestConfig {
        concurrent_users: 50,
        requests_per_user: 100,
        test_duration: Duration::from_secs(30),
        ramp_up_duration: Duration::from_secs(5),
        think_time: Duration::from_millis(50),
    };
    
    let tester = CasbinLoadTester::new(config).await?;
    let results = tester.run_load_test().await?;
    
    // Assertions for acceptable performance
    assert!(results.successful_requests > 0, "Should have successful requests");
    assert!(results.requests_per_second > 100.0, "Should handle at least 100 RPS");
    assert!(results.p95_response_time < Duration::from_millis(500), "P95 should be under 500ms");
    
    Ok(())
}

#[tokio::test] 
async fn test_high_load() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoadTestConfig {
        concurrent_users: 200,
        requests_per_user: 500,
        test_duration: Duration::from_secs(60),
        ramp_up_duration: Duration::from_secs(10),
        think_time: Duration::from_millis(25),
    };
    
    let tester = CasbinLoadTester::new(config).await?;
    let results = tester.run_load_test().await?;
    
    // Assertions for high load performance
    assert!(results.successful_requests > 0, "Should handle high load");
    assert!(results.requests_per_second > 500.0, "Should handle at least 500 RPS under high load");
    assert!(results.p99_response_time < Duration::from_secs(2), "P99 should be under 2s even under high load");
    
    Ok(())
}

#[tokio::test]
#[ignore] // Use --ignored to run stress tests
async fn test_stress_limits() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoadTestConfig::default();
    let tester = CasbinLoadTester::new(config).await?;
    
    let _results = tester.run_stress_test().await?;
    
    // This test finds the breaking point - no specific assertions
    // Results are logged for analysis
    
    Ok(())
}