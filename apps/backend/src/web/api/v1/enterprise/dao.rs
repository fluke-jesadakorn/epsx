// Enterprise DAO API endpoints
// Comprehensive DAO governance for Web3 enterprise authentication

use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};

use crate::auth::{
    EnterpriseDAOService, EnterpriseDAOResult, DAOMembership, GovernancePowerSummary,
    GovernanceTier, VotingHistory, ProposalActivity, DelegationStatus
};
use crate::infrastructure::container::AppState;
use crate::web::middleware::web3_enterprise_auth::Web3AuthenticatedUser;

/// DAO governance verification request
#[derive(Debug, Deserialize)]
pub struct DAOVerificationRequest {
    pub wallet_address: String,
    pub include_history: Option<bool>,
    pub include_proposals: Option<bool>,
    pub include_delegations: Option<bool>,
}

/// DAO governance verification response
#[derive(Debug, Serialize)]
pub struct DAOVerificationResponse {
    pub success: bool,
    pub governance_result: EnterpriseDAOResult,
    pub verification_timestamp: DateTime<Utc>,
    pub enterprise_tier_upgrade: Option<String>,
    pub additional_benefits: Vec<String>,
}

/// DAO membership query parameters
#[derive(Debug, Deserialize)]
pub struct DAOMembershipQuery {
    pub dao_name: Option<String>,
    pub network: Option<String>,
    pub membership_type: Option<String>,
    pub include_inactive: Option<bool>,
}

/// DAO membership list response
#[derive(Debug, Serialize)]
pub struct DAOMembershipListResponse {
    pub success: bool,
    pub memberships: Vec<DAOMembership>,
    pub total_count: u32,
    pub governance_summary: GovernancePowerSummary,
}

/// Governance metrics query parameters
#[derive(Debug, Deserialize)]
pub struct GovernanceMetricsQuery {
    pub period_days: Option<u32>,
    pub include_cross_chain: Option<bool>,
    pub min_participation: Option<f32>,
}

/// Governance metrics response
#[derive(Debug, Serialize)]
pub struct GovernanceMetricsResponse {
    pub success: bool,
    pub voting_history: VotingHistory,
    pub proposal_activity: ProposalActivity,
    pub delegation_status: DelegationStatus,
    pub compliance_score: f32,
    pub period_summary: PeriodSummary,
}

#[derive(Debug, Serialize)]
pub struct PeriodSummary {
    pub period_days: u32,
    pub total_governance_actions: u32,
    pub dao_participation_count: u32,
    pub leadership_roles_held: u32,
    pub governance_influence_score: f32,
}

/// DAO proposal creation request
#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub dao_name: String,
    pub title: String,
    pub description: String,
    pub proposal_type: String,
    pub target_contracts: Vec<String>,
    pub execution_payload: Option<String>,
    pub voting_period_override: Option<u32>,
}

/// DAO proposal creation response
#[derive(Debug, Serialize)]
pub struct CreateProposalResponse {
    pub success: bool,
    pub proposal_id: String,
    pub estimated_gas: u64,
    pub voting_start: DateTime<Utc>,
    pub voting_end: DateTime<Utc>,
    pub required_quorum: String,
}

/// Vote casting request
#[derive(Debug, Deserialize)]
pub struct CastVoteRequest {
    pub dao_name: String,
    pub proposal_id: String,
    pub vote_choice: String, // "for", "against", "abstain"
    pub vote_reason: Option<String>,
    pub voting_power_override: Option<String>,
}

/// Vote casting response
#[derive(Debug, Serialize)]
pub struct CastVoteResponse {
    pub success: bool,
    pub transaction_hash: String,
    pub voting_power_used: String,
    pub vote_weight: f32,
    pub proposal_status_update: String,
}

/// Delegation request
#[derive(Debug, Deserialize)]
pub struct DelegateVotingPowerRequest {
    pub dao_name: String,
    pub delegate_address: String,
    pub delegation_amount: String,
    pub delegation_period: Option<u32>, // Days
}

/// Delegation response
#[derive(Debug, Serialize)]
pub struct DelegateVotingPowerResponse {
    pub success: bool,
    pub transaction_hash: String,
    pub delegated_amount: String,
    pub delegate_reputation: f32,
    pub estimated_delegation_efficiency: f32,
}

