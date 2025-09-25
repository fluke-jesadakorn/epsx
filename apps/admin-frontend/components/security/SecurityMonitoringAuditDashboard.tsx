'use client'

import { useState, useEffect, useCallback, useMemo } from 'react'
import { PancakeCard } from '../ui/PancakeCard'
import { Button } from '../ui/button'
import { Input } from '../ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select'
import { Badge } from '../ui/badge'
import { AlertTriangle, Shield, Eye, Clock, Download, Filter, Search, TrendingUp, AlertCircle, CheckCircle, XCircle } from 'lucide-react'

// Comprehensive Security Event Types
interface SecurityEvent {
  id: string
  timestamp: string
  eventType: 'input_threat' | 'xss_attempt' | 'csrf_violation' | 'permission_breach' | 'data_exposure' | 'authentication_anomaly' | 'system_intrusion'
  severity: 'low' | 'medium' | 'high' | 'critical'
  source: string
  userId?: string
  userEmail?: string
  component: string
  description: string
  threatVector?: string
  blocked: boolean
  remediationAction?: string
  metadata: Record<string, any>
  ipAddress?: string
  userAgent?: string
  sessionId?: string
  riskScore: number
  complianceImpact: string[]
  remediated: boolean
}

interface ThreatMetrics {
  totalThreats: number
  blockedThreats: number
  activeIncidents: number
  riskScore: number
  complianceScore: number
  lastThreatTime?: string
  threatTrends: {
    period: string
    count: number
    severity: string
  }[]
}

interface SecurityAlert {
  id: string
  type: 'critical_threat' | 'compliance_violation' | 'system_anomaly' | 'user_behavior_anomaly'
  priority: 'urgent' | 'high' | 'medium' | 'low'
  title: string
  description: string
  timestamp: string
  acknowledged: boolean
  resolved: boolean
  assignedTo?: string
  relatedEvents: string[]
}

interface ComplianceStatus {
  gdpr: { status: 'compliant' | 'warning' | 'violation', lastCheck: string, issues: string[] }
  pci: { status: 'compliant' | 'warning' | 'violation', lastCheck: string, issues: string[] }
  sox: { status: 'compliant' | 'warning' | 'violation', lastCheck: string, issues: string[] }
  hipaa: { status: 'compliant' | 'warning' | 'violation', lastCheck: string, issues: string[] }
}

interface SecurityMonitoringAuditDashboardProps {
  className?: string
  autoRefresh?: boolean
  refreshInterval?: number
}

