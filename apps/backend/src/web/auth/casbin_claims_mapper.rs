// Casbin Claims Mapper
// Extracts user permissions from JWT tokens for Casbin authorization system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::types::Email;

/// User claims extracted from unified JWT for Casbin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasbinUserClaims {
    // Core identity
    pub user_id: String,
    pub email: Email,
    
    // IAM information
    pub role: String,
    pub permissions: Vec<String>,
    pub profile_id: Option<String>,
    
    // Provider information
    pub provider: String,
    pub provider_user_id: String,
    
    // Session context
    pub session_id: String,
    pub expires_at: i64,
    
    // Additional attributes for policy evaluation
    pub attributes: HashMap<String, String>,
}

/// Casbin subject format (user identifier for policies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasbinSubject {
    pub user_id: String,
    pub role: String,
    pub email: String,
    pub provider: String,
}

impl CasbinSubject {
    /// Convert to Casbin subject string format
    /// Format: "user:{user_id}:role:{role}:provider:{provider}"
    pub fn to_subject_string(&self) -> String {
        format!(
            "user:{}:role:{}:provider:{}",
            self.user_id,
            self.role,
            self.provider
        )
    }
    
    /// Create from subject string
    pub fn from_subject_string(subject: &str) -> Result<Self, CasbinMappingError> {
        let parts: Vec<&str> = subject.split(':').collect();
        
        if parts.len() < 6 || parts[0] != "user" || parts[2] != "role" || parts[4] != "provider" {
            return Err(CasbinMappingError::InvalidSubjectFormat(subject.to_string()));
        }
        
        Ok(CasbinSubject {
            user_id: parts[1].to_string(),
            role: parts[3].to_string(),
            email: String::new(), // Not available in subject string
            provider: parts[5].to_string(),
        })
    }
}

/// Casbin claims mapping errors
#[derive(Debug, thiserror::Error)]
pub enum CasbinMappingError {
    #[error("Invalid subject format: {0}")]
    InvalidSubjectFormat(String),
    
    #[error("Missing required claim: {0}")]
    MissingClaim(String),
    
    #[error("Invalid claim format for {field}: {value}")]
    InvalidClaimFormat { field: String, value: String },
    
    #[error("Email parsing error: {0}")]
    EmailError(String),
    
    #[error("JWT parsing error: {0}")]
    JwtError(String),
}

/// Casbin claims mapper
pub struct CasbinClaimsMapper;

impl CasbinClaimsMapper {
    /// Extract Casbin user claims from unified JWT payload
    pub fn extract_claims(jwt_payload: &serde_json::Value) -> Result<CasbinUserClaims, CasbinMappingError> {
        // Extract core identity
        let user_id = jwt_payload.get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CasbinMappingError::MissingClaim("sub".to_string()))?
            .to_string();