/// Error response for DAO operations
#[derive(Debug, Serialize)]
pub struct DAOErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub details: Option<String>,
    pub recovery_suggestions: Vec<String>,
}

/// Create DAO API router
pub fn create_dao_router() -> Router<AppState> {
    Router::new()
        .route("/governance/verify", post(verify_dao_governance))
        .route("/governance/metrics", get(get_governance_metrics))
        .route("/memberships", get(get_dao_memberships))
        .route("/memberships/:dao_name", get(get_dao_membership_details))
        .route("/proposals/create", post(create_dao_proposal))
        .route("/proposals/:dao_name/:proposal_id/vote", post(cast_vote))
        .route("/delegation/delegate", post(delegate_voting_power))
        .route("/delegation/revoke", post(revoke_delegation))
        .route("/analytics/dashboard", get(get_governance_dashboard))
}

/// Verify comprehensive DAO governance for enterprise authentication
async fn verify_dao_governance(
    State(state): State<AppState>,
    user: Web3AuthenticatedUser,
    Json(request): Json<DAOVerificationRequest>,
) -> Result<Json<DAOVerificationResponse>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("DAO governance verification requested for wallet: {}", request.wallet_address);

    // Verify the requesting user has permission to check this wallet
    if user.wallet_address != request.wallet_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(DAOErrorResponse {
                success: false,
                error: "Permission denied".to_string(),
                error_code: "WALLET_MISMATCH".to_string(),
                details: Some("Can only verify governance for your own wallet".to_string()),
                recovery_suggestions: vec!["Use the wallet address from your authentication".to_string()],
            })
        ));
    }

    // Get DAO service from state
    let dao_service = &state.dao_service;

    // Perform comprehensive DAO governance verification
    match dao_service.verify_enterprise_dao_governance(&request.wallet_address).await {
        Ok(governance_result) => {
            // Determine if governance activity qualifies for tier upgrade
            let enterprise_tier_upgrade = determine_tier_upgrade(&governance_result);
            
            // Calculate additional benefits from governance participation
            let additional_benefits = calculate_governance_benefits(&governance_result);

            Ok(Json(DAOVerificationResponse {
                success: true,
                governance_result,
                verification_timestamp: Utc::now(),
                enterprise_tier_upgrade,
                additional_benefits,
            }))
        }
        Err(e) => {
            warn!("DAO governance verification failed for {}: {}", request.wallet_address, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DAOErrorResponse {
                    success: false,
                    error: "Governance verification failed".to_string(),
                    error_code: "VERIFICATION_ERROR".to_string(),
                    details: Some(e.to_string()),
                    recovery_suggestions: vec![
                        "Ensure wallet has governance tokens".to_string(),
                        "Try again in a few minutes".to_string(),
                    ],
                })
            ))
        }
    }
}

/// Get detailed governance metrics for authenticated user
async fn get_governance_metrics(
    State(state): State<AppState>,
    user: Web3AuthenticatedUser,
    Query(query): Query<GovernanceMetricsQuery>,
) -> Result<Json<GovernanceMetricsResponse>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("Governance metrics requested for wallet: {}", user.wallet_address);

    let dao_service = &state.dao_service;
    let period_days = query.period_days.unwrap_or(90);

    match dao_service.verify_enterprise_dao_governance(&user.wallet_address).await {
        Ok(governance_result) => {
            // Calculate period-specific summary
            let period_summary = calculate_period_summary(&governance_result, period_days);

            Ok(Json(GovernanceMetricsResponse {
                success: true,
                voting_history: governance_result.voting_history.clone(),
                proposal_activity: governance_result.proposal_activity.clone(),
                delegation_status: governance_result.delegation_status.clone(),
                compliance_score: governance_result.compliance_score,
                period_summary,
            }))
        }
        Err(e) => {
            warn!("Governance metrics fetch failed for {}: {}", user.wallet_address, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DAOErrorResponse {
                    success: false,
                    error: "Failed to fetch governance metrics".to_string(),
                    error_code: "METRICS_ERROR".to_string(),
                    details: Some(e.to_string()),
                    recovery_suggestions: vec!["Try again later".to_string()],
                })
            ))
        }
    }
}

