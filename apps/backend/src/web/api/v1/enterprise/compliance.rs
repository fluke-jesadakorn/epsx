// Enterprise Compliance API endpoints
// Comprehensive compliance verification for Web3 enterprise authentication

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
    EnterpriseComplianceService, EnterpriseComplianceResult, ComplianceStatus,
    KYCVerification, AMLScreening, RiskAssessment, EnterpriseUserProfile
};
use crate::infrastructure::container::AppState;
use crate::web::middleware::web3_enterprise_auth::Web3AuthenticatedUser;

/// Compliance verification request
#[derive(Debug, Deserialize)]
pub struct ComplianceVerificationRequest {
    pub wallet_address: String,
    pub user_profile: Option<EnterpriseUserProfile>,
    pub verification_level: Option<String>, // basic, standard, enhanced
    pub include_detailed_report: Option<bool>,
}

/// Compliance verification response
#[derive(Debug, Serialize)]
pub struct ComplianceVerificationResponse {
    pub success: bool,
    pub compliance_result: EnterpriseComplianceResult,
    pub verification_timestamp: DateTime<Utc>,
    pub tier_eligibility: TierEligibilityResult,
    pub recommended_actions: Vec<String>,
    pub next_review_date: DateTime<Utc>,
}

/// Tier eligibility result
#[derive(Debug, Serialize)]
pub struct TierEligibilityResult {
    pub starter_eligible: bool,
    pub business_eligible: bool,
    pub enterprise_eligible: bool,
    pub whale_eligible: bool,
    pub current_maximum_tier: String,
    pub blocking_factors: Vec<String>,
}

/// KYC initiation request
#[derive(Debug, Deserialize)]
pub struct KYCInitiationRequest {
    pub provider: String, // jumio, onfido, sumsub
    pub verification_level: String, // basic, standard, enhanced
    pub user_profile: EnterpriseUserProfile,
    pub callback_url: Option<String>,
}

/// KYC initiation response
#[derive(Debug, Serialize)]
pub struct KYCInitiationResponse {
    pub success: bool,
    pub kyc_session_id: String,
    pub verification_url: String,
    pub required_documents: Vec<String>,
    pub estimated_duration_minutes: u32,
    pub expires_at: DateTime<Utc>,
}

/// AML screening request
#[derive(Debug, Deserialize)]
pub struct AMLScreeningRequest {
    pub wallet_address: String,
    pub full_screening: Option<bool>,
    pub include_transaction_monitoring: Option<bool>,
    pub screening_depth: Option<String>, // basic, standard, comprehensive
}

/// AML screening response
#[derive(Debug, Serialize)]
pub struct AMLScreeningResponse {
    pub success: bool,
    pub screening_result: AMLScreening,
    pub screening_timestamp: DateTime<Utc>,
    pub requires_further_investigation: bool,
    pub escalation_required: bool,
    pub next_screening_date: DateTime<Utc>,
}

/// Compliance status query parameters
#[derive(Debug, Deserialize)]
pub struct ComplianceStatusQuery {
    pub include_history: Option<bool>,
    pub include_risk_assessment: Option<bool>,
    pub date_range_days: Option<u32>,
}

