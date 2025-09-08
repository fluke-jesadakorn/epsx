use std::collections::HashMap;// Admin JWT Claims and Service
// Enhanced security structure for administrative operations

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Algorithm, Validation};
use uuid::Uuid;
use tracing::{info, error};

use crate::auth::granular_permissions::GranularPermissionClaim;

/// Enhanced security context for admin operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSecurityContext {
    /// Multi-factor authentication verification status
    pub mfa_verified: bool,
    
    /// MFA verification timestamp
    pub mfa_timestamp: Option<u64>,
    
    /// Risk assessment score (0.0 = safe, 1.0 = high risk)
    pub risk_score: f32,
    
    /// Risk factors that contributed to the score
    pub risk_factors: Vec<String>,
    
    /// Device fingerprint for device binding
    pub device_binding: String,
    
    /// IP address restrictions (if any)
    pub ip_restrictions: Vec<String>,
    
    /// Current session IP address
    pub current_ip: String,
    
    /// Geographic location hash
    pub location_hash: String,
    
    /// Session start timestamp
    pub session_start: u64,
    
    /// Last activity timestamp
    pub last_activity: u64,
}

/// Privileged operation context for admin actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegedOperationContext {
    /// List of privileged operations performed in this session
    pub operations: Vec<PrivilegedOperation>,
    
    /// Maximum allowed privileged operations per session
    pub max_operations: u32,
    
    /// Privileged access expiry (shorter than token expiry)
    pub privileged_expires_at: u64,
    
    /// Requires re-authentication for next privileged operation
    pub requires_reauth: bool,
}

/// Individual privileged operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegedOperation {
    /// Operation type (e.g., "user_delete", "permission_grant")
    pub operation: String,
    
    /// Timestamp of operation
    pub timestamp: u64,
    
    /// Target of operation (user ID, permission, etc.)
    pub target: String,
    
    /// IP address from which operation was performed
    pub ip_address: String,
    
    /// Additional operation context
    pub context: HashMap<String, String>,
}

/// Audit trail entry for admin actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditEntry {
    /// Action performed
    pub action: String,
    
    /// Timestamp of action
    pub timestamp: u64,
    
    /// Resource affected
    pub resource: String,
    
    /// Previous state (for rollback purposes)
    pub previous_state: Option<String>,
    
    /// New state
    pub new_state: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Admin permission matrix with enhanced structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissionMatrix {
    /// Platform-specific permissions with metadata
    pub platforms: HashMap<String, AdminPlatformAccess>,
    
    /// System-level access permissions
    pub system_access: SystemAccessLevel,
    
    /// Delegation rights (ability to grant permissions to others)
    pub delegation_rights: Vec<DelegationRule>,
    
    /// Emergency access permissions (time-limited)
    pub emergency_access: Option<EmergencyAccessContext>,
    
    /// Permission version for cache invalidation
    pub version: u32,
    
    /// Permission hash for instant revocation
    pub hash: String,
}

/// Platform-specific admin access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPlatformAccess {
    /// Platform name (admin, epsx, epsx-pay, etc.)
    pub platform: String,
    
    /// Granular permissions with metadata
    pub permissions: HashMap<String, AdminPermissionClaim>,
    
    /// Platform-specific access level
    pub access_level: String,
    
    /// Platform access restrictions
    pub restrictions: Vec<String>,
}

/// Enhanced admin permission claim with security metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissionClaim {
    /// Base permission claim
    pub base: GranularPermissionClaim,
    
    /// Admin-specific metadata
    pub admin_metadata: AdminPermissionMetadata,
    
    /// Security constraints
    pub constraints: Vec<SecurityConstraint>,
    
    /// Approval workflow information
    pub approval_info: Option<ApprovalInfo>,
}

/// Admin-specific permission metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissionMetadata {
    /// Sensitivity level of this permission
    pub sensitivity_level: String,
    
    /// Requires additional approval for use
    pub requires_approval: bool,
    
    /// Maximum usage count
    pub max_usage: Option<u32>,
    
    /// Current usage count
    pub current_usage: u32,
    
    /// Time-based restrictions
    pub time_restrictions: Vec<TimeRestriction>,
    
    /// Monitoring flags
    pub monitoring_enabled: bool,
}

/// Security constraint for admin permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConstraint {
    /// Type of constraint
    pub constraint_type: String,
    
    /// Constraint value
    pub value: String,
    
    /// Whether constraint is enforced
    pub enforced: bool,
}

/// Time-based restriction for admin permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    /// Day of week restrictions (0-6, where 0 is Sunday)
    pub allowed_days: Vec<u8>,
    
    /// Hour restrictions (0-23)
    pub allowed_hours: Vec<u8>,
    
    /// Timezone for restrictions
    pub timezone: String,
}