/// Get DAO memberships for authenticated user
async fn get_dao_memberships(
    State(state): State<AppState>,
    user: Web3AuthenticatedUser,
    Query(query): Query<DAOMembershipQuery>,
) -> Result<Json<DAOMembershipListResponse>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("DAO memberships requested for wallet: {}", user.wallet_address);

    let dao_service = &state.dao_service;

    match dao_service.verify_enterprise_dao_governance(&user.wallet_address).await {
        Ok(governance_result) => {
            // Filter memberships based on query parameters
            let filtered_memberships = filter_memberships(
                governance_result.dao_memberships,
                &query
            );

            Ok(Json(DAOMembershipListResponse {
                success: true,
                memberships: filtered_memberships.clone(),
                total_count: filtered_memberships.len() as u32,
                governance_summary: governance_result.governance_power,
            }))
        }
        Err(e) => {
            warn!("DAO memberships fetch failed for {}: {}", user.wallet_address, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DAOErrorResponse {
                    success: false,
                    error: "Failed to fetch DAO memberships".to_string(),
                    error_code: "MEMBERSHIPS_ERROR".to_string(),
                    details: Some(e.to_string()),
                    recovery_suggestions: vec!["Try again later".to_string()],
                })
            ))
        }
    }
}

/// Get detailed membership information for specific DAO
async fn get_dao_membership_details(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Path(dao_name): Path<String>,
) -> Result<Json<DAOMembership>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("DAO membership details requested for {}: {}", user.wallet_address, dao_name);

    // TODO: Implement detailed membership lookup
    // For now, return a placeholder error
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(DAOErrorResponse {
            success: false,
            error: "Detailed membership lookup not yet implemented".to_string(),
            error_code: "NOT_IMPLEMENTED".to_string(),
            details: Some("This feature is coming soon".to_string()),
            recovery_suggestions: vec!["Use the general memberships endpoint".to_string()],
        })
    ))
}

/// Create a new DAO proposal (enterprise feature)
async fn create_dao_proposal(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Json(_request): Json<CreateProposalRequest>,
) -> Result<Json<CreateProposalResponse>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("DAO proposal creation requested by: {}", user.wallet_address);

    // TODO: Implement proposal creation
    // This requires integration with specific DAO frameworks
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(DAOErrorResponse {
            success: false,
            error: "Proposal creation not yet implemented".to_string(),
            error_code: "NOT_IMPLEMENTED".to_string(),
            details: Some("This enterprise feature is coming soon".to_string()),
            recovery_suggestions: vec!["Create proposals directly through DAO interfaces for now".to_string()],
        })
    ))
}

/// Cast vote on DAO proposal
async fn cast_vote(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Path((_dao_name, _proposal_id)): Path<(String, String)>,
    Json(_request): Json<CastVoteRequest>,
) -> Result<Json<CastVoteResponse>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("Vote casting requested by: {}", user.wallet_address);

    // TODO: Implement vote casting
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(DAOErrorResponse {
            success: false,
            error: "Vote casting not yet implemented".to_string(),
            error_code: "NOT_IMPLEMENTED".to_string(),
            details: Some("This enterprise feature is coming soon".to_string()),
            recovery_suggestions: vec!["Vote directly through DAO interfaces for now".to_string()],
        })
    ))
}

/// Delegate voting power to another address
async fn delegate_voting_power(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Json(_request): Json<DelegateVotingPowerRequest>,
) -> Result<Json<DelegateVotingPowerResponse>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("Voting power delegation requested by: {}", user.wallet_address);

    // TODO: Implement delegation
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(DAOErrorResponse {
            success: false,
            error: "Delegation not yet implemented".to_string(),
            error_code: "NOT_IMPLEMENTED".to_string(),
            details: Some("This enterprise feature is coming soon".to_string()),
            recovery_suggestions: vec!["Delegate directly through DAO interfaces for now".to_string()],
        })
    ))
}

/// Revoke delegation
async fn revoke_delegation(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("Delegation revocation requested by: {}", user.wallet_address);

    // TODO: Implement delegation revocation
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(DAOErrorResponse {
            success: false,
            error: "Delegation revocation not yet implemented".to_string(),
            error_code: "NOT_IMPLEMENTED".to_string(),
            details: Some("This enterprise feature is coming soon".to_string()),
            recovery_suggestions: vec!["Revoke delegation directly through DAO interfaces for now".to_string()],
        })
    ))
}

