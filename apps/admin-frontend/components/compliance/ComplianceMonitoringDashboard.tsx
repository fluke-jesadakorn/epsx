/**
 * Compliance Monitoring Dashboard
 * Admin interface for enterprise compliance, KYC/AML, and risk management
 * Provides real-time compliance monitoring and regulatory oversight
 */

'use client';

import {
  Activity,
  AlertTriangle,
  Ban,
  CheckCircle,
  Clock,
  Download,
  Eye,
  FileText,
  Flag,
  RefreshCw,
  Search,
  Shield,
  Users,
  XCircle
} from 'lucide-react';
import { useEffect, useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

// Compliance Types
interface ComplianceStatus {
  wallet_address: string;
  enterprise_tier: string;
  kyc_status: 'pending' | 'verified' | 'rejected' | 'expired';
  aml_status: 'clear' | 'flagged' | 'under_review' | 'blocked';
  risk_score: number; // 0-100
  last_kyc_date: string;
  kyc_provider: string;
  compliance_flags: string[];
  verification_documents: string[];
  sanctions_check: boolean;
  pep_check: boolean; // Politically Exposed Person
  watchlist_status: string;
}

interface RiskAssessment {
  id: string;
  wallet_address: string;
  assessment_type: 'transaction' | 'behavior' | 'network' | 'compliance';
  risk_level: 'low' | 'medium' | 'high' | 'critical';
  risk_factors: string[];
  score: number;
  automated: boolean;
  created_at: string;
  reviewed_by?: string;
  review_notes?: string;
}

interface AuditTrail {
  id: string;
  wallet_address: string;
  action: string;
  actor: string; // Admin who performed action
  details: Record<string, any>;
  compliance_impact: string;
  created_at: string;
  regulatory_category: string;
}

interface SuspiciousActivity {
  id: string;
  wallet_address: string;
  activity_type: 'rapid_tier_upgrade' | 'unusual_volume' | 'multiple_accounts' | 'sanctions_hit' | 'suspicious_timing';
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  detected_at: string;
  investigation_status: 'open' | 'investigating' | 'resolved' | 'false_positive';
  assigned_to?: string;
  resolution_notes?: string;
}

interface ComplianceMetrics {
  total_verified_users: number;
  pending_kyc: number;
  high_risk_users: number;
  active_investigations: number;
  compliance_rate: number;
  average_verification_time_hours: number;
  flagged_transactions_24h: number;
  sanctions_hits_30d: number;
}

interface RegulatorySetting {
  id: string;
  regulation: string;
  jurisdiction: string;
  requirement: string;
  threshold_value?: number;
  mandatory: boolean;
  implementation_status: 'active' | 'pending' | 'disabled';
  last_updated: string;
}

/**
 *
 */
export default function ComplianceMonitoringDashboard() {
  // State Management
  const [complianceStatuses, setComplianceStatuses] = useState<ComplianceStatus[]>([]);
  const [riskAssessments, setRiskAssessments] = useState<RiskAssessment[]>([]);
  const [auditTrail, setAuditTrail] = useState<AuditTrail[]>([]);
  const [suspiciousActivities, setSuspiciousActivities] = useState<SuspiciousActivity[]>([]);
  const [complianceMetrics, setComplianceMetrics] = useState<ComplianceMetrics | null>(null);
  const [regulatorySettings, setRegulatorySettings] = useState<RegulatorySetting[]>([]);

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedStatus, setSelectedStatus] = useState<ComplianceStatus | null>(null);
  const [selectedRisk, setSelectedRisk] = useState<RiskAssessment | null>(null);
  const [activeTab, setActiveTab] = useState('overview');
  const [loading, setLoading] = useState(false);

  // Load compliance data
  useEffect(() => {
    loadComplianceData();
  }, []);

  const loadComplianceData = async () => {
    setLoading(true);
    try {
      // Load all compliance data in parallel
      await Promise.all([
        loadComplianceStatuses(),
        loadRiskAssessments(),
        loadAuditTrail(),
        loadSuspiciousActivities(),
        loadComplianceMetrics(),
        loadRegulatorySettings(),
      ]);
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load compliance data:', _error);
    } finally {
      setLoading(false);
    }
  };

  const loadComplianceStatuses = async () => {
    try {
      const response = await fetch('/api/admin/compliance/statuses', {
        credentials: 'include',
      });
      if (response.ok) {
        const data = await response.json();
        setComplianceStatuses(data.statuses || []);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load compliance statuses:', _error);
    }
  };

  const loadRiskAssessments = async () => {
    try {
      const response = await fetch('/api/admin/compliance/risk-assessments', {
        credentials: 'include',
      });
      if (response.ok) {
        const data = await response.json();
        setRiskAssessments(data.assessments || []);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load risk assessments:', _error);
    }
  };

  const loadAuditTrail = async () => {
    try {
      const response = await fetch('/api/admin/compliance/audit-trail', {
        credentials: 'include',
      });
      if (response.ok) {
        const data = await response.json();
        setAuditTrail(data.trail || []);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load audit trail:', _error);
    }
  };

  const loadSuspiciousActivities = async () => {
    try {
      const response = await fetch('/api/admin/compliance/suspicious-activities', {
        credentials: 'include',
      });
      if (response.ok) {
        const data = await response.json();
        setSuspiciousActivities(data.activities || []);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load suspicious activities:', _error);
    }
  };

  const loadComplianceMetrics = async () => {
    try {
      const response = await fetch('/api/admin/compliance/metrics', {
        credentials: 'include',
      });
      if (response.ok) {
        const data = await response.json();
        setComplianceMetrics(data.metrics);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load compliance metrics:', _error);
    }
  };

  const loadRegulatorySettings = async () => {
    try {
      const response = await fetch('/api/admin/compliance/regulatory-settings', {
        credentials: 'include',
      });
      if (response.ok) {
        const data = await response.json();
        setRegulatorySettings(data.settings || []);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to load regulatory settings:', _error);
    }
  };

  // Action Handlers
  const handleApproveKyc = async (walletAddress: string) => {
    try {
      const response = await fetch(`/api/admin/compliance/kyc/approve/${walletAddress}`, {
        method: 'POST',
        credentials: 'include',
      });

      if (response.ok) {
        await loadComplianceStatuses();
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to approve KYC:', _error);
    }
  };

  const handleRejectKyc = async (walletAddress: string, reason: string) => {
    try {
      const response = await fetch(`/api/admin/compliance/kyc/reject/${walletAddress}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason }),
        credentials: 'include',
      });

      if (response.ok) {
        await loadComplianceStatuses();
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to reject KYC:', _error);
    }
  };

  const handleFlagUser = async (walletAddress: string, flagType: string, notes: string) => {
    try {
      const response = await fetch(`/api/admin/compliance/flag-user/${walletAddress}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ flag_type: flagType, notes }),
        credentials: 'include',
      });

      if (response.ok) {
        await loadComplianceData();
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to flag user:', _error);
    }
  };

  const handleBlockUser = async (walletAddress: string, reason: string) => {
    try {
      const response = await fetch(`/api/admin/compliance/block-user/${walletAddress}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason }),
        credentials: 'include',
      });

      if (response.ok) {
        await loadComplianceData();
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to block user:', _error);
    }
  };

  const handleUpdateRiskAssessment = async (assessmentId: string, updates: Partial<RiskAssessment>) => {
    try {
      const response = await fetch(`/api/admin/compliance/risk-assessments/${assessmentId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates),
        credentials: 'include',
      });

      if (response.ok) {
        await loadRiskAssessments();
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to update risk assessment:', _error);
    }
  };

  const handleInvestigateActivity = async (activityId: string, assignedTo: string) => {
    try {
      const response = await fetch(`/api/admin/compliance/investigate/${activityId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ assigned_to: assignedTo }),
        credentials: 'include',
      });

      if (response.ok) {
        await loadSuspiciousActivities();
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to assign investigation:', _error);
    }
  };

  const handleExportComplianceReport = async (reportType: string, startDate: string, endDate: string) => {
    try {
      const response = await fetch('/api/admin/compliance/export-report', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          report_type: reportType,
          start_date: startDate,
          end_date: endDate,
        }),
        credentials: 'include',
      });

      if (response.ok) {
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `compliance-report-${reportType}-${new Date().toISOString().split('T')[0]}.pdf`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to export compliance report:', _error);
    }
  };

  // Utility Functions
  const getKycStatusBadge = (status: string) => {
    const variants = {
      verified: 'bg-green-100 text-green-800',
      pending: 'bg-yellow-100 text-yellow-800',
      rejected: 'bg-red-100 text-red-800',
      expired: 'bg-gray-100 text-gray-800',
    };
    return variants[status as keyof typeof variants] || variants.pending;
  };

  const getAmlStatusBadge = (status: string) => {
    const variants = {
      clear: 'bg-green-100 text-green-800',
      flagged: 'bg-red-100 text-red-800',
      under_review: 'bg-yellow-100 text-yellow-800',
      blocked: 'bg-red-100 text-red-800',
    };
    return variants[status as keyof typeof variants] || variants.under_review;
  };

  const getRiskLevelColor = (level: string) => {
    const colors = {
      low: 'text-green-600',
      medium: 'text-yellow-600',
      high: 'text-orange-600',
      critical: 'text-red-600',
    };
    return colors[level as keyof typeof colors] || colors.medium;
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical': return <XCircle className="w-4 h-4 text-red-600" />;
      case 'high': return <AlertTriangle className="w-4 h-4 text-orange-600" />;
      case 'medium': return <Flag className="w-4 h-4 text-yellow-600" />;
      case 'low': return <Clock className="w-4 h-4 text-blue-600" />;
      default: return <Clock className="w-4 h-4 text-gray-600" />;
    }
  };

  const filteredStatuses = complianceStatuses.filter(status =>
    status.wallet_address.toLowerCase().includes(searchQuery.toLowerCase()) ||
    status.enterprise_tier.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Compliance Monitoring</h1>
          <p className="text-sm text-gray-500">Enterprise KYC/AML compliance and risk management</p>
        </div>
        <div className="flex items-center space-x-3">
          <Button
            onClick={loadComplianceData}
            disabled={loading}
            variant="outline"
            size="sm"
          >
            <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button
            onClick={() => handleExportComplianceReport('full',
              (new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0] || ''),
              (new Date().toISOString().split('T')[0] || '')
            )}
            size="sm"
          >
            <Download className="w-4 h-4 mr-2" />
            Export Report
          </Button>
        </div>
      </div>

      {/* Compliance Metrics */}
      {complianceMetrics && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center">
                <Shield className="w-5 h-5 text-green-600" />
                <div className="ml-3">
                  <p className="text-sm font-medium text-gray-500">Verified Users</p>
                  <p className="text-2xl font-bold text-gray-900">{complianceMetrics.total_verified_users}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center">
                <Clock className="w-5 h-5 text-yellow-600" />
                <div className="ml-3">
                  <p className="text-sm font-medium text-gray-500">Pending KYC</p>
                  <p className="text-2xl font-bold text-gray-900">{complianceMetrics.pending_kyc}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center">
                <AlertTriangle className="w-5 h-5 text-red-600" />
                <div className="ml-3">
                  <p className="text-sm font-medium text-gray-500">High Risk</p>
                  <p className="text-2xl font-bold text-gray-900">{complianceMetrics.high_risk_users}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center">
                <Activity className="w-5 h-5 text-blue-600" />
                <div className="ml-3">
                  <p className="text-sm font-medium text-gray-500">Active Investigations</p>
                  <p className="text-2xl font-bold text-gray-900">{complianceMetrics.active_investigations}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Search */}
      <div className="flex items-center space-x-3">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
          <Input
            placeholder="Search by wallet address or tier..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
      </div>

      {/* Main Content Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="kyc">KYC/AML</TabsTrigger>
          <TabsTrigger value="risk">Risk Management</TabsTrigger>
          <TabsTrigger value="investigations">Investigations</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>

        {/* Compliance Overview */}
        <TabsContent value="overview" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Compliance Status Overview */}
            <Card>
              <CardHeader>
                <CardTitle>Compliance Status Distribution</CardTitle>
                <CardDescription>Current verification status across all users</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {['verified', 'pending', 'rejected', 'expired'].map(status => {
                    const count = complianceStatuses.filter(s => s.kyc_status === status).length;
                    const percentage = complianceStatuses.length > 0 ? (count / complianceStatuses.length * 100).toFixed(1) : 0;
                    return (
                      <div key={status} className="flex items-center justify-between">
                        <div className="flex items-center">
                          <Badge className={`mr-2 ${getKycStatusBadge(status)}`}>
                            {status.charAt(0).toUpperCase() + status.slice(1)}
                          </Badge>
                        </div>
                        <div className="text-right">
                          <div className="text-sm font-medium">{count} users</div>
                          <div className="text-xs text-gray-500">{percentage}%</div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>

            {/* Recent Suspicious Activities */}
            <Card>
              <CardHeader>
                <CardTitle>Recent Suspicious Activities</CardTitle>
                <CardDescription>Latest flagged activities requiring attention</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {suspiciousActivities.slice(0, 5).map(activity => (
                    <div key={activity.id} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center">
                        {getSeverityIcon(activity.severity)}
                        <div className="ml-3">
                          <p className="text-sm font-medium">{activity.activity_type.replace(/_/g, ' ')}</p>
                          <p className="text-xs text-gray-500">
                            {activity.wallet_address.slice(0, 8)}...{activity.wallet_address.slice(-6)}
                          </p>
                        </div>
                      </div>
                      <Badge
                        className={`
                          ${activity.investigation_status === 'open' ? 'bg-red-100 text-red-800' : ''}
                          ${activity.investigation_status === 'investigating' ? 'bg-yellow-100 text-yellow-800' : ''}
                          ${activity.investigation_status === 'resolved' ? 'bg-green-100 text-green-800' : ''}
                          ${activity.investigation_status === 'false_positive' ? 'bg-gray-100 text-gray-800' : ''}
                        `}
                      >
                        {activity.investigation_status.replace(/_/g, ' ')}
                      </Badge>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* KYC/AML Management */}
        <TabsContent value="kyc" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>KYC/AML Status Management</CardTitle>
              <CardDescription>Manage user verification and compliance status</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {filteredStatuses.map(status => (
                  <div key={status.wallet_address} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-3">
                      <div>
                        <p className="font-medium">{status.wallet_address}</p>
                        <p className="text-sm text-gray-500">Tier: {status.enterprise_tier}</p>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Badge className={getKycStatusBadge(status.kyc_status)}>
                          KYC: {status.kyc_status}
                        </Badge>
                        <Badge className={getAmlStatusBadge(status.aml_status)}>
                          AML: {status.aml_status}
                        </Badge>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-3">
                      <div>
                        <p className="text-xs text-gray-500">Risk Score</p>
                        <p className={`font-medium ${getRiskLevelColor(
                          status.risk_score >= 75 ? 'critical' :
                            status.risk_score >= 50 ? 'high' :
                              status.risk_score >= 25 ? 'medium' : 'low'
                        )}`}>
                          {status.risk_score}/100
                        </p>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500">Last KYC Date</p>
                        <p className="text-sm">{new Date(status.last_kyc_date).toLocaleDateString()}</p>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500">Provider</p>
                        <p className="text-sm">{status.kyc_provider}</p>
                      </div>
                    </div>

                    {status.compliance_flags.length > 0 && (
                      <div className="mb-3">
                        <p className="text-xs text-gray-500 mb-1">Compliance Flags</p>
                        <div className="flex flex-wrap gap-1">
                          {status.compliance_flags.map(flag => (
                            <Badge key={flag} variant="destructive" className="text-xs">
                              {flag}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    )}

                    <div className="flex items-center space-x-2">
                      {status.kyc_status === 'pending' && (
                        <>
                          <Button
                            size="sm"
                            onClick={() => handleApproveKyc(status.wallet_address)}
                          >
                            <CheckCircle className="w-4 h-4 mr-1" />
                            Approve
                          </Button>
                          <Button
                            size="sm"
                            variant="destructive"
                            onClick={() => handleRejectKyc(status.wallet_address, 'Manual review rejection')}
                          >
                            <XCircle className="w-4 h-4 mr-1" />
                            Reject
                          </Button>
                        </>
                      )}
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setSelectedStatus(status)}
                      >
                        <Eye className="w-4 h-4 mr-1" />
                        View Details
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleFlagUser(status.wallet_address, 'manual_review', 'Flagged for review')}
                      >
                        <Flag className="w-4 h-4 mr-1" />
                        Flag
                      </Button>
                      {status.aml_status !== 'blocked' && (
                        <Button
                          size="sm"
                          variant="destructive"
                          onClick={() => handleBlockUser(status.wallet_address, 'Compliance violation')}
                        >
                          <Ban className="w-4 h-4 mr-1" />
                          Block
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Risk Management */}
        <TabsContent value="risk" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Risk Assessment Management</CardTitle>
              <CardDescription>Review and manage automated and manual risk assessments</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {riskAssessments.map(assessment => (
                  <div key={assessment.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-3">
                      <div>
                        <p className="font-medium">
                          {assessment.wallet_address.slice(0, 8)}...{assessment.wallet_address.slice(-6)}
                        </p>
                        <p className="text-sm text-gray-500">
                          {assessment.assessment_type} • {assessment.automated ? 'Automated' : 'Manual'}
                        </p>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Badge className={`
                          ${assessment.risk_level === 'critical' ? 'bg-red-100 text-red-800' : ''}
                          ${assessment.risk_level === 'high' ? 'bg-orange-100 text-orange-800' : ''}
                          ${assessment.risk_level === 'medium' ? 'bg-yellow-100 text-yellow-800' : ''}
                          ${assessment.risk_level === 'low' ? 'bg-green-100 text-green-800' : ''}
                        `}>
                          {assessment.risk_level.toUpperCase()}
                        </Badge>
                        <span className={`font-bold ${getRiskLevelColor(assessment.risk_level)}`}>
                          {assessment.score}/100
                        </span>
                      </div>
                    </div>

                    <div className="mb-3">
                      <p className="text-xs text-gray-500 mb-1">Risk Factors</p>
                      <div className="flex flex-wrap gap-1">
                        {assessment.risk_factors.map(factor => (
                          <Badge key={factor} variant="secondary" className="text-xs">
                            {factor}
                          </Badge>
                        ))}
                      </div>
                    </div>

                    {assessment.review_notes && (
                      <div className="mb-3">
                        <p className="text-xs text-gray-500">Review Notes</p>
                        <p className="text-sm">{assessment.review_notes}</p>
                      </div>
                    )}

                    <div className="flex items-center justify-between">
                      <p className="text-xs text-gray-500">
                        Created: {new Date(assessment.created_at).toLocaleDateString()}
                        {assessment.reviewed_by && ` • Reviewed by: ${assessment.reviewed_by}`}
                      </p>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setSelectedRisk(assessment)}
                      >
                        <Eye className="w-4 h-4 mr-1" />
                        Review
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Investigations */}
        <TabsContent value="investigations" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Suspicious Activity Investigations</CardTitle>
              <CardDescription>Manage ongoing compliance investigations</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {suspiciousActivities.map(activity => (
                  <div key={activity.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-3">
                      <div className="flex items-center">
                        {getSeverityIcon(activity.severity)}
                        <div className="ml-3">
                          <p className="font-medium">{activity.activity_type.replace(/_/g, ' ')}</p>
                          <p className="text-sm text-gray-500">
                            {activity.wallet_address.slice(0, 8)}...{activity.wallet_address.slice(-6)}
                          </p>
                        </div>
                      </div>
                      <Badge className={`
                        ${activity.investigation_status === 'open' ? 'bg-red-100 text-red-800' : ''}
                        ${activity.investigation_status === 'investigating' ? 'bg-yellow-100 text-yellow-800' : ''}
                        ${activity.investigation_status === 'resolved' ? 'bg-green-100 text-green-800' : ''}
                        ${activity.investigation_status === 'false_positive' ? 'bg-gray-100 text-gray-800' : ''}
                      `}>
                        {activity.investigation_status.replace(/_/g, ' ')}
                      </Badge>
                    </div>

                    <p className="text-sm text-gray-700 mb-3">{activity.description}</p>

                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-xs text-gray-500">
                          Detected: {new Date(activity.detected_at).toLocaleDateString()}
                        </p>
                        {activity.assigned_to && (
                          <p className="text-xs text-gray-500">
                            Assigned to: {activity.assigned_to}
                          </p>
                        )}
                      </div>
                      <div className="flex items-center space-x-2">
                        {activity.investigation_status === 'open' && (
                          <Button
                            size="sm"
                            onClick={() => handleInvestigateActivity(activity.id, 'current_admin')}
                          >
                            <Users className="w-4 h-4 mr-1" />
                            Assign Investigation
                          </Button>
                        )}
                        <Button
                          size="sm"
                          variant="outline"
                        >
                          <FileText className="w-4 h-4 mr-1" />
                          View Details
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Regulatory Settings */}
        <TabsContent value="settings" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Regulatory Compliance Settings</CardTitle>
              <CardDescription>Configure compliance requirements and thresholds</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {regulatorySettings.map(setting => (
                  <div key={setting.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <div>
                        <p className="font-medium">{setting.regulation}</p>
                        <p className="text-sm text-gray-500">{setting.jurisdiction}</p>
                      </div>
                      <Badge className={`
                        ${setting.implementation_status === 'active' ? 'bg-green-100 text-green-800' : ''}
                        ${setting.implementation_status === 'pending' ? 'bg-yellow-100 text-yellow-800' : ''}
                        ${setting.implementation_status === 'disabled' ? 'bg-gray-100 text-gray-800' : ''}
                      `}>
                        {setting.implementation_status}
                      </Badge>
                    </div>

                    <p className="text-sm text-gray-700 mb-2">{setting.requirement}</p>

                    {setting.threshold_value && (
                      <p className="text-sm text-gray-500 mb-2">
                        Threshold: ${setting.threshold_value.toLocaleString()}
                      </p>
                    )}

                    <div className="flex items-center justify-between">
                      <p className="text-xs text-gray-500">
                        Last updated: {new Date(setting.last_updated).toLocaleDateString()}
                        {setting.mandatory && ' • Mandatory'}
                      </p>
                      <Button size="sm" variant="outline">
                        Configure
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}