/// System access level for admin operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAccessLevel {
    /// Level name (super_admin, admin, operator)
    pub level: String,
    
    /// System capabilities
    pub capabilities: Vec<String>,
    
    /// Access restrictions
    pub restrictions: Vec<String>,
    
    /// Monitoring level
    pub monitoring_level: String,
}

/// Delegation rule for granting permissions to others
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRule {
    /// What permissions can be delegated
    pub delegatable_permissions: Vec<String>,
    
    /// To whom permissions can be delegated
    pub target_roles: Vec<String>,
    
    /// Maximum delegation duration
    pub max_duration: u64,
    
    /// Requires approval for delegation
    pub requires_approval: bool,
}

/// Emergency access context for crisis situations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAccessContext {
    /// Emergency access level
    pub level: String,
    
    /// Granted emergency permissions
    pub permissions: Vec<String>,
    
    /// Emergency access expiry (very short)
    pub expires_at: u64,
    
    /// Reason for emergency access
    pub reason: String,
    
    /// Who granted emergency access
    pub granted_by: String,
    
    /// Requires post-emergency review
    pub requires_review: bool,
}

/// Approval information for sensitive permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalInfo {
    /// Who approved this permission
    pub approved_by: String,
    
    /// Approval timestamp
    pub approved_at: u64,
    
    /// Approval reason
    pub reason: String,
    
    /// Approval expires at
    pub expires_at: Option<u64>,
    
    /// Reference to approval workflow
    pub workflow_id: Option<String>,
}

/// Impersonation context for admin user impersonation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpersonationContext {
    /// Original admin user ID
    pub original_admin_id: String,
    
    /// User being impersonated
    pub target_user_id: String,
    
    /// Impersonation start time
    pub started_at: u64,
    
    /// Maximum impersonation duration
    pub max_duration: u64,
    
    /// Reason for impersonation
    pub reason: String,
    
    /// Impersonation permissions (subset of admin permissions)
    pub allowed_actions: Vec<String>,
    
    /// Audit trail for impersonated actions
    pub audit_trail: Vec<AdminAuditEntry>,
}

/// Enhanced Admin JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminJWTClaims {
    // Standard JWT claims
    pub iss: String,        // Issuer
    pub sub: String,        // Subject (admin user ID)
    pub aud: String,        // Audience (admin)
    pub exp: u64,          // Expiration time (shorter for admin)
    pub iat: u64,          // Issued at
    pub nbf: u64,          // Not before
    pub jti: String,       // JWT ID for revocation
    
    // Admin user information
    pub email: String,
    pub name: String,
    // Role removed - using permissions-based system only
    
    // Enhanced admin security context
    pub security_context: AdminSecurityContext,
    
    // Privileged operation tracking
    pub privileged_ops: PrivilegedOperationContext,
    
    // Admin permission matrix
    pub permissions: AdminPermissionMatrix,
    
    // Audit trail for this session
    pub audit_trail: Vec<AdminAuditEntry>,
    
    // Impersonation context (if applicable)
    pub impersonation: Option<ImpersonationContext>,
    
    // Token type identifier
    pub token_type: String, // "admin_access"
    
    // Session metadata
    pub session_id: String,
    pub client_id: Option<String>,
}

/// Admin JWT validation result
#[derive(Debug, Clone)]
pub struct AdminValidationResult {
    /// Whether token is valid
    pub valid: bool,
    
    /// Validated claims
    pub claims: Option<AdminJWTClaims>,
    
    /// Security warnings
    pub warnings: Vec<String>,
    
    /// Risk assessment results
    pub risk_assessment: RiskAssessment,
    
    /// Whether MFA re-verification is required
    pub requires_mfa: bool,
    
    /// Whether privileged access has expired
    pub privileged_expired: bool,
}

/// Risk assessment result for admin session
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// Overall risk score
    pub risk_score: f32,
    
    /// Risk factors identified
    pub risk_factors: Vec<RiskFactor>,
    
    /// Recommended actions
    pub recommendations: Vec<String>,
    
    /// Whether session should be terminated
    pub terminate_session: bool,
}

/// Individual risk factor
#[derive(Debug, Clone)]
pub struct RiskFactor {
    /// Risk factor type
    pub factor_type: String,
    
    /// Risk factor description
    pub description: String,
    
    /// Risk score contribution
    pub score_impact: f32,
    
    /// Severity level
    pub severity: String,
}

/// Admin JWT Service for enhanced security operations
pub struct AdminJWTService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
}