/// Get comprehensive governance dashboard
async fn get_governance_dashboard(
    State(state): State<AppState>,
    user: Web3AuthenticatedUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<DAOErrorResponse>)> {
    info!("Governance dashboard requested for: {}", user.wallet_address);

    let dao_service = &state.dao_service;

    match dao_service.verify_enterprise_dao_governance(&user.wallet_address).await {
        Ok(governance_result) => {
            // Create comprehensive dashboard data
            let dashboard = serde_json::json!({
                "success": true,
                "wallet_address": user.wallet_address,
                "governance_tier": governance_result.governance_power.governance_tier,
                "compliance_score": governance_result.compliance_score,
                "dao_summary": {
                    "active_memberships": governance_result.dao_memberships.len(),
                    "delegate_roles": governance_result.governance_power.delegate_role_count,
                    "total_voting_power": governance_result.governance_power.total_voting_power,
                    "cross_dao_influence": governance_result.governance_power.cross_dao_influence
                },
                "activity_summary": {
                    "total_votes": governance_result.voting_history.total_votes_cast,
                    "proposals_created": governance_result.proposal_activity.proposals_created,
                    "success_rate": governance_result.proposal_activity.proposal_success_rate,
                    "participation_rate": governance_result.voting_history.vote_participation_rate
                },
                "delegation_summary": {
                    "is_delegate": governance_result.delegation_status.is_delegate,
                    "delegators_count": governance_result.delegation_status.delegators_count,
                    "reputation_score": governance_result.delegation_status.reputation_score
                },
                "enterprise_benefits": governance_result.enterprise_benefits,
                "last_updated": governance_result.verification_timestamp
            });

            Ok(Json(dashboard))
        }
        Err(e) => {
            warn!("Governance dashboard failed for {}: {}", user.wallet_address, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DAOErrorResponse {
                    success: false,
                    error: "Failed to load governance dashboard".to_string(),
                    error_code: "DASHBOARD_ERROR".to_string(),
                    details: Some(e.to_string()),
                    recovery_suggestions: vec!["Try refreshing the page".to_string()],
                })
            ))
        }
    }
}

// Helper functions

fn determine_tier_upgrade(governance_result: &EnterpriseDAOResult) -> Option<String> {
    match governance_result.governance_power.governance_tier {
        GovernanceTier::Founder | GovernanceTier::Leader => Some("Enterprise".to_string()),
        GovernanceTier::Delegate => Some("Business".to_string()),
        _ => None,
    }
}

fn calculate_governance_benefits(governance_result: &EnterpriseDAOResult) -> Vec<String> {
    let mut benefits = governance_result.enterprise_benefits.clone();
    
    // Add dynamic benefits based on current state
    if governance_result.compliance_score > 0.8 {
        benefits.push("governance:compliance:high".to_string());
    }
    
    if governance_result.governance_power.active_daos >= 5 {
        benefits.push("governance:multi_dao:expert".to_string());
    }
    
    benefits
}

fn calculate_period_summary(governance_result: &EnterpriseDAOResult, period_days: u32) -> PeriodSummary {
    PeriodSummary {
        period_days,
        total_governance_actions: governance_result.voting_history.total_votes_cast + 
                                 governance_result.proposal_activity.proposals_created,
        dao_participation_count: governance_result.governance_power.active_daos,
        leadership_roles_held: governance_result.governance_power.delegate_role_count,
        governance_influence_score: governance_result.governance_power.cross_dao_influence,
    }
}

fn filter_memberships(memberships: Vec<DAOMembership>, query: &DAOMembershipQuery) -> Vec<DAOMembership> {
    memberships.into_iter().filter(|membership| {
        // Filter by DAO name if specified
        if let Some(dao_name) = &query.dao_name {
            if membership.dao_name.to_lowercase() != dao_name.to_lowercase() {
                return false;
            }
        }
        
        // Filter by network if specified
        if let Some(network) = &query.network {
            if membership.network.to_lowercase() != network.to_lowercase() {
                return false;
            }
        }
        
        // Filter by membership type if specified
        if let Some(membership_type) = &query.membership_type {
            if format!("{:?}", membership.membership_type).to_lowercase() != membership_type.to_lowercase() {
                return false;
            }
        }
        
        true
    }).collect()
}