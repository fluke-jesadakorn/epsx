#[cfg(test)]
mod firebase_user_tests {
    use std::env;
    use crate::infra::firebase_admin::FirebaseAdmin;

    #[tokio::test]
    async fn test_create_test_user() {
        // Set up environment variables for testing
        env::set_var("FIREBASE_PROJECT_ID", "epsx-449804");
        env::set_var("FIREBASE_API_KEY", "AIzaSyC8Qx2Y4k9mJ7nL3vP8tR6wE1qA5sD9fG2h"); // Placeholder - need real key
        
        let firebase_admin = FirebaseAdmin::new().await.expect("Failed to create Firebase admin");
        
        // Test user credentials
        let test_email = "jesadakorn.kirtnu@gmail.com";
        let test_password = "Aa_12345678";
        let test_display_name = "Jesadakorn Kirtnu";
        
        println!("Testing Firebase user creation...");
        
        // Try to create the user (this will fail if user already exists)
        match firebase_admin.create_user(
            Some(test_email.to_string()),
            Some(test_password.to_string()),
            Some(test_display_name.to_string())
        ).await {
            Ok(uid) => {
                println!("User created successfully with UID: {}", uid);
                
                // Set admin role for the user
                match firebase_admin.set_user_role(&uid, "admin").await {
                    Ok(_) => println!("Admin role set successfully for user {}", uid),
                    Err(e) => println!("Failed to set admin role: {}", e),
                }
            },
            Err(e) => {
                if e.to_string().contains("EMAIL_EXISTS") {
                    println!("User already exists, attempting to get existing user...");
                    
                    // Try to get existing user
                    match firebase_admin.get_user_by_email(test_email).await {
                        Ok(user) => {
                            println!("Found existing user: {} ({})", user.email.unwrap_or_default(), user.uid);
                            
                            // Ensure user has admin role
                            match firebase_admin.set_user_role(&user.uid, "admin").await {
                                Ok(_) => println!("Admin role set for existing user"),
                                Err(e) => println!("Failed to set admin role: {}", e),
                            }
                        },
                        Err(e) => println!("Failed to get existing user: {}", e),
                    }
                } else {
                    println!("Failed to create user: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_authenticate_user() {
        // Set up environment variables for testing
        env::set_var("FIREBASE_PROJECT_ID", "epsx-449804");
        env::set_var("FIREBASE_API_KEY", "AIzaSyC8Qx2Y4k9mJ7nL3vP8tR6wE1qA5sD9fG2h"); // Placeholder - need real key
        
        let firebase_admin = FirebaseAdmin::new().await.expect("Failed to create Firebase admin");
        
        // Test user credentials
        let test_email = "jesadakorn.kirtnu@gmail.com";
        let test_password = "Aa_12345678";
        
        println!("Testing Firebase authentication...");
        
        match firebase_admin.authenticate_user(test_email, test_password).await {
            Ok(user) => {
                println!("Authentication successful!");
                println!("User UID: {}", user.uid);
                println!("User email: {}", user.email.unwrap_or_default());
                println!("Email verified: {}", user.email_verified);
                println!("User has admin access: {}", firebase_admin.user_has_admin_access(&user));
            },
            Err(e) => {
                println!("Authentication failed: {}", e);
                
                // If authentication fails, let's try to understand why
                println!("Checking if user exists...");
                match firebase_admin.get_user_by_email(test_email).await {
                    Ok(user) => {
                        println!("User exists: {} ({})", user.email.unwrap_or_default(), user.uid);
                        println!("User disabled: {}", user.disabled);
                        println!("Email verified: {}", user.email_verified);
                    },
                    Err(e) => println!("User lookup failed: {}", e),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_firebase_connection() {
        // Set up environment variables for testing
        env::set_var("FIREBASE_PROJECT_ID", "epsx-449804");
        env::set_var("FIREBASE_API_KEY", "AIzaSyC8Qx2Y4k9mJ7nL3vP8tR6wE1qA5sD9fG2h"); // Placeholder - need real key
        
        match FirebaseAdmin::new().await {
            Ok(firebase_admin) => {
                println!("Firebase Admin SDK initialized successfully");
                println!("Project ID: {}", firebase_admin.project_id);
                
                // Test access token generation
                match firebase_admin.get_access_token().await {
                    Ok(token) => println!("Access token generated: {}", if token.starts_with("mock") { "MOCK TOKEN (needs configuration)" } else { "REAL TOKEN" }),
                    Err(e) => println!("Failed to get access token: {}", e),
                }
            },
            Err(e) => {
                println!("Failed to initialize Firebase Admin SDK: {}", e);
            }
        }
    }
}