/**
 * UNIFIED COMPLIANCE API CLIENT
 *
 * KYC, risk assessment, and compliance monitoring endpoints.
 * Consolidates compliance-related API calls for admin applications.
 *
 * Features:
 * - KYC status management
 * - Risk assessments
 * - Audit trails
 * - Suspicious activity detection
 * - Compliance metrics
 * - Regulatory settings
 */

import type { ApiResponse, PaginatedResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface KYCStatus {
  wallet_address: string;
  status: 'pending' | 'approved' | 'rejected' | 'under_review';
  submission_date: string;
  review_date?: string;
  reviewed_by?: string;
  documents_submitted: number;
  verification_level: 'none' | 'basic' | 'intermediate' | 'advanced';
  notes?: string;
  rejection_reason?: string;
}

export interface RiskAssessment {
  id: string;
  wallet_address: string;
  risk_level: 'low' | 'medium' | 'high' | 'critical';
  risk_score: number;
  assessment_date: string;
  assessed_by?: string;
  factors: Array<{
    factor: string;
    weight: number;
    score: number;
    description: string;
  }>;
  recommended_actions: string[];
  status: 'active' | 'resolved' | 'monitoring';
  resolution_date?: string;
  notes?: string;
}

export interface AuditLogEntry {
  id: string;
  timestamp: string;
  wallet_address?: string;
  admin_address?: string;
  action: string;
  resource_type: string;
  resource_id: string;
  changes?: Record<string, unknown>;
  ip_address?: string;
  user_agent?: string;
  result: 'success' | 'failure';
  error_message?: string;
}

export interface SuspiciousActivity {
  id: string;
  wallet_address: string;
  activity_type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  detected_at: string;
  description: string;
  evidence: Array<{
    type: string;
    data: unknown;
    timestamp: string;
  }>;
  status: 'new' | 'investigating' | 'resolved' | 'false_positive';
  assigned_to?: string;
  resolution_notes?: string;
  resolved_at?: string;
}

export interface ComplianceMetrics {
  total_kyc_submissions: number;
  kyc_pending: number;
  kyc_approved: number;
  kyc_rejected: number;
  approval_rate: number;
  average_processing_time_hours: number;
  total_risk_assessments: number;
  high_risk_wallets: number;
  critical_risk_wallets: number;
  suspicious_activities: number;
  unresolved_activities: number;
  audit_log_entries: number;
  compliance_score: number;
}

export interface RegulatorySettings {
  kyc_required: boolean;
  kyc_required_tier: string;
  auto_flag_threshold: number;
  auto_block_threshold: number;
  risk_assessment_frequency_days: number;
  mandatory_review_criteria: string[];
  restricted_jurisdictions: string[];
  enhanced_monitoring_rules: Array<{
    rule_name: string;
    criteria: Record<string, unknown>;
    action: 'flag' | 'block' | 'monitor';
  }>;
}

// ============================================================================
// COMPLIANCE API CLASS
// ============================================================================

export class ComplianceApi {
  private client: UnifiedApiClient;

  constructor(client: UnifiedApiClient) {
    this.client = client;
  }

  // ============================================================================
  // KYC MANAGEMENT
  // ============================================================================

  /**
   * Get KYC statuses
   * GET /api/admin/compliance/statuses
   */
  async getKYCStatuses(filters?: { status?: string; limit?: number }): Promise<ApiResponse<KYCStatus[]>> {
    return this.client.get<KYCStatus[]>('/api/admin/compliance/statuses', filters);
  }

  /**
   * Get KYC status for wallet
   * GET /api/admin/compliance/kyc/{wallet_address}
   */
  async getKYCStatus(wallet_address: string): Promise<ApiResponse<KYCStatus>> {
    return this.client.get<KYCStatus>(`/api/admin/compliance/kyc/${wallet_address}`);
  }

  /**
   * Approve KYC
   * POST /api/admin/compliance/kyc/approve/{wallet_address}
   */
  async approveKYC(wallet_address: string, verification_level?: string, notes?: string): Promise<ApiResponse<{ approved: boolean }>> {
    return this.client.post<{ approved: boolean }>(`/api/admin/compliance/kyc/approve/${wallet_address}`, {
      verification_level,
      notes
    });
  }

  /**
   * Reject KYC
   * POST /api/admin/compliance/kyc/reject/{wallet_address}
   */
  async rejectKYC(wallet_address: string, reason: string): Promise<ApiResponse<{ rejected: boolean }>> {
    return this.client.post<{ rejected: boolean }>(`/api/admin/compliance/kyc/reject/${wallet_address}`, { reason });
  }

  // ============================================================================
  // RISK ASSESSMENT
  // ============================================================================

  /**
   * Get risk assessments
   * GET /api/admin/compliance/risk-assessments
   */
  async getRiskAssessments(filters?: { risk_level?: string; status?: string; limit?: number }): Promise<ApiResponse<RiskAssessment[]>> {
    return this.client.get<RiskAssessment[]>('/api/admin/compliance/risk-assessments', filters);
  }

  /**
   * Get risk assessment for wallet
   * GET /api/admin/compliance/risk-assessments/{wallet_address}
   */
  async getWalletRiskAssessment(wallet_address: string): Promise<ApiResponse<RiskAssessment>> {
    return this.client.get<RiskAssessment>(`/api/admin/compliance/risk-assessments/${wallet_address}`);
  }

  /**
   * Create risk assessment
   * POST /api/admin/compliance/risk-assessments
   */
  async createRiskAssessment(wallet_address: string, data?: Partial<RiskAssessment>): Promise<ApiResponse<RiskAssessment>> {
    return this.client.post<RiskAssessment>('/api/admin/compliance/risk-assessments', {
      wallet_address,
      ...data
    });
  }

  /**
   * Update risk assessment
   * PUT /api/admin/compliance/risk-assessments/{assessment_id}
   */
  async updateRiskAssessment(assessment_id: string, data: Partial<RiskAssessment>): Promise<ApiResponse<RiskAssessment>> {
    return this.client.put<RiskAssessment>(`/api/admin/compliance/risk-assessments/${assessment_id}`, data);
  }

  // ============================================================================
  // AUDIT TRAIL
  // ============================================================================

  /**
   * Get audit trail
   * GET /api/admin/compliance/audit-trail
   */
  async getAuditTrail(filters?: {
    wallet_address?: string;
    action?: string;
    resource_type?: string;
    from_date?: string;
    to_date?: string;
    limit?: number;
    offset?: number;
  }): Promise<ApiResponse<PaginatedResponse<AuditLogEntry>>> {
    return this.client.get<PaginatedResponse<AuditLogEntry>>('/api/admin/compliance/audit-trail', filters);
  }

  /**
   * Export audit trail
   * POST /api/admin/compliance/export-report
   */
  async exportAuditReport(filters?: {
    from_date?: string;
    to_date?: string;
    format?: 'csv' | 'pdf' | 'json';
  }): Promise<ApiResponse<{ download_url: string }>> {
    return this.client.post<{ download_url: string }>('/api/admin/compliance/export-report', filters);
  }

  // ============================================================================
  // SUSPICIOUS ACTIVITY
  // ============================================================================

  /**
   * Get suspicious activities
   * GET /api/admin/compliance/suspicious-activities
   */
  async getSuspiciousActivities(filters?: {
    status?: string;
    severity?: string;
    limit?: number;
  }): Promise<ApiResponse<SuspiciousActivity[]>> {
    return this.client.get<SuspiciousActivity[]>('/api/admin/compliance/suspicious-activities', filters);
  }

  /**
   * Flag user for suspicious activity
   * POST /api/admin/compliance/flag-user/{wallet_address}
   */
  async flagUser(wallet_address: string, reason: string, severity?: string): Promise<ApiResponse<{ flagged: boolean }>> {
    return this.client.post<{ flagged: boolean }>(`/api/admin/compliance/flag-user/${wallet_address}`, {
      reason,
      severity
    });
  }

  /**
   * Investigate activity
   * POST /api/admin/compliance/investigate/{activity_id}
   */
  async investigateActivity(activity_id: string, notes?: string): Promise<ApiResponse<{ investigating: boolean }>> {
    return this.client.post<{ investigating: boolean }>(`/api/admin/compliance/investigate/${activity_id}`, { notes });
  }

  /**
   * Resolve activity
   * POST /api/admin/compliance/resolve/{activity_id}
   */
  async resolveActivity(activity_id: string, resolution: string, is_false_positive?: boolean): Promise<ApiResponse<{ resolved: boolean }>> {
    return this.client.post<{ resolved: boolean }>(`/api/admin/compliance/resolve/${activity_id}`, {
      resolution,
      is_false_positive
    });
  }

  // ============================================================================
  // USER ACTIONS
  // ============================================================================

  /**
   * Block user
   * POST /api/admin/compliance/block-user/{wallet_address}
   */
  async blockUser(wallet_address: string, reason: string, duration?: number): Promise<ApiResponse<{ blocked: boolean }>> {
    return this.client.post<{ blocked: boolean }>(`/api/admin/compliance/block-user/${wallet_address}`, {
      reason,
      duration
    });
  }

  /**
   * Unblock user
   * POST /api/admin/compliance/unblock-user/{wallet_address}
   */
  async unblockUser(wallet_address: string): Promise<ApiResponse<{ unblocked: boolean }>> {
    return this.client.post<{ unblocked: boolean }>(`/api/admin/compliance/unblock-user/${wallet_address}`);
  }

  // ============================================================================
  // METRICS & SETTINGS
  // ============================================================================

  /**
   * Get compliance metrics
   * GET /api/admin/compliance/metrics
   */
  async getMetrics(): Promise<ApiResponse<ComplianceMetrics>> {
    return this.client.get<ComplianceMetrics>('/api/admin/compliance/metrics');
  }

  /**
   * Get regulatory settings
   * GET /api/admin/compliance/regulatory-settings
   */
  async getRegulatorySettings(): Promise<ApiResponse<RegulatorySettings>> {
    return this.client.get<RegulatorySettings>('/api/admin/compliance/regulatory-settings');
  }

  /**
   * Update regulatory settings
   * PUT /api/admin/compliance/regulatory-settings
   */
  async updateRegulatorySettings(settings: Partial<RegulatorySettings>): Promise<ApiResponse<RegulatorySettings>> {
    return this.client.put<RegulatorySettings>('/api/admin/compliance/regulatory-settings', settings);
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create Compliance API client
 */
export function createComplianceClient(client: UnifiedApiClient): ComplianceApi {
  return new ComplianceApi(client);
}

// ============================================================================
// EXPORTS
// ============================================================================

export default ComplianceApi;
