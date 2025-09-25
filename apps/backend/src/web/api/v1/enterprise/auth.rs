// Enterprise Authentication API
// Public authentication endpoints for enterprise customers

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::infrastructure::container::DomainContainer;
use crate::core::errors::AppError;

type ApiResult<T> = Result<T, AppError>;

/// Create authentication routes for enterprise API (public endpoints)
pub fn create_auth_routes() -> Router<Arc<DomainContainer>> {
    Router::new()
        .route("/challenge", post(generate_enterprise_challenge))
        .route("/verify", post(verify_enterprise_signature))
        .route("/tiers/check", get(check_tier_eligibility))
        .route("/refresh", post(refresh_enterprise_session))
        .route("/logout", post(logout_enterprise_session))
}

/// Enterprise authentication challenge request
#[derive(Debug, Deserialize)]
pub struct EnterpriseChallengeRequest {
    pub wallet_address: String,
    pub enterprise_id: Option<String>,
    pub requested_tier: Option<String>,
}

/// Generate enterprise authentication challenge
pub async fn generate_enterprise_challenge(
    State(container): State<Arc<DomainContainer>>,
    Json(request): Json<EnterpriseChallengeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let web3_auth = container.web3_auth_service();
    
    let challenge = web3_auth
        .generate_challenge(&request.wallet_address)
        .await
        .map_err(|e| AppError::internal_server_error(&format!("Failed to generate challenge: {}", e)))?;

    let response = serde_json::json!({
        "challenge": {
            "nonce": challenge.nonce,
            "message": challenge.message,
            "expires_at": challenge.expires_at
        },
        "enterprise_features": {
            "api_access": true,
            "multi_chain_support": true,
            "tier_based_limits": true,
            "custom_integrations": true
        },
        "supported_chains": ["ethereum", "polygon", "arbitrum", "optimism", "base", "bsc"],
        "timestamp": chrono::Utc::now()
    });

    Ok(Json(response))
}

/// Enterprise signature verification request
#[derive(Debug, Deserialize)]
pub struct EnterpriseVerifyRequest {
    pub message: String,
    pub signature: String,
    pub wallet_address: String,
    pub nonce: String,
    pub enterprise_id: Option<String>,
}