export const SecurityMonitoringAuditDashboard: React.FC<SecurityMonitoringAuditDashboardProps> = ({
  className = '',
  autoRefresh = true,
  refreshInterval = 30000
}) => {
  const [events, setEvents] = useState<SecurityEvent[]>([])
  const [alerts, setAlerts] = useState<SecurityAlert[]>([])
  const [metrics, setMetrics] = useState<ThreatMetrics | null>(null)
  const [complianceStatus, setComplianceStatus] = useState<ComplianceStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  
  // Filtering and Search
  const [searchTerm, setSearchTerm] = useState('')
  const [severityFilter, setSeverityFilter] = useState<string>('all')
  const [eventTypeFilter, setEventTypeFilter] = useState<string>('all')
  const [dateRangeFilter, setDateRangeFilter] = useState<string>('24h')
  const [showOnlyUnresolved, setShowOnlyUnresolved] = useState(false)

  // Real-time monitoring
  const [isRealTimeEnabled, setIsRealTimeEnabled] = useState(true)
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)

  // Load security events and metrics
  const loadSecurityData = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      // Mock data for comprehensive security monitoring
      const mockEvents: SecurityEvent[] = [
        {
          id: 'evt_001',
          timestamp: new Date().toISOString(),
          eventType: 'input_threat',
          severity: 'high',
          source: 'ComprehensiveInputSanitizer',
          userId: 'usr_123',
          userEmail: 'user@example.com',
          component: 'UserManagement',
          description: 'SQL injection attempt detected in user input',
          threatVector: 'sql_injection',
          blocked: true,
          remediationAction: 'Input sanitized and blocked',
          metadata: { inputField: 'username', sanitizedValue: 'user***', threatPattern: 'UNION SELECT' },
          ipAddress: '192.168.1.100',
          userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
          sessionId: 'sess_456',
          riskScore: 85,
          complianceImpact: ['gdpr', 'pci'],
          remediated: true
        },
        {
          id: 'evt_002',
          timestamp: new Date(Date.now() - 300000).toISOString(),
          eventType: 'xss_attempt',
          severity: 'critical',
          source: 'AdvancedXSSCSRFProtection',
          userId: 'usr_789',
          userEmail: 'attacker@malicious.com',
          component: 'PermissionForm',
          description: 'Cross-site scripting attack blocked',
          threatVector: 'reflected_xss',
          blocked: true,
          remediationAction: 'Script tags sanitized, user session flagged',
          metadata: { scriptContent: '<script>alert("xss")</script>', sanitizedContent: '&lt;script&gt;alert("xss")&lt;/script&gt;' },
          ipAddress: '10.0.0.50',
          userAgent: 'Malicious Bot 1.0',
          sessionId: 'sess_789',
          riskScore: 95,
          complianceImpact: ['gdpr', 'sox'],
          remediated: false
        },
        {
          id: 'evt_003',
          timestamp: new Date(Date.now() - 600000).toISOString(),
          eventType: 'permission_breach',
          severity: 'high',
          source: 'SecurePermissionContextManager',
          userId: 'usr_456',
          userEmail: 'insider@company.com',
          component: 'AdminPanel',
          description: 'Unauthorized access attempt to admin resources',
          threatVector: 'privilege_escalation',
          blocked: true,
          remediationAction: 'Access denied, security team notified',
          metadata: { requestedPermission: 'admin:users:delete', userPermission: 'user:profile:read' },
          ipAddress: '172.16.0.25',
          userAgent: 'Chrome/119.0.0.0',
          sessionId: 'sess_101',
          riskScore: 78,
          complianceImpact: ['sox', 'hipaa'],
          remediated: true
        }
      ]

      const mockAlerts: SecurityAlert[] = [
        {
          id: 'alert_001',
          type: 'critical_threat',
          priority: 'urgent',
          title: 'Multiple XSS Attempts Detected',
          description: 'Coordinated XSS attack from IP range 10.0.0.0/24',
          timestamp: new Date().toISOString(),
          acknowledged: false,
          resolved: false,
          relatedEvents: ['evt_002']
        },
        {
          id: 'alert_002',
          type: 'compliance_violation',
          priority: 'high',
          title: 'GDPR Compliance Warning',
          description: 'Sensitive data exposure detected in API responses',
          timestamp: new Date(Date.now() - 900000).toISOString(),
          acknowledged: true,
          resolved: false,
          assignedTo: 'security@company.com',
          relatedEvents: ['evt_001', 'evt_003']
        }
      ]

      const mockMetrics: ThreatMetrics = {
        totalThreats: 247,
        blockedThreats: 242,
        activeIncidents: 3,
        riskScore: 23,
        complianceScore: 87,
        lastThreatTime: new Date(Date.now() - 300000).toISOString(),
        threatTrends: [
          { period: '00:00', count: 5, severity: 'low' },
          { period: '06:00', count: 12, severity: 'medium' },
          { period: '12:00', count: 8, severity: 'high' },
          { period: '18:00', count: 15, severity: 'critical' }
        ]
      }

      const mockComplianceStatus: ComplianceStatus = {
        gdpr: { status: 'warning', lastCheck: new Date().toISOString(), issues: ['Data exposure in evt_001'] },
        pci: { status: 'compliant', lastCheck: new Date().toISOString(), issues: [] },
        sox: { status: 'compliant', lastCheck: new Date().toISOString(), issues: [] },
        hipaa: { status: 'compliant', lastCheck: new Date().toISOString(), issues: [] }
      }

      setEvents(mockEvents)
      setAlerts(mockAlerts)
      setMetrics(mockMetrics)
      setComplianceStatus(mockComplianceStatus)
      setLastUpdate(new Date())

    } catch (err) {
      setError(`Failed to load security data: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setLoading(false)
    }
  }, [])

  // Auto-refresh functionality
  useEffect(() => {
    loadSecurityData()
    
    if (autoRefresh && isRealTimeEnabled) {
      const interval = setInterval(loadSecurityData, refreshInterval)
      return () => clearInterval(interval)
    }
  }, [loadSecurityData, autoRefresh, isRealTimeEnabled, refreshInterval])

  // Filter events based on current filters
  const filteredEvents = useMemo(() => {
    return events.filter(event => {
      const matchesSearch = searchTerm === '' || 
        event.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.userEmail?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.component.toLowerCase().includes(searchTerm.toLowerCase())
      
      const matchesSeverity = severityFilter === 'all' || event.severity === severityFilter
      const matchesEventType = eventTypeFilter === 'all' || event.eventType === eventTypeFilter
      const matchesResolved = !showOnlyUnresolved || !event.remediated

      // Date range filtering
      const eventTime = new Date(event.timestamp).getTime()
      const now = Date.now()
      const matchesDateRange = (() => {
        switch (dateRangeFilter) {
          case '1h': return now - eventTime <= 3600000
          case '24h': return now - eventTime <= 86400000
          case '7d': return now - eventTime <= 604800000
          case '30d': return now - eventTime <= 2592000000
          default: return true
        }
      })()

      return matchesSearch && matchesSeverity && matchesEventType && matchesResolved && matchesDateRange
    })
  }, [events, searchTerm, severityFilter, eventTypeFilter, dateRangeFilter, showOnlyUnresolved])

  // Export security data
  const exportSecurityData = useCallback(() => {
    const dataToExport = {
      exportTime: new Date().toISOString(),
      events: filteredEvents,
      alerts,
      metrics,
      complianceStatus
    }
    
    const blob = new Blob([JSON.stringify(dataToExport, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `security-audit-${new Date().toISOString().split('T')[0]}.json`
    a.click()
    URL.revokeObjectURL(url)
  }, [filteredEvents, alerts, metrics, complianceStatus])

  // Acknowledge alert
  const acknowledgeAlert = useCallback(async (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId ? { ...alert, acknowledged: true } : alert
    ))
  }, [])

  // Resolve alert
  const resolveAlert = useCallback(async (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId ? { ...alert, resolved: true, acknowledged: true } : alert
    ))
  }, [])

  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'critical': return 'bg-red-100 text-red-800 border-red-200'
      case 'high': return 'bg-orange-100 text-orange-800 border-orange-200'
      case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200'
      case 'low': return 'bg-blue-100 text-blue-800 border-blue-200'
      default: return 'bg-gray-100 text-gray-800 border-gray-200'
    }
  }

  const getComplianceStatusIcon = (status: string) => {
    switch (status) {
      case 'compliant': return <CheckCircle className="w-4 h-4 text-green-600" />
      case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-600" />
      case 'violation': return <XCircle className="w-4 h-4 text-red-600" />
      default: return <AlertCircle className="w-4 h-4 text-gray-600" />
    }
  }

  if (loading) {
    return (
      <div className={`space-y-4 ${className}`}>
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded mb-4"></div>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="h-24 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <PancakeCard className={className}>
        <div className="flex items-center space-x-2 text-red-600">
          <AlertTriangle className="w-5 h-5" />
          <span>Security Dashboard Error: {error}</span>
        </div>
        <Button onClick={loadSecurityData} className="mt-4">Retry</Button>
      </PancakeCard>
    )
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header with Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center space-x-2">
          <Shield className="w-6 h-6 text-blue-600" />
          <h2 className="text-2xl font-semibold">Security Monitoring Dashboard</h2>
          {isRealTimeEnabled && (
            <Badge variant="outline" className="bg-green-50 text-green-700 border-green-200">
              Live
            </Badge>
          )}
        </div>
        <div className="flex items-center space-x-2">
          <Button
            onClick={() => setIsRealTimeEnabled(!isRealTimeEnabled)}
            variant={isRealTimeEnabled ? 'default' : 'outline'}
            size="sm"
          >
            <Eye className="w-4 h-4 mr-1" />
            {isRealTimeEnabled ? 'Live' : 'Paused'}
          </Button>
          <Button onClick={exportSecurityData} variant="outline" size="sm">
            <Download className="w-4 h-4 mr-1" />
            Export
          </Button>
        </div>
      </div>

      {/* Last Update Indicator */}
      {lastUpdate && (
        <div className="flex items-center text-sm text-gray-600">
          <Clock className="w-4 h-4 mr-1" />
          Last updated: {lastUpdate.toLocaleTimeString()}
        </div>
      )}

      {/* Security Metrics Overview */}
      {metrics && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
          <PancakeCard className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Total Threats</p>
                <p className="text-2xl font-semibold">{metrics.totalThreats}</p>
              </div>
              <TrendingUp className="w-8 h-8 text-blue-600" />
            </div>
          </PancakeCard>
          <PancakeCard className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Blocked</p>
                <p className="text-2xl font-semibold text-green-600">{metrics.blockedThreats}</p>
              </div>
              <Shield className="w-8 h-8 text-green-600" />
            </div>
          </PancakeCard>
          <PancakeCard className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Active Incidents</p>
                <p className="text-2xl font-semibold text-red-600">{metrics.activeIncidents}</p>
              </div>
              <AlertTriangle className="w-8 h-8 text-red-600" />
            </div>
          </PancakeCard>
          <PancakeCard className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Risk Score</p>
                <p className="text-2xl font-semibold">{metrics.riskScore}/100</p>
              </div>
              <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                metrics.riskScore > 70 ? 'bg-red-100 text-red-600' :
                metrics.riskScore > 40 ? 'bg-yellow-100 text-yellow-600' :
                'bg-green-100 text-green-600'
              }`}>
                {metrics.riskScore}
              </div>
            </div>
          </PancakeCard>
          <PancakeCard className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Compliance</p>
                <p className="text-2xl font-semibold">{metrics.complianceScore}%</p>
              </div>
              <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                metrics.complianceScore > 90 ? 'bg-green-100 text-green-600' :
                metrics.complianceScore > 70 ? 'bg-yellow-100 text-yellow-600' :
                'bg-red-100 text-red-600'
              }`}>
                ✓
              </div>
            </div>
          </PancakeCard>
        </div>
      )}

      {/* Compliance Status */}
      {complianceStatus && (
        <PancakeCard className="p-6">
          <h3 className="text-lg font-semibold mb-4">Compliance Status</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {Object.entries(complianceStatus).map(([standard, status]) => (
              <div key={standard} className="flex items-center space-x-3 p-3 border rounded-lg">
                {getComplianceStatusIcon(status.status)}
                <div>
                  <p className="font-medium uppercase">{standard}</p>
                  <p className="text-sm text-gray-600 capitalize">{status.status}</p>
                  {status.issues.length > 0 && (
                    <p className="text-xs text-red-600">{status.issues.length} issues</p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </PancakeCard>
      )}

      {/* Active Alerts */}
      {alerts.length > 0 && (
        <PancakeCard className="p-6">
          <h3 className="text-lg font-semibold mb-4">Active Security Alerts</h3>
          <div className="space-y-3">
            {alerts.filter(alert => !alert.resolved).map(alert => (
              <div key={alert.id} className={`p-4 border rounded-lg ${
                alert.priority === 'urgent' ? 'border-red-200 bg-red-50' :
                alert.priority === 'high' ? 'border-orange-200 bg-orange-50' :
                'border-yellow-200 bg-yellow-50'
              }`}>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-2">
                      <Badge className={getSeverityColor(alert.priority)}>
                        {alert.priority.toUpperCase()}
                      </Badge>
                      <Badge variant="outline">{alert.type.replace('_', ' ')}</Badge>
                    </div>
                    <h4 className="font-medium">{alert.title}</h4>
                    <p className="text-sm text-gray-600 mt-1">{alert.description}</p>
                    <p className="text-xs text-gray-500 mt-2">
                      {new Date(alert.timestamp).toLocaleString()}
                    </p>
                  </div>
                  <div className="flex space-x-2">
                    {!alert.acknowledged && (
                      <Button
                        onClick={() => acknowledgeAlert(alert.id)}
                        variant="outline"
                        size="sm"
                      >
                        Acknowledge
                      </Button>
                    )}
                    <Button
                      onClick={() => resolveAlert(alert.id)}
                      variant="default"
                      size="sm"
                    >
                      Resolve
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </PancakeCard>
      )}

      {/* Filters */}
      <PancakeCard className="p-6">
        <div className="flex flex-col sm:flex-row gap-4 items-end">
          <div className="flex-1">
            <label className="block text-sm font-medium mb-2">Search Events</label>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
              <Input
                placeholder="Search by description, user, or component..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">Severity</label>
            <Select value={severityFilter} onValueChange={setSeverityFilter}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All</SelectItem>
                <SelectItem value="critical">Critical</SelectItem>
                <SelectItem value="high">High</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="low">Low</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">Event Type</label>
            <Select value={eventTypeFilter} onValueChange={setEventTypeFilter}>
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Types</SelectItem>
                <SelectItem value="input_threat">Input Threat</SelectItem>
                <SelectItem value="xss_attempt">XSS Attempt</SelectItem>
                <SelectItem value="csrf_violation">CSRF Violation</SelectItem>
                <SelectItem value="permission_breach">Permission Breach</SelectItem>
                <SelectItem value="data_exposure">Data Exposure</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">Time Range</label>
            <Select value={dateRangeFilter} onValueChange={setDateRangeFilter}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1h">Last Hour</SelectItem>
                <SelectItem value="24h">Last 24h</SelectItem>
                <SelectItem value="7d">Last 7 days</SelectItem>
                <SelectItem value="30d">Last 30 days</SelectItem>
                <SelectItem value="all">All Time</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </PancakeCard>

      {/* Security Events Table */}
      <PancakeCard className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">Security Events ({filteredEvents.length})</h3>
          <Button
            onClick={() => setShowOnlyUnresolved(!showOnlyUnresolved)}
            variant={showOnlyUnresolved ? 'default' : 'outline'}
            size="sm"
          >
            <Filter className="w-4 h-4 mr-1" />
            {showOnlyUnresolved ? 'Show All' : 'Unresolved Only'}
          </Button>
        </div>
        
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b">
                <th className="text-left py-3 px-4">Timestamp</th>
                <th className="text-left py-3 px-4">Type</th>
                <th className="text-left py-3 px-4">Severity</th>
                <th className="text-left py-3 px-4">User</th>
                <th className="text-left py-3 px-4">Component</th>
                <th className="text-left py-3 px-4">Description</th>
                <th className="text-left py-3 px-4">Risk Score</th>
                <th className="text-left py-3 px-4">Status</th>
              </tr>
            </thead>
            <tbody>
              {filteredEvents.map(event => (
                <tr key={event.id} className="border-b hover:bg-gray-50">
                  <td className="py-3 px-4 text-sm">
                    {new Date(event.timestamp).toLocaleString()}
                  </td>
                  <td className="py-3 px-4">
                    <Badge variant="outline" className="text-xs">
                      {event.eventType.replace('_', ' ')}
                    </Badge>
                  </td>
                  <td className="py-3 px-4">
                    <Badge className={getSeverityColor(event.severity)}>
                      {event.severity}
                    </Badge>
                  </td>
                  <td className="py-3 px-4 text-sm">
                    <div>
                      {event.userEmail && (
                        <div className="font-medium">{event.userEmail}</div>
                      )}
                      <div className="text-gray-500">{event.ipAddress}</div>
                    </div>
                  </td>
                  <td className="py-3 px-4 text-sm font-medium">{event.component}</td>
                  <td className="py-3 px-4 text-sm max-w-xs truncate" title={event.description}>
                    {event.description}
                  </td>
                  <td className="py-3 px-4">
                    <div className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                      event.riskScore > 80 ? 'bg-red-100 text-red-800' :
                      event.riskScore > 60 ? 'bg-orange-100 text-orange-800' :
                      event.riskScore > 40 ? 'bg-yellow-100 text-yellow-800' :
                      'bg-green-100 text-green-800'
                    }`}>
                      {event.riskScore}
                    </div>
                  </td>
                  <td className="py-3 px-4">
                    {event.blocked ? (
                      <Badge className="bg-green-100 text-green-800 border-green-200">
                        Blocked
                      </Badge>
                    ) : (
                      <Badge className="bg-red-100 text-red-800 border-red-200">
                        Allowed
                      </Badge>
                    )}
                    {event.remediated && (
                      <Badge className="bg-blue-100 text-blue-800 border-blue-200 ml-1">
                        Resolved
                      </Badge>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          
          {filteredEvents.length === 0 && (
            <div className="text-center py-8 text-gray-500">
              No security events found matching your filters.
            </div>
          )}
        </div>
      </PancakeCard>
    </div>
  )
}