/// Compliance dashboard response
#[derive(Debug, Serialize)]
pub struct ComplianceDashboardResponse {
    pub success: bool,
    pub wallet_address: String,
    pub overall_compliance_status: ComplianceStatus,
    pub compliance_score: f32,
    pub risk_level: String,
    pub summary: ComplianceSummary,
    pub recent_activities: Vec<ComplianceActivity>,
    pub required_actions: Vec<ComplianceAction>,
    pub expiring_verifications: Vec<ExpiringVerification>,
    pub dashboard_timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ComplianceSummary {
    pub kyc_status: String,
    pub aml_status: String,
    pub sanctions_status: String,
    pub geographic_compliance: bool,
    pub accredited_investor: bool,
    pub data_privacy_compliant: bool,
    pub regulatory_compliant: u32, // number of regulations compliant with
    pub total_regulations: u32,
}

#[derive(Debug, Serialize)]
pub struct ComplianceActivity {
    pub activity_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub impact_level: String,
}

#[derive(Debug, Serialize)]
pub struct ComplianceAction {
    pub action_type: String,
    pub description: String,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_duration: String,
}

#[derive(Debug, Serialize)]
pub struct ExpiringVerification {
    pub verification_type: String,
    pub provider: String,
    pub expires_at: DateTime<Utc>,
    pub days_until_expiry: i64,
    pub renewal_required: bool,
}

/// Regulatory requirements query
#[derive(Debug, Deserialize)]
pub struct RegulatoryRequirementsQuery {
    pub jurisdiction: Option<String>,
    pub business_type: Option<String>,
    pub include_recommendations: Option<bool>,
}

/// Regulatory requirements response
#[derive(Debug, Serialize)]
pub struct RegulatoryRequirementsResponse {
    pub success: bool,
    pub applicable_regulations: Vec<RegulatoryRequirement>,
    pub compliance_roadmap: Vec<ComplianceRoadmapItem>,
    pub estimated_compliance_cost: Option<f32>,
    pub estimated_timeline_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct RegulatoryRequirement {
    pub regulation_name: String,
    pub regulation_type: String,
    pub jurisdiction: String,
    pub description: String,
    pub mandatory: bool,
    pub penalties: String,
    pub compliance_steps: Vec<String>,
    pub documentation_required: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ComplianceRoadmapItem {
    pub step_number: u32,
    pub step_name: String,
    pub description: String,
    pub estimated_duration_days: u32,
    pub dependencies: Vec<String>,
    pub cost_estimate: Option<f32>,
    pub priority: String,
}

/// Error response for compliance operations
#[derive(Debug, Serialize)]
pub struct ComplianceErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub details: Option<String>,
    pub recovery_suggestions: Vec<String>,
    pub support_reference: Option<String>,
}

/// Create compliance API router
pub fn create_compliance_router() -> Router<AppState> {
    Router::new()
        .route("/verify", post(verify_compliance))
        .route("/status", get(get_compliance_status))
        .route("/dashboard", get(get_compliance_dashboard))
        .route("/kyc/initiate", post(initiate_kyc_verification))
        .route("/kyc/status/:session_id", get(get_kyc_status))
        .route("/aml/screen", post(perform_aml_screening))
        .route("/aml/status", get(get_aml_status))
        .route("/requirements", get(get_regulatory_requirements))
        .route("/risk-assessment", get(get_risk_assessment))
        .route("/audit-trail", get(get_compliance_audit_trail))
}

/// Verify comprehensive enterprise compliance
async fn verify_compliance(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Json(request): Json<ComplianceVerificationRequest>,
) -> Result<Json<ComplianceVerificationResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("Compliance verification requested for wallet: {}", request.wallet_address);

    // Verify the requesting user has permission to check this wallet
    if user.wallet_address != request.wallet_address {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ComplianceErrorResponse {
                success: false,
                error: "Permission denied".to_string(),
                error_code: "WALLET_MISMATCH".to_string(),
                details: Some("Can only verify compliance for your own wallet".to_string()),
                recovery_suggestions: vec!["Use the wallet address from your authentication".to_string()],
                support_reference: Some("COMP-001".to_string()),
            })
        ));
    }

    // TODO: Get compliance service from state and perform verification
    // For now, return a placeholder response
    
    let mock_compliance_result = create_mock_compliance_result(&request.wallet_address);
    
    // Calculate tier eligibility
    let tier_eligibility = calculate_tier_eligibility(&mock_compliance_result);
    
    // Generate recommendations
    let recommended_actions = generate_compliance_recommendations(&mock_compliance_result);

    Ok(Json(ComplianceVerificationResponse {
        success: true,
        compliance_result: mock_compliance_result,
        verification_timestamp: Utc::now(),
        tier_eligibility,
        recommended_actions,
        next_review_date: Utc::now() + chrono::Duration::days(90),
    }))
}

