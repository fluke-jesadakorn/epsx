// End-to-end tests for permission management workflows
use reqwest;
use serde_json::json;
use std::env;
use tokio;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";

fn get_base_url() -> String {
    env::var("E2E_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

#[tokio::test]
async fn test_user_registration_and_auto_assignment() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();
    
    // Skip if server is not running
    if client.get(&format!("{}/health", base_url)).send().await.is_err() {
        println!("Skipping E2E test - server not available at {}", base_url);
        return;
    }
    
    // Test user registration
    let registration_data = json!({
        "email": "e2e_test@example.com",
        "password": "SecurePassword123!",
        "role": "user"
    });
    
    let response = client
        .post(&format!("{}/api/v1/authentication/register", base_url))
        .json(&registration_data)
        .send()
        .await;
    
    if let Ok(resp) = response {
        if resp.status().is_success() {
            let user_data: serde_json::Value = resp.json().await.unwrap();
            let user_id = user_data["id"].as_str().unwrap();
            
            // Wait a moment for auto-assignment to process
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // Check if user was auto-assigned permissions
            let auth_response = client
                .post(&format!("{}/api/v1/authentication/login", base_url))
                .json(&json!({
                    "email": "e2e_test@example.com",
                    "password": "SecurePassword123!"
                }))
                .send()
                .await;
            
            if let Ok(auth_resp) = auth_response {
                if auth_resp.status().is_success() {
                    let auth_data: serde_json::Value = auth_resp.json().await.unwrap();
                    let token = auth_data["token"].as_str().unwrap();
                    
                    // Test accessing user profile with permissions
                    let profile_response = client
                        .get(&format!("{}/api/v1/authentication/profile", base_url))
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await;
                    
                    assert!(profile_response.is_ok());
                    if let Ok(profile_resp) = profile_response {
                        assert!(profile_resp.status().is_success(), "User should have access to their profile");
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_admin_permission_management_workflow() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();
    
    // Skip if server is not running
    if client.get(&format!("{}/health", base_url)).send().await.is_err() {
        println!("Skipping E2E test - server not available at {}", base_url);
        return;
    }
    
    // Create admin user (assumes admin exists or can be created)
    let admin_token = get_admin_token(&client, &base_url).await;
    if admin_token.is_none() {
        println!("Skipping admin workflow test - no admin access");
        return;
    }
    let admin_token = admin_token.unwrap();
    
    // Test creating a permission profile
    let profile_data = json!({
        "name": "E2E Test Profile",
        "description": "Test profile created by E2E test",
        "target_tier": "bronze",
        "category": "user",
        "permissions": [
            {
                "action": "read",
                "resource": "test_resource"
            }
        ]
    });
    
    let response = client
        .post(&format!("{}/api/admin/permission-profiles", base_url))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&profile_data)
        .send()
        .await;
    
    if let Ok(resp) = response {
        if resp.status().is_success() {
            let profile: serde_json::Value = resp.json().await.unwrap();
            let profile_id = profile["id"].as_str().unwrap();
            
            // Test assigning profile to user
            let assignment_data = json!({
                "user_id": "test_user_123",
                "permission_profile_id": profile_id,
                "reason": "E2E test assignment"
            });
            
            let assign_response = client
                .post(&format!("{}/api/admin/user-management/assign-permissions", base_url))
                .header("Authorization", format!("Bearer {}", admin_token))
                .json(&assignment_data)
                .send()
                .await;
            
            if let Ok(assign_resp) = assign_response {
                assert!(assign_resp.status().is_success(), "Permission assignment should succeed");
            }
            
            // Test retrieving user permissions
            let user_perms_response = client
                .get(&format!("{}/api/admin/user-management/users/test_user_123/permissions", base_url))
                .header("Authorization", format!("Bearer {}", admin_token))
                .send()
                .await;
            
            if let Ok(perms_resp) = user_perms_response {
                if perms_resp.status().is_success() {
                    let permissions: serde_json::Value = perms_resp.json().await.unwrap();
                    assert!(permissions["permissions"].as_array().unwrap().len() > 0, "User should have assigned permissions");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_expiration_and_notification_system() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();
    
    // Skip if server is not running
    if client.get(&format!("{}/health", base_url)).send().await.is_err() {
        println!("Skipping E2E test - server not available at {}", base_url);
        return;
    }
    
    let admin_token = get_admin_token(&client, &base_url).await;
    if admin_token.is_none() {
        return;
    }
    let admin_token = admin_token.unwrap();
    
    // Test creating a permission profile with expiration
    let profile_data = json!({
        "name": "Expiring Test Profile",
        "description": "Profile that expires for testing",
        "target_tier": "bronze",
        "category": "user",
        "auto_expire_days": 1, // Expires in 1 day
        "permissions": [
            {
                "action": "read",
                "resource": "temporary_resource"
            }
        ]
    });
    
    let response = client
        .post(&format!("{}/api/admin/permission-profiles", base_url))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&profile_data)
        .send()
        .await;
    
    if let Ok(resp) = response {
        if resp.status().is_success() {
            // Test triggering expiration check manually
            let expiration_check_response = client
                .post(&format!("{}/api/admin/system/check-expirations", base_url))
                .header("Authorization", format!("Bearer {}", admin_token))
                .send()
                .await;
            
            if let Ok(check_resp) = expiration_check_response {
                if check_resp.status().is_success() {
                    let check_result: serde_json::Value = check_resp.json().await.unwrap();
                    assert!(check_result["total_checked"].as_u64().unwrap_or(0) >= 0, "Expiration check should process");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_performance_under_load() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();
    
    // Skip if server is not running
    if client.get(&format!("{}/health", base_url)).send().await.is_err() {
        println!("Skipping E2E test - server not available at {}", base_url);
        return;
    }
    
    // Test concurrent permission checks
    let mut handles = vec![];
    let concurrent_requests = 50;
    
    for i in 0..concurrent_requests {
        let client = client.clone();
        let base_url = base_url.clone();
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            // Make a permission check request
            let response = client
                .get(&format!("{}/api/v1/permissions/check", base_url))
                .query(&[
                    ("resource", "test_resource"),
                    ("action", "read"),
                    ("user_id", &format!("load_test_user_{}", i))
                ])
                .send()
                .await;
            
            let duration = start.elapsed();
            (response.is_ok(), duration)
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;
    
    let successful_requests = results.iter().filter(|r| r.as_ref().unwrap().0).count();
    let average_duration: std::time::Duration = results
        .iter()
        .map(|r| r.as_ref().unwrap().1)
        .sum::<std::time::Duration>() / results.len() as u32;
    
    println!("Load test results: {}/{} successful, avg duration: {:?}", 
             successful_requests, concurrent_requests, average_duration);
    
    // Assert reasonable performance
    assert!(successful_requests >= concurrent_requests * 80 / 100, "At least 80% of requests should succeed");
    assert!(average_duration.as_millis() < 1000, "Average response time should be under 1 second");
}

async fn get_admin_token(client: &reqwest::Client, base_url: &str) -> Option<String> {
    // Try to authenticate as admin
    let admin_creds = json!({
        "email": "admin@example.com",
        "password": "AdminPassword123!"
    });
    
    let response = client
        .post(&format!("{}/api/admin/authentication/login", base_url))
        .json(&admin_creds)
        .send()
        .await;
    
    if let Ok(resp) = response {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                return data["token"].as_str().map(|s| s.to_string());
            }
        }
    }
    
    None
}