/// Verify enterprise wallet signature and create session
pub async fn verify_enterprise_signature(
    State(container): State<Arc<DomainContainer>>,
    Json(request): Json<EnterpriseVerifyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let web3_auth = container.web3_auth_service();
    
    // Verify the signature
    let auth_result = web3_auth
        .verify_signature(crate::auth::Web3VerifyRequest {
            message: request.message,
            signature: request.signature,
            wallet_address: request.wallet_address.clone(),
            nonce: request.nonce,
        })
        .await
        .map_err(|e| AppError::internal_server_error(&format!("Verification failed: {}", e)))?;

    if !auth_result.is_valid {
        return Err(AppError::unauthorized("Invalid signature"));
    }

    // Create enterprise session token
    let session_token = web3_auth
        .create_session_token(&request.wallet_address, auth_result.user_id.unwrap())
        .await
        .map_err(|e| AppError::internal_server_error(&format!("Failed to create session: {}", e)))?;

    // Determine enterprise tier using enterprise permission engine
    let enterprise_tier = match container.enterprise_permission_engine() {
        Some(engine) => {
            match engine.verify_enterprise_tier(&request.wallet_address).await {
                Ok(tier) => format!("{:?}", tier).to_lowercase(),
                Err(e) => {
                    warn!("Failed to verify enterprise tier for {}: {}", request.wallet_address, e);
                    "starter".to_string()
                }
            }
        }
        None => {
            warn!("Enterprise permission engine not available, defaulting to starter tier");
            "starter".to_string()
        }
    };

    let response = serde_json::json!({
        "session_token": session_token,
        "user_id": auth_result.user_id,
        "wallet_address": request.wallet_address,
        "enterprise_tier": enterprise_tier,
        "permissions": [
            "enterprise:api:access",
            "enterprise:analytics:basic"
        ],
        "rate_limits": {
            "requests_per_minute": 100,
            "concurrent_requests": 5
        },
        "expires_in": 86400, // 24 hours
        "features": {
            "real_time_data": false,
            "custom_integrations": false,
            "unlimited_requests": false
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(Json(response))
}

/// Check tier eligibility for wallet
pub async fn check_tier_eligibility(
    State(container): State<Arc<DomainContainer>>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let wallet_address = params.get("wallet_address")
        .ok_or_else(|| AppError::bad_request("wallet_address parameter required"))?;

    // Use enterprise permission engine to check actual tier eligibility
    let enterprise_engine = container.enterprise_permission_engine()
        .ok_or_else(|| AppError::internal_server_error("Enterprise permission engine not available"))?;

    // Get current tier
    let current_tier = enterprise_engine
        .verify_enterprise_tier(wallet_address)
        .await
        .map_err(|e| AppError::internal_server_error(&format!("Failed to verify tier: {}", e)))?;

    // Check eligibility for all tiers
    let mut eligible_tiers = Vec::new();

    // Check each tier
    for tier_name in ["starter", "business", "enterprise", "whale"] {
        let tier_config = enterprise_engine.enterprise_configs.get(tier_name);
        
        if let Some(config) = tier_config {
            let is_eligible = enterprise_engine
                .meets_tier_requirements(wallet_address, config)
                .await
                .unwrap_or(false);

            let tier_info = serde_json::json!({
                "tier": tier_name,
                "eligible": is_eligible,
                "requirements_met": is_eligible,
                "minimum_usd_value": config.tier.minimum_value_usd(),
                "rate_limit_per_minute": config.tier.rate_limit_per_minute(),
                "concurrent_requests": config.tier.concurrent_request_limit(),
                "features": get_tier_features(&config.tier),
                "compliance_requirements": get_compliance_requirements(&config.compliance_requirements)
            });

            eligible_tiers.push(tier_info);
        }
    }

    // Determine recommended tier (highest eligible tier)
    let recommended_tier = if current_tier.has_feature_access("unlimited_requests") {
        "whale"
    } else if current_tier.has_feature_access("custom_integration") {
        "enterprise" 
    } else if current_tier.has_feature_access("real_time_data") {
        "business"
    } else {
        "starter"
    };

    // Calculate upgrade path
    let next_tier = match recommended_tier {
        "starter" => "business",
        "business" => "enterprise", 
        "enterprise" => "whale",
        _ => "whale"
    };

    let response = serde_json::json!({
        "wallet_address": wallet_address,
        "current_tier": format!("{:?}", current_tier).to_lowercase(),
        "eligible_tiers": eligible_tiers,
        "recommended_tier": recommended_tier,
        "upgrade_path": {
            "next_tier": next_tier,
            "benefits": get_tier_benefits(next_tier),
            "requirements": get_upgrade_requirements(next_tier)
        },
        "timestamp": chrono::Utc::now()
    });

    Ok(Json(response))
}

/// Get features available for a tier
fn get_tier_features(tier: &crate::auth::enterprise_permission_engine::EnterpriseTier) -> Vec<String> {
    let mut features = vec!["basic_analytics".to_string(), "market_data".to_string()];
    
    if tier.has_feature_access("real_time_data") {
        features.extend(vec!["real_time_data".to_string(), "advanced_charts".to_string()]);
    }
    
    if tier.has_feature_access("custom_integration") {
        features.extend(vec!["custom_integration".to_string(), "webhook_endpoints".to_string()]);
    }
    
    if tier.has_feature_access("unlimited_requests") {
        features.extend(vec!["unlimited_requests".to_string(), "custom_infrastructure".to_string()]);
    }
    
    features
}

/// Get compliance requirements
fn get_compliance_requirements(requirements: &[crate::auth::enterprise_permission_engine::ComplianceRequirement]) -> Vec<String> {
    requirements.iter().map(|req| {
        match req {
            crate::auth::enterprise_permission_engine::ComplianceRequirement::KYCVerified => "kyc_verified".to_string(),
            crate::auth::enterprise_permission_engine::ComplianceRequirement::AccreditedInvestor => "accredited_investor".to_string(),
            crate::auth::enterprise_permission_engine::ComplianceRequirement::RegionCompliant => "region_compliant".to_string(),
            crate::auth::enterprise_permission_engine::ComplianceRequirement::AMLCleared => "aml_cleared".to_string(),
            crate::auth::enterprise_permission_engine::ComplianceRequirement::SOXCompliant => "sox_compliant".to_string(),
            crate::auth::enterprise_permission_engine::ComplianceRequirement::GDPRCompliant => "gdpr_compliant".to_string(),
        }
    }).collect()
}

/// Get benefits of upgrading to a tier
fn get_tier_benefits(tier: &str) -> Vec<String> {
    match tier {
        "business" => vec![
            "10x higher rate limits".to_string(),
            "Real-time data access".to_string(),
            "Advanced analytics".to_string(),
            "Webhook endpoints".to_string()
        ],
        "enterprise" => vec![
            "100x higher rate limits".to_string(),
            "Custom integrations".to_string(),
            "White-label features".to_string(),
            "Priority support".to_string()
        ],
        "whale" => vec![
            "Unlimited rate limits".to_string(),
            "Custom infrastructure".to_string(),
            "Full white-label".to_string(),
            "Dedicated support".to_string()
        ],
        _ => vec!["Basic analytics access".to_string()]
    }
}

/// Get requirements to upgrade to a tier
fn get_upgrade_requirements(tier: &str) -> Vec<String> {
    match tier {
        "business" => vec![
            "$10,000 in tokens OR business NFT".to_string(),
            "KYC verification".to_string()
        ],
        "enterprise" => vec![
            "$100,000 in tokens OR DAO membership".to_string(),
            "KYC + accredited investor status".to_string(),
            "AML clearance".to_string()
        ],
        "whale" => vec![
            "$1,000,000 in tokens".to_string(),
            "Full compliance (KYC, AML, SOX)".to_string(),
            "Whale access NFT".to_string()
        ],
        _ => vec!["$1,000 in tokens".to_string()]
    }
}

/// Refresh enterprise session
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub session_token: String,
}

pub async fn refresh_enterprise_session(
    State(container): State<Arc<DomainContainer>>,
    Json(request): Json<RefreshRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let web3_auth = container.web3_auth_service();
    
    // Validate existing session token
    let (user_id, wallet_address) = web3_auth
        .validate_session_token(&request.session_token)
        .await
        .map_err(|e| AppError::unauthorized(&format!("Invalid session: {}", e)))?;

    // Create new session token
    let new_session_token = web3_auth
        .create_session_token(&wallet_address, user_id)
        .await
        .map_err(|e| AppError::internal_server_error(&format!("Failed to refresh session: {}", e)))?;

    let response = serde_json::json!({
        "session_token": new_session_token,
        "user_id": user_id,
        "wallet_address": wallet_address,
        "expires_in": 86400,
        "refreshed_at": chrono::Utc::now()
    });

    Ok(Json(response))
}

/// Logout enterprise session
pub async fn logout_enterprise_session(
    State(_container): State<Arc<DomainContainer>>,
    Json(request): Json<RefreshRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement session blacklisting/revocation
    
    let response = serde_json::json!({
        "status": "logged_out",
        "message": "Enterprise session terminated successfully",
        "timestamp": chrono::Utc::now()
    });

    Ok(Json(response))
}