/// Get current compliance status
async fn get_compliance_status(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Query(_query): Query<ComplianceStatusQuery>,
) -> Result<Json<ComplianceVerificationResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("Compliance status requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual compliance status retrieval
    let mock_compliance_result = create_mock_compliance_result(&user.wallet_address);
    let tier_eligibility = calculate_tier_eligibility(&mock_compliance_result);
    let recommended_actions = generate_compliance_recommendations(&mock_compliance_result);

    Ok(Json(ComplianceVerificationResponse {
        success: true,
        compliance_result: mock_compliance_result,
        verification_timestamp: Utc::now(),
        tier_eligibility,
        recommended_actions,
        next_review_date: Utc::now() + chrono::Duration::days(30),
    }))
}

/// Get comprehensive compliance dashboard
async fn get_compliance_dashboard(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
) -> Result<Json<ComplianceDashboardResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("Compliance dashboard requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual dashboard data retrieval
    let dashboard = ComplianceDashboardResponse {
        success: true,
        wallet_address: user.wallet_address.clone(),
        overall_compliance_status: ComplianceStatus::Compliant,
        compliance_score: 0.85,
        risk_level: "Low".to_string(),
        summary: ComplianceSummary {
            kyc_status: "Verified".to_string(),
            aml_status: "Clear".to_string(),
            sanctions_status: "Clear".to_string(),
            geographic_compliance: true,
            accredited_investor: true,
            data_privacy_compliant: true,
            regulatory_compliant: 4,
            total_regulations: 5,
        },
        recent_activities: vec![
            ComplianceActivity {
                activity_type: "KYC_VERIFICATION".to_string(),
                description: "KYC verification completed successfully".to_string(),
                timestamp: Utc::now() - chrono::Duration::days(30),
                status: "Completed".to_string(),
                impact_level: "High".to_string(),
            },
            ComplianceActivity {
                activity_type: "AML_SCREENING".to_string(),
                description: "Monthly AML screening completed".to_string(),
                timestamp: Utc::now() - chrono::Duration::days(5),
                status: "Clear".to_string(),
                impact_level: "Medium".to_string(),
            },
        ],
        required_actions: vec![
            ComplianceAction {
                action_type: "DOCUMENT_RENEWAL".to_string(),
                description: "Update proof of address documentation".to_string(),
                priority: "Medium".to_string(),
                due_date: Some(Utc::now() + chrono::Duration::days(60)),
                estimated_duration: "15 minutes".to_string(),
            },
        ],
        expiring_verifications: vec![
            ExpiringVerification {
                verification_type: "KYC".to_string(),
                provider: "Jumio".to_string(),
                expires_at: Utc::now() + chrono::Duration::days(335),
                days_until_expiry: 335,
                renewal_required: false,
            },
        ],
        dashboard_timestamp: Utc::now(),
    };

    Ok(Json(dashboard))
}

/// Initiate KYC verification process
async fn initiate_kyc_verification(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Json(_request): Json<KYCInitiationRequest>,
) -> Result<Json<KYCInitiationResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("KYC initiation requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual KYC provider integration
    Ok(Json(KYCInitiationResponse {
        success: true,
        kyc_session_id: "kyc_session_123".to_string(),
        verification_url: "https://verification.example.com/session/123".to_string(),
        required_documents: vec![
            "Government ID".to_string(),
            "Proof of Address".to_string(),
            "Selfie".to_string(),
        ],
        estimated_duration_minutes: 15,
        expires_at: Utc::now() + chrono::Duration::hours(24),
    }))
}