        let email_str = jwt_payload.get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CasbinMappingError::MissingClaim("email".to_string()))?;
            
        let email = Email::new(email_str.to_string())
            .map_err(|e| CasbinMappingError::EmailError(e.to_string()))?;

        // Extract role and permissions
        let role = jwt_payload.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string();

        let permissions = jwt_payload.get("permissions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect())
            .unwrap_or_default();

        // Extract provider information
        let provider = jwt_payload.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let provider_user_id = jwt_payload.get("provider_user_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&user_id)
            .to_string();

        // Extract session information
        let session_id = jwt_payload.get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CasbinMappingError::MissingClaim("session_id".to_string()))?
            .to_string();

        let expires_at = jwt_payload.get("exp")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| CasbinMappingError::MissingClaim("exp".to_string()))?;

        // Extract profile ID if available
        let profile_id = jwt_payload.get("profile_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Extract additional attributes
        let mut attributes = HashMap::new();
        
        // Add subscription tier
        if let Some(tier) = jwt_payload.get("subscription_tier").and_then(|v| v.as_str()) {
            attributes.insert("subscription_tier".to_string(), tier.to_string());
        }
        
        // Add organization info
        if let Some(org) = jwt_payload.get("organization").and_then(|v| v.as_str()) {
            attributes.insert("organization".to_string(), org.to_string());
        }
        
        // Add user level
        if let Some(level) = jwt_payload.get("user_level").and_then(|v| v.as_u64()) {
            attributes.insert("user_level".to_string(), level.to_string());
        }
        
        // Add custom attributes
        if let Some(custom_attrs) = jwt_payload.get("attributes").and_then(|v| v.as_object()) {
            for (key, value) in custom_attrs {
                if let Some(str_value) = value.as_str() {
                    attributes.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(CasbinUserClaims {
            user_id,
            email,
            role,
            permissions,
            profile_id,
            provider,
            provider_user_id,
            session_id,
            expires_at,
            attributes,
        })
    }

    /// Convert user claims to Casbin subject
    pub fn to_casbin_subject(claims: &CasbinUserClaims) -> CasbinSubject {
        CasbinSubject {
            user_id: claims.user_id.clone(),
            role: claims.role.clone(),
            email: claims.email.to_string(),
            provider: claims.provider.clone(),
        }
    }

    /// Extract permissions for Casbin policy evaluation
    pub fn extract_permissions(claims: &CasbinUserClaims) -> Vec<String> {
        let mut all_permissions = claims.permissions.clone();
        
        // Add role-based permissions
        let role_permissions = Self::get_role_permissions(&claims.role);
        all_permissions.extend(role_permissions);
        
        // Add attribute-based permissions
        let attribute_permissions = Self::get_attribute_permissions(&claims.attributes);
        all_permissions.extend(attribute_permissions);
        
        // Remove duplicates and sort
        all_permissions.sort();
        all_permissions.dedup();
        
        all_permissions
    }

    /// Get permissions based on user role
    fn get_role_permissions(role: &str) -> Vec<String> {
        match role {
            "admin-full-004" => vec![
                "read:*".to_string(),
                "write:*".to_string(),
                "delete:*".to_string(),
                "admin:*".to_string(),
            ],
            "moderator-standard-003" => vec![
                "read:*".to_string(),
                "write:users".to_string(),
                "write:reports".to_string(),
                "moderate:content".to_string(),
            ],
            "user-premium-002" => vec![
                "read:analytics".to_string(),
                "read:premium-data".to_string(),
                "write:portfolio".to_string(),
                "access:advanced-features".to_string(),
            ],
            "user-basic-001" => vec![
                "read:public-data".to_string(),
                "write:own-portfolio".to_string(),
                "access:basic-features".to_string(),
            ],
            _ => vec![
                "read:public-data".to_string(),
            ],
        }
    }

    /// Get permissions based on user attributes
    fn get_attribute_permissions(attributes: &HashMap<String, String>) -> Vec<String> {
        let mut permissions = Vec::new();
        
        // Subscription tier based permissions
        if let Some(tier) = attributes.get("subscription_tier") {
            match tier.as_str() {
                "enterprise" => {
                    permissions.extend(vec![
                        "access:enterprise-features".to_string(),
                        "read:enterprise-analytics".to_string(),
                        "write:enterprise-reports".to_string(),
                    ]);
                },
                "pro" => {
                    permissions.extend(vec![
                        "access:pro-features".to_string(),
                        "read:pro-analytics".to_string(),
                    ]);
                },
                _ => {}
            }
        }
        
        // Organization based permissions
        if let Some(_org) = attributes.get("organization") {
            permissions.push("access:organization-features".to_string());
        }
        
        // User level based permissions
        if let Some(level_str) = attributes.get("user_level") {
            if let Ok(level) = level_str.parse::<u32>() {
                if level >= 10 {
                    permissions.push("access:high-level-features".to_string());
                }
                if level >= 5 {
                    permissions.push("access:mid-level-features".to_string());
                }
            }
        }
        
        permissions
    }

    /// Create Casbin policy rules from user claims
    pub fn create_policy_rules(claims: &CasbinUserClaims) -> Vec<Vec<String>> {
        let subject = Self::to_casbin_subject(claims);
        let permissions = Self::extract_permissions(claims);
        
        let mut rules = Vec::new();
        
        for permission in permissions {
            // Parse permission format "action:resource" or "action:resource:condition"
            let parts: Vec<&str> = permission.split(':').collect();
            
            if parts.len() >= 2 {
                let action = parts[0];
                let resource = parts[1];
                
                // Create basic policy rule: subject, resource, action
                let rule = vec![
                    subject.to_subject_string(),
                    resource.to_string(),
                    action.to_string(),
                ];
                
                rules.push(rule);
                
                // Add condition-based rules if present
                if parts.len() > 2 {
                    let condition = parts[2];
                    let conditional_rule = vec![
                        subject.to_subject_string(),
                        resource.to_string(),
                        action.to_string(),
                        condition.to_string(),
                    ];
                    rules.push(conditional_rule);
                }
            }
        }
        
        rules
    }

    /// Validate token claims for Casbin evaluation
    pub fn validate_claims(claims: &CasbinUserClaims) -> Result<(), CasbinMappingError> {
        // Check if token is expired
        let now = chrono::Utc::now().timestamp();
        if claims.expires_at <= now {
            return Err(CasbinMappingError::InvalidClaimFormat {
                field: "exp".to_string(),
                value: claims.expires_at.to_string(),
            });
        }
        
        // Validate email format
        if claims.email.to_string().is_empty() {
            return Err(CasbinMappingError::MissingClaim("email".to_string()));
        }
        
        // Validate user ID
        if claims.user_id.is_empty() {
            return Err(CasbinMappingError::MissingClaim("user_id".to_string()));
        }
        
        // Validate session ID
        if claims.session_id.is_empty() {
            return Err(CasbinMappingError::MissingClaim("session_id".to_string()));
        }
        
        Ok(())
    }

    /// Create attribute map for ABAC (Attribute-Based Access Control)
    pub fn create_attribute_map(claims: &CasbinUserClaims) -> HashMap<String, String> {
        let mut attributes = claims.attributes.clone();
        
        // Add core attributes
        attributes.insert("user_id".to_string(), claims.user_id.clone());
        attributes.insert("email".to_string(), claims.email.to_string());
        attributes.insert("role".to_string(), claims.role.clone());
        attributes.insert("provider".to_string(), claims.provider.clone());
        attributes.insert("session_id".to_string(), claims.session_id.clone());
        
        // Add computed attributes
        let permissions_count = claims.permissions.len();
        attributes.insert("permissions_count".to_string(), permissions_count.to_string());
        
        // Add time-based attributes
        let now = chrono::Utc::now().timestamp();
        let time_until_expiry = claims.expires_at - now;
        attributes.insert("time_until_expiry".to_string(), time_until_expiry.to_string());
        
        attributes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_claims_success() {
        let jwt_payload = json!({
            "sub": "user123",
            "email": "test@example.com",
            "role": "user-premium-002",
            "permissions": ["read:analytics", "write:portfolio"],
            "provider": "firebase",
            "provider_user_id": "firebase123",
            "session_id": "sess_abc123",
            "exp": 1735689600,
            "subscription_tier": "pro",
            "user_level": 7
        });

        let claims = CasbinClaimsMapper::extract_claims(&jwt_payload).unwrap();
        
        assert_eq!(claims.user_id, "user123");
        assert_eq!(claims.email.to_string(), "test@example.com");
        assert_eq!(claims.role, "user-premium-002");
        assert_eq!(claims.permissions.len(), 2);
        assert_eq!(claims.provider, "firebase");
        assert_eq!(claims.session_id, "sess_abc123");
        assert_eq!(claims.attributes.get("subscription_tier"), Some(&"pro".to_string()));
    }

    #[test]
    fn test_casbin_subject_format() {
        let subject = CasbinSubject {
            user_id: "user123".to_string(),
            role: "admin".to_string(),
            email: "test@example.com".to_string(),
            provider: "firebase".to_string(),
        };

        let subject_string = subject.to_subject_string();
        assert_eq!(subject_string, "user:user123:role:admin:provider:firebase");

        let parsed_subject = CasbinSubject::from_subject_string(&subject_string).unwrap();
        assert_eq!(parsed_subject.user_id, "user123");
        assert_eq!(parsed_subject.role, "admin");
        assert_eq!(parsed_subject.provider, "firebase");
    }

    #[test]
    fn test_extract_permissions() {
        let claims = CasbinUserClaims {
            user_id: "user123".to_string(),
            email: Email::new("test@example.com").unwrap(),
            role: "user-premium-002".to_string(),
            permissions: vec!["read:special".to_string()],
            profile_id: None,
            provider: "firebase".to_string(),
            provider_user_id: "firebase123".to_string(),
            session_id: "sess_abc123".to_string(),
            expires_at: 1735689600,
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("subscription_tier".to_string(), "pro".to_string());
                attrs
            },
        };

        let permissions = CasbinClaimsMapper::extract_permissions(&claims);
        
        // Should include role permissions, attribute permissions, and explicit permissions
        assert!(permissions.contains(&"read:special".to_string()));
        assert!(permissions.contains(&"read:analytics".to_string()));
        assert!(permissions.contains(&"access:pro-features".to_string()));
    }

    #[test]
    fn test_create_policy_rules() {
        let claims = CasbinUserClaims {
            user_id: "user123".to_string(),
            email: Email::new("test@example.com").unwrap(),
            role: "user-basic-001".to_string(),
            permissions: vec!["read:portfolio".to_string(), "write:own-data:owner".to_string()],
            profile_id: None,
            provider: "firebase".to_string(),
            provider_user_id: "firebase123".to_string(),
            session_id: "sess_abc123".to_string(),
            expires_at: 1735689600,
            attributes: HashMap::new(),
        };

        let rules = CasbinClaimsMapper::create_policy_rules(&claims);
        
        // Should create proper policy rules
        assert!(rules.len() > 0);
        
        // Find a basic rule
        let basic_rule = rules.iter().find(|r| r.len() == 3).unwrap();
        assert_eq!(basic_rule[0], "user:user123:role:user-basic-001:provider:firebase");
        
        // Find a conditional rule
        let conditional_rule = rules.iter().find(|r| r.len() == 4).unwrap();
        assert_eq!(conditional_rule[3], "owner");
    }
}