impl AdminJWTService {
    /// Create new AdminJWTService with enhanced security configuration
    pub fn new(secret: &[u8], issuer: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            issuer,
        }
    }
    
    /// Generate admin JWT token with enhanced security context
    pub fn generate_admin_token(
        &self,
        admin_id: String,
        email: String,
        name: String,
        security_context: AdminSecurityContext,
        permissions: AdminPermissionMatrix,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // Admin tokens have shorter expiry (30 minutes)
        let exp = now + (30 * 60);
        
        let claims = AdminJWTClaims {
            iss: self.issuer.clone(),
            sub: admin_id,
            aud: "epsx-api".to_string(),
            exp,
            iat: now,
            nbf: now,
            jti: Uuid::new_v4().to_string(),
            email,
            name,
            security_context,
            privileged_ops: PrivilegedOperationContext {
                operations: Vec::new(),
                max_operations: 50, // Limit privileged operations
                privileged_expires_at: now + (15 * 60), // 15 minutes for privileged access
                requires_reauth: false,
            },
            permissions,
            audit_trail: Vec::new(),
            impersonation: None,
            token_type: "admin_access".to_string(),
            session_id: Uuid::new_v4().to_string(),
            client_id: None,
        };
        
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.encoding_key)?;
        
        info!("Generated admin JWT token for user: {}", claims.email);
        Ok(token)
    }
    
    /// Validate admin JWT token with enhanced security checks
    pub fn validate_admin_token(&self, token: &str, current_ip: &str) -> AdminValidationResult {
        let mut warnings = Vec::new();
        let mut risk_factors = Vec::new();
        
        // Decode token
        let validation = Validation::new(Algorithm::HS256);
        let token_data = match decode::<AdminJWTClaims>(token, &self.decoding_key, &validation) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to decode admin token: {}", e);
                return AdminValidationResult {
                    valid: false,
                    claims: None,
                    warnings: vec!["Invalid token format".to_string()],
                    risk_assessment: RiskAssessment {
                        risk_score: 1.0,
                        risk_factors: vec![],
                        recommendations: vec!["Re-authenticate".to_string()],
                        terminate_session: true,
                    },
                    requires_mfa: true,
                    privileged_expired: true,
                };
            }
        };
        
        let claims = token_data.claims;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Enhanced security validations
        let mut risk_score = 0.0;
        
        // Check IP address binding
        if !claims.security_context.ip_restrictions.is_empty() {
            if !claims.security_context.ip_restrictions.contains(&current_ip.to_string()) {
                risk_factors.push(RiskFactor {
                    factor_type: "ip_violation".to_string(),
                    description: "Request from unauthorized IP address".to_string(),
                    score_impact: 0.8,
                    severity: "high".to_string(),
                });
                risk_score += 0.8;
            }
        }
        
        // Check IP change
        if claims.security_context.current_ip != current_ip {
            warnings.push("IP address changed during session".to_string());
            risk_factors.push(RiskFactor {
                factor_type: "ip_change".to_string(),
                description: "IP address changed during session".to_string(),
                score_impact: 0.3,
                severity: "medium".to_string(),
            });
            risk_score += 0.3;
        }
        
        // Check MFA expiry
        let requires_mfa = if let Some(mfa_timestamp) = claims.security_context.mfa_timestamp {
            now - mfa_timestamp > (60 * 60) // MFA valid for 1 hour
        } else {
            true // No MFA timestamp means MFA required
        };
        
        if requires_mfa {
            warnings.push("MFA re-verification required".to_string());
            risk_factors.push(RiskFactor {
                factor_type: "mfa_expired".to_string(),
                description: "MFA verification expired".to_string(),
                score_impact: 0.4,
                severity: "medium".to_string(),
            });
            risk_score += 0.4;
        }
        
        // Check privileged access expiry
        let privileged_expired = now > claims.privileged_ops.privileged_expires_at;
        if privileged_expired {
            warnings.push("Privileged access expired".to_string());
        }
        
        // Check session duration
        let session_duration = now - claims.security_context.session_start;
        if session_duration > (4 * 60 * 60) { // 4 hours max session
            risk_factors.push(RiskFactor {
                factor_type: "long_session".to_string(),
                description: "Session duration exceeds recommended limits".to_string(),
                score_impact: 0.2,
                severity: "low".to_string(),
            });
            risk_score += 0.2;
        }
        
        // Check privileged operations count
        if claims.privileged_ops.operations.len() as u32 > claims.privileged_ops.max_operations {
            risk_factors.push(RiskFactor {
                factor_type: "excessive_operations".to_string(),
                description: "Too many privileged operations in session".to_string(),
                score_impact: 0.5,
                severity: "high".to_string(),
            });
            risk_score += 0.5;
        }
        
        // Combine with existing risk score
        risk_score = (risk_score + claims.security_context.risk_score) / 2.0;
        
        let recommendations = if risk_score > 0.7 {
            vec!["Terminate session and re-authenticate".to_string()]
        } else if risk_score > 0.5 {
            vec!["Require MFA re-verification".to_string()]
        } else if risk_score > 0.3 {
            vec!["Monitor session closely".to_string()]
        } else {
            vec![]
        };
        
        AdminValidationResult {
            valid: risk_score < 0.8, // Terminate if risk too high
            claims: Some(claims),
            warnings,
            risk_assessment: RiskAssessment {
                risk_score,
                risk_factors,
                recommendations,
                terminate_session: risk_score > 0.7,
            },
            requires_mfa,
            privileged_expired,
        }
    }
}