/// Get KYC verification status
async fn get_kyc_status(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Path(_session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("KYC status requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual KYC status retrieval
    let status = serde_json::json!({
        "success": true,
        "session_id": "kyc_session_123",
        "status": "Completed",
        "verification_result": "Verified",
        "completion_timestamp": Utc::now() - chrono::Duration::hours(2),
        "documents_verified": ["Government ID", "Proof of Address", "Selfie"],
        "risk_assessment": "Low"
    });

    Ok(Json(status))
}

/// Perform AML screening
async fn perform_aml_screening(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
    Json(_request): Json<AMLScreeningRequest>,
) -> Result<Json<AMLScreeningResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("AML screening requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual AML screening
    let mock_aml_result = create_mock_aml_screening();

    Ok(Json(AMLScreeningResponse {
        success: true,
        screening_result: mock_aml_result,
        screening_timestamp: Utc::now(),
        requires_further_investigation: false,
        escalation_required: false,
        next_screening_date: Utc::now() + chrono::Duration::days(30),
    }))
}

/// Get AML screening status
async fn get_aml_status(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
) -> Result<Json<AMLScreeningResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("AML status requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual AML status retrieval
    let mock_aml_result = create_mock_aml_screening();

    Ok(Json(AMLScreeningResponse {
        success: true,
        screening_result: mock_aml_result,
        screening_timestamp: Utc::now() - chrono::Duration::days(5),
        requires_further_investigation: false,
        escalation_required: false,
        next_screening_date: Utc::now() + chrono::Duration::days(25),
    }))
}

/// Get regulatory requirements for jurisdiction
async fn get_regulatory_requirements(
    State(_state): State<AppState>,
    _user: Web3AuthenticatedUser,
    Query(_query): Query<RegulatoryRequirementsQuery>,
) -> Result<Json<RegulatoryRequirementsResponse>, (StatusCode, Json<ComplianceErrorResponse>)> {
    // TODO: Implement actual regulatory requirements retrieval
    Ok(Json(RegulatoryRequirementsResponse {
        success: true,
        applicable_regulations: vec![
            RegulatoryRequirement {
                regulation_name: "GDPR".to_string(),
                regulation_type: "Data Protection".to_string(),
                jurisdiction: "European Union".to_string(),
                description: "General Data Protection Regulation for user data protection".to_string(),
                mandatory: true,
                penalties: "Up to 4% of annual revenue or €20 million".to_string(),
                compliance_steps: vec![
                    "Obtain explicit consent".to_string(),
                    "Implement data protection by design".to_string(),
                    "Maintain data processing records".to_string(),
                ],
                documentation_required: vec![
                    "Privacy Policy".to_string(),
                    "Data Processing Records".to_string(),
                    "Consent Records".to_string(),
                ],
            },
        ],
        compliance_roadmap: vec![
            ComplianceRoadmapItem {
                step_number: 1,
                step_name: "Privacy Policy Update".to_string(),
                description: "Update privacy policy to meet GDPR requirements".to_string(),
                estimated_duration_days: 7,
                dependencies: vec!["Legal Review".to_string()],
                cost_estimate: Some(2500.0),
                priority: "High".to_string(),
            },
        ],
        estimated_compliance_cost: Some(10000.0),
        estimated_timeline_days: Some(45),
    }))
}

/// Get risk assessment details
async fn get_risk_assessment(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
) -> Result<Json<RiskAssessment>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("Risk assessment requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual risk assessment retrieval
    let mock_risk_assessment = create_mock_risk_assessment();

    Ok(Json(mock_risk_assessment))
}

/// Get compliance audit trail
async fn get_compliance_audit_trail(
    State(_state): State<AppState>,
    user: Web3AuthenticatedUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ComplianceErrorResponse>)> {
    info!("Compliance audit trail requested for wallet: {}", user.wallet_address);

    // TODO: Implement actual audit trail retrieval
    let audit_trail = serde_json::json!({
        "success": true,
        "wallet_address": user.wallet_address,
        "audit_events": [
            {
                "event_type": "COMPLIANCE_VERIFICATION",
                "timestamp": Utc::now() - chrono::Duration::days(1),
                "status": "Compliant",
                "compliance_score": 0.85,
                "details": "Full compliance verification completed"
            },
            {
                "event_type": "KYC_VERIFICATION",
                "timestamp": Utc::now() - chrono::Duration::days(30),
                "status": "Verified",
                "provider": "Jumio",
                "details": "Identity verification completed successfully"
            },
            {
                "event_type": "AML_SCREENING",
                "timestamp": Utc::now() - chrono::Duration::days(5),
                "status": "Clear",
                "provider": "Chainalysis",
                "details": "No adverse findings in AML screening"
            }
        ],
        "total_events": 15,
        "date_range": {
            "start": Utc::now() - chrono::Duration::days(90),
            "end": Utc::now()
        }
    });

    Ok(Json(audit_trail))
}

// Helper functions for mock data generation

fn create_mock_compliance_result(wallet_address: &str) -> EnterpriseComplianceResult {
    use crate::auth::{
        KYCStatus, KYCLevel, DocumentVerification, BiometricVerification,
        AMLStatus, PEPCheck, SanctionsListCheck, AdverseMediaCheck, TransactionMonitoring,
        SanctionsStatus, RestrictionType, DataPrivacyCompliance, GDPRCompliance, 
        CCPACompliance, ConsentManagement, DataRetention, RiskLevel, MonitoringFrequency,
        RegulationType, RegulationStatus, RegulatoryCheck
    };

    EnterpriseComplianceResult {
        wallet_address: wallet_address.to_string(),
        compliance_status: ComplianceStatus::Compliant,
        regulatory_checks: vec![
            RegulatoryCheck {
                regulation_type: RegulationType::GDPR,
                status: RegulationStatus::Compliant,
                jurisdiction: "EU".to_string(),
                requirements_met: vec!["Consent".to_string()],
                requirements_pending: vec![],
                expiry_date: None,
                last_updated: Utc::now(),
            }
        ],
        kyc_verification: KYCVerification {
            status: KYCStatus::Verified,
            identity_verified: true,
            document_verification: DocumentVerification {
                government_id: true,
                proof_of_address: true,
                proof_of_income: Some(true),
                business_registration: None,
                documents_uploaded: 3,
                documents_verified: 3,
            },
            biometric_verification: Some(BiometricVerification {
                face_verification: true,
                liveness_check: true,
                voice_verification: None,
                fingerprint_verification: None,
            }),
            verification_level: KYCLevel::Enhanced,
            provider: "Jumio".to_string(),
            verification_date: Some(Utc::now() - chrono::Duration::days(30)),
            expiry_date: Some(Utc::now() + chrono::Duration::days(365)),
            risk_rating: "Low".to_string(),
        },
        aml_screening: create_mock_aml_screening(),
        sanctions_check: crate::auth::SanctionsCheck {
            status: SanctionsStatus::Clear,
            lists_checked: vec!["OFAC".to_string()],
            matches_found: vec![],
            check_date: Utc::now(),
            confidence_score: 0.95,
        },
        geographic_compliance: crate::auth::GeographicCompliance {
            jurisdiction: "US".to_string(),
            country_code: "US".to_string(),
            region: "North America".to_string(),
            is_restricted: false,
            restriction_type: None,
            allowed_services: vec!["trading".to_string()],
            restricted_services: vec![],
            compliance_requirements: vec!["SEC".to_string()],
        },
        accredited_investor_status: Some(crate::auth::AccreditedInvestorStatus {
            is_accredited: true,
            verification_method: "Income".to_string(),
            income_verification: Some(true),
            net_worth_verification: Some(true),
            professional_certification: Some(false),
            verification_date: Utc::now() - chrono::Duration::days(60),
            expiry_date: Utc::now() + chrono::Duration::days(305),
        }),
        data_privacy_compliance: DataPrivacyCompliance {
            gdpr_compliance: GDPRCompliance {
                lawful_basis: vec!["Consent".to_string()],
                consent_obtained: true,
                legitimate_interests: true,
                data_protection_impact_assessment: true,
                privacy_by_design: true,
            },
            ccpa_compliance: CCPACompliance {
                consumer_rights_notice: true,
                opt_out_mechanism: true,
                third_party_disclosures: true,
                data_deletion_process: true,
            },
            consent_management: ConsentManagement {
                explicit_consent: true,
                granular_consent: true,
                consent_withdrawal: true,
                consent_date: Some(Utc::now() - chrono::Duration::days(60)),
                consent_version: "v2.1".to_string(),
            },
            data_retention: DataRetention {
                retention_period_days: 2555,
                deletion_schedule: "Annual".to_string(),
                archive_requirements: true,
                legal_holds: vec![],
            },
            right_to_erasure: true,
            data_portability: true,
        },
        risk_assessment: create_mock_risk_assessment(),
        compliance_score: 0.85,
        next_review_date: Utc::now() + chrono::Duration::days(90),
        verification_timestamp: Utc::now(),
    }
}

fn create_mock_aml_screening() -> AMLScreening {
    use crate::auth::{AMLStatus, PEPCheck, SanctionsListCheck, AdverseMediaCheck, TransactionMonitoring};

    AMLScreening {
        status: AMLStatus::Clear,
        pep_check: PEPCheck {
            is_pep: false,
            pep_category: None,
            jurisdiction: None,
            relationship_to_pep: None,
        },
        sanctions_list_check: SanctionsListCheck {
            is_sanctioned: false,
            sanctions_lists: vec!["OFAC".to_string()],
            match_details: None,
        },
        adverse_media_check: AdverseMediaCheck {
            adverse_media_found: false,
            media_sources_count: 0,
            severity_score: 0.0,
            categories: vec![],
        },
        transaction_monitoring: TransactionMonitoring {
            suspicious_activity: false,
            transaction_patterns: vec![],
            velocity_alerts: 0,
            amount_alerts: 0,
            geographic_alerts: 0,
        },
        risk_score: 0.1,
        last_screening_date: Utc::now(),
        next_screening_date: Utc::now() + chrono::Duration::days(30),
    }
}

fn create_mock_risk_assessment() -> RiskAssessment {
    use crate::auth::{RiskLevel, MonitoringFrequency, crate::auth::RiskFactor};

    RiskAssessment {
        overall_risk_score: 0.15,
        risk_factors: vec![],
        mitigation_measures: vec!["Regular monitoring".to_string()],
        monitoring_frequency: MonitoringFrequency::Monthly,
        escalation_triggers: vec!["Risk increase".to_string()],
        last_assessment_date: Utc::now(),
        next_assessment_date: Utc::now() + chrono::Duration::days(90),
    }
}

fn calculate_tier_eligibility(compliance_result: &EnterpriseComplianceResult) -> TierEligibilityResult {
    let starter_eligible = matches!(compliance_result.compliance_status, ComplianceStatus::Compliant | ComplianceStatus::ConditionallyCompliant);
    let business_eligible = starter_eligible && matches!(compliance_result.kyc_verification.status, KYCStatus::Verified);
    let enterprise_eligible = business_eligible && matches!(compliance_result.aml_screening.status, AMLStatus::Clear) && compliance_result.compliance_score >= 0.8;
    let whale_eligible = enterprise_eligible && compliance_result.accredited_investor_status.as_ref().map_or(false, |ai| ai.is_accredited) && compliance_result.compliance_score >= 0.95;

    let current_maximum_tier = if whale_eligible {
        "Whale"
    } else if enterprise_eligible {
        "Enterprise"
    } else if business_eligible {
        "Business"
    } else if starter_eligible {
        "Starter"
    } else {
        "None"
    }.to_string();

    let mut blocking_factors = Vec::new();
    if !starter_eligible {
        blocking_factors.push("Basic compliance not met".to_string());
    }
    if !business_eligible && starter_eligible {
        blocking_factors.push("KYC verification required".to_string());
    }
    if !enterprise_eligible && business_eligible {
        blocking_factors.push("AML clearance and higher compliance score required".to_string());
    }
    if !whale_eligible && enterprise_eligible {
        blocking_factors.push("Accredited investor status and maximum compliance score required".to_string());
    }

    TierEligibilityResult {
        starter_eligible,
        business_eligible,
        enterprise_eligible,
        whale_eligible,
        current_maximum_tier,
        blocking_factors,
    }
}

fn generate_compliance_recommendations(compliance_result: &EnterpriseComplianceResult) -> Vec<String> {
    let mut recommendations = Vec::new();

    if compliance_result.compliance_score < 0.9 {
        recommendations.push("Consider enhancing compliance measures to achieve higher tier eligibility".to_string());
    }

    if matches!(compliance_result.kyc_verification.status, KYCStatus::NotStarted | KYCStatus::InProgress) {
        recommendations.push("Complete KYC verification to unlock business tier features".to_string());
    }

    if !matches!(compliance_result.aml_screening.status, AMLStatus::Clear) {
        recommendations.push("Resolve AML screening issues for enterprise tier access".to_string());
    }

    if compliance_result.accredited_investor_status.is_none() {
        recommendations.push("Consider accredited investor verification for whale tier access".to_string());
    }

    if recommendations.is_empty() {
        recommendations.push("Compliance status is excellent. Continue regular monitoring".to_string());
    }

    recommendations
}