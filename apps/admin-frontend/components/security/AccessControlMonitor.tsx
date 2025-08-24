'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { UserActivityLog } from './UserActivityLog';
import { cn } from '@/lib/utils';
import {
  Lock,
  Users,
  AlertTriangle,
  Eye,
  Clock,
  Globe,
  Shield,
  Activity,
  Search,
  Filter,
  Download,
  RefreshCw,
  CheckCircle,
  XCircle,
  Key,
  UserCheck,
  Zap
} from 'lucide-react';

interface AccessEvent {
  id: string;
  timestamp: Date;
  userId: string;
  userEmail: string;
  action: 'login' | 'logout' | 'permission_check' | 'api_access' | 'failed_login' | 'session_expired';
  resource: string;
  result: 'granted' | 'denied' | 'error';
  ip: string;
  userAgent: string;
  location?: string;
  reason?: string;
  riskScore: number;
}

interface SessionInfo {
  id: string;
  userId: string;
  userEmail: string;
  startTime: Date;
  lastActivity: Date;
  ip: string;
  location: string;
  device: string;
  status: 'active' | 'expired' | 'terminated';
  riskLevel: 'low' | 'medium' | 'high';
  permissions: string[];
}

interface PermissionStat {
  resource: string;
  totalChecks: number;
  granted: number;
  denied: number;
  errorRate: number;
  avgResponseTime: number;
}

export function AccessControlMonitor() {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterAction, setFilterAction] = useState<string>('all');
  const [filterResult, setFilterResult] = useState<string>('all');
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('24h');
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedSession, setSelectedSession] = useState<SessionInfo | null>(null);

  const [accessEvents, setAccessEvents] = useState<AccessEvent[]>([
    {
      id: '1',
      timestamp: new Date(Date.now() - 300000),
      userId: 'user_123',
      userEmail: 'john.doe@example.com',
      action: 'permission_check',
      resource: '/api/admin/users',
      result: 'granted',
      ip: '192.168.1.100',
      userAgent: 'Mozilla/5.0...',
      location: 'New York, US',
      riskScore: 15
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 600000),
      userId: 'user_456',
      userEmail: 'jane.smith@example.com',
      action: 'failed_login',
      resource: '/admin/login',
      result: 'denied',
      ip: '203.0.113.0',
      userAgent: 'Chrome/91.0...',
      location: 'London, UK',
      reason: 'Invalid credentials',
      riskScore: 75
    },
    {
      id: '3',
      timestamp: new Date(Date.now() - 900000),
      userId: 'user_789',
      userEmail: 'admin@epsx.com',
      action: 'api_access',
      resource: '/api/security/threats',
      result: 'granted',
      ip: '198.51.100.0',
      userAgent: 'PostmanRuntime/7.28.4',
      location: 'San Francisco, US',
      riskScore: 25
    },
    {
      id: '4',
      timestamp: new Date(Date.now() - 1200000),
      userId: 'user_101',
      userEmail: 'suspicious@test.com',
      action: 'permission_check',
      resource: '/api/admin/billing',
      result: 'denied',
      ip: '233.252.0.0',
      userAgent: 'curl/7.68.0',
      location: 'Unknown',
      reason: 'Insufficient permissions',
      riskScore: 85
    },
    {
      id: '5',
      timestamp: new Date(Date.now() - 1800000),
      userId: 'user_202',
      userEmail: 'support@epsx.com',
      action: 'login',
      resource: '/admin/dashboard',
      result: 'granted',
      ip: '192.168.1.200',
      userAgent: 'Mozilla/5.0...',
      location: 'Toronto, CA',
      riskScore: 10
    }
  ]);

  const [activeSessions, setActiveSessions] = useState<SessionInfo[]>([
    {
      id: 'sess_001',
      userId: 'user_123',
      userEmail: 'john.doe@example.com',
      startTime: new Date(Date.now() - 7200000),
      lastActivity: new Date(Date.now() - 300000),
      ip: '192.168.1.100',
      location: 'New York, US',
      device: 'Chrome on Windows',
      status: 'active',
      riskLevel: 'low',
      permissions: ['user_operations', 'analytics_specialist']
    },
    {
      id: 'sess_002',
      userId: 'user_456',
      userEmail: 'jane.smith@example.com',
      startTime: new Date(Date.now() - 3600000),
      lastActivity: new Date(Date.now() - 1800000),
      ip: '203.0.113.0',
      location: 'London, UK',
      device: 'Safari on macOS',
      status: 'active',
      riskLevel: 'medium',
      permissions: ['billing_admin', 'support_specialist']
    },
    {
      id: 'sess_003',
      userId: 'user_789',
      userEmail: 'admin@epsx.com',
      startTime: new Date(Date.now() - 14400000),
      lastActivity: new Date(Date.now() - 600000),
      ip: '198.51.100.0',
      location: 'San Francisco, US',
      device: 'Firefox on Linux',
      status: 'active',
      riskLevel: 'low',
      permissions: ['system_admin', 'security_management', 'audit_logs']
    }
  ]);

  const [permissionStats, setPermissionStats] = useState<PermissionStat[]>([
    {
      resource: '/api/admin/users',
      totalChecks: 1247,
      granted: 1198,
      denied: 49,
      errorRate: 0.4,
      avgResponseTime: 12
    },
    {
      resource: '/api/admin/billing',
      totalChecks: 356,
      granted: 342,
      denied: 14,
      errorRate: 1.2,
      avgResponseTime: 15
    },
    {
      resource: '/api/security/threats',
      totalChecks: 89,
      granted: 87,
      denied: 2,
      errorRate: 0.1,
      avgResponseTime: 8
    },
    {
      resource: '/api/analytics/dashboard',
      totalChecks: 2134,
      granted: 2089,
      denied: 45,
      errorRate: 0.3,
      avgResponseTime: 18
    }
  ]);

  // Filter access events
  const filteredEvents = accessEvents.filter(event => {
    const matchesSearch = event.userEmail.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         event.resource.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         event.ip.includes(searchQuery);
    const matchesAction = filterAction === 'all' || event.action === filterAction;
    const matchesResult = filterResult === 'all' || event.result === filterResult;
    
    return matchesSearch && matchesAction && matchesResult;
  });

  const handleRefresh = async () => {
    setIsRefreshing(true);
    // Simulate API refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  const handleTerminateSession = async (sessionId: string) => {
    setActiveSessions(prev => prev.map(session => 
      session.id === sessionId 
        ? { ...session, status: 'terminated' as const }
        : session
    ));
  };

  const getActionIcon = (action: string) => {
    switch (action) {
      case 'login': return UserCheck;
      case 'logout': return Users;
      case 'permission_check': return Lock;
      case 'api_access': return Key;
      case 'failed_login': return AlertTriangle;
      case 'session_expired': return Clock;
      default: return Activity;
    }
  };

  const getResultColor = (result: string) => {
    switch (result) {
      case 'granted': return 'text-green-500 bg-green-500/10';
      case 'denied': return 'text-red-500 bg-red-500/10';
      case 'error': return 'text-orange-500 bg-orange-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  const getResultIcon = (result: string) => {
    switch (result) {
      case 'granted': return CheckCircle;
      case 'denied': return XCircle;
      case 'error': return AlertTriangle;
      default: return Activity;
    }
  };

  const getRiskColor = (score: number) => {
    if (score >= 70) return 'text-red-500 bg-red-500/10';
    if (score >= 40) return 'text-orange-500 bg-orange-500/10';
    if (score >= 20) return 'text-yellow-500 bg-yellow-500/10';
    return 'text-green-500 bg-green-500/10';
  };

  const getRiskLevel = (score: number) => {
    if (score >= 70) return 'High';
    if (score >= 40) return 'Medium';
    if (score >= 20) return 'Low';
    return 'Minimal';
  };

  const getSessionRiskColor = (level: string) => {
    switch (level) {
      case 'high': return 'text-red-500 bg-red-500/10';
      case 'medium': return 'text-orange-500 bg-orange-500/10';
      case 'low': return 'text-green-500 bg-green-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  const calculateSuccessRate = (granted: number, total: number) => {
    return total > 0 ? ((granted / total) * 100).toFixed(1) : '0';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Access Control Monitor</h1>
          <p className="text-muted-foreground mt-1">
            Monitor permissions, sessions, and access patterns across the platform
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button 
            variant="outline" 
            size="sm" 
            onClick={handleRefresh}
            disabled={isRefreshing}
          >
            <RefreshCw className={cn("w-4 h-4 mr-2", isRefreshing && "animate-spin")} />
            Refresh
          </Button>
          <Button variant="outline" size="sm">
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-green-500/10">
                <CheckCircle className="w-5 h-5 text-green-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {accessEvents.filter(e => e.result === 'granted').length}
                </p>
                <p className="text-sm text-muted-foreground">Access Granted</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-red-500/10">
                <XCircle className="w-5 h-5 text-red-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {accessEvents.filter(e => e.result === 'denied').length}
                </p>
                <p className="text-sm text-muted-foreground">Access Denied</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-blue-500/10">
                <Users className="w-5 h-5 text-blue-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">{activeSessions.length}</p>
                <p className="text-sm text-muted-foreground">Active Sessions</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-orange-500/10">
                <AlertTriangle className="w-5 h-5 text-orange-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {accessEvents.filter(e => e.riskScore >= 70).length}
                </p>
                <p className="text-sm text-muted-foreground">High Risk Events</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs defaultValue="events" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="events">Access Events</TabsTrigger>
          <TabsTrigger value="sessions">Active Sessions</TabsTrigger>
          <TabsTrigger value="permissions">Permissions</TabsTrigger>
          <TabsTrigger value="analytics">Analytics</TabsTrigger>
        </TabsList>

        {/* Access Events */}
        <TabsContent value="events" className="space-y-4">
          {/* Filters */}
          <Card>
            <CardHeader>
              <CardTitle>Access Events</CardTitle>
              <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <div className="flex-1">
                  <div className="relative">
                    <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      placeholder="Search by user, resource, or IP..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-9"
                    />
                  </div>
                </div>
                <Select value={filterAction} onValueChange={setFilterAction}>
                  <SelectTrigger className="w-40">
                    <SelectValue placeholder="Action" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Actions</SelectItem>
                    <SelectItem value="login">Login</SelectItem>
                    <SelectItem value="logout">Logout</SelectItem>
                    <SelectItem value="permission_check">Permission Check</SelectItem>
                    <SelectItem value="api_access">API Access</SelectItem>
                    <SelectItem value="failed_login">Failed Login</SelectItem>
                  </SelectContent>
                </Select>
                <Select value={filterResult} onValueChange={setFilterResult}>
                  <SelectTrigger className="w-32">
                    <SelectValue placeholder="Result" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Results</SelectItem>
                    <SelectItem value="granted">Granted</SelectItem>
                    <SelectItem value="denied">Denied</SelectItem>
                    <SelectItem value="error">Error</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {filteredEvents.map((event) => {
                  const ActionIcon = getActionIcon(event.action);
                  const ResultIcon = getResultIcon(event.result);
                  
                  return (
                    <div 
                      key={event.id} 
                      className="flex items-center gap-4 p-4 border rounded-lg hover:bg-muted/50 transition-colors"
                    >
                      <div className={cn("p-2 rounded-full", getResultColor(event.result))}>
                        <ActionIcon className="w-4 h-4" />
                      </div>
                      
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="font-medium text-sm">{event.userEmail}</span>
                          <Badge variant="outline" className="text-xs">
                            {event.action.replace('_', ' ')}
                          </Badge>
                          <Badge className={cn("text-xs flex items-center gap-1", getResultColor(event.result))}>
                            <ResultIcon className="w-3 h-3" />
                            {event.result}
                          </Badge>
                          <Badge className={cn("text-xs", getRiskColor(event.riskScore))}>
                            {getRiskLevel(event.riskScore)} Risk
                          </Badge>
                        </div>
                        <div className="text-xs text-muted-foreground mb-1">
                          Resource: <span className="font-mono">{event.resource}</span>
                        </div>
                        <div className="flex items-center gap-4 text-xs text-muted-foreground">
                          <div className="flex items-center gap-1">
                            <Globe className="w-3 h-3" />
                            {event.ip} {event.location && `• ${event.location}`}
                          </div>
                          <div className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            {event.timestamp.toLocaleString()}
                          </div>
                          {event.reason && (
                            <div className="flex items-center gap-1">
                              <AlertTriangle className="w-3 h-3" />
                              {event.reason}
                            </div>
                          )}
                        </div>
                      </div>
                      
                      <Button variant="ghost" size="sm">
                        <Eye className="w-4 h-4" />
                      </Button>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Active Sessions */}
        <TabsContent value="sessions" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Active User Sessions</CardTitle>
              <p className="text-sm text-muted-foreground">
                Monitor and manage active user sessions across the platform
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {activeSessions.map((session) => (
                  <div 
                    key={session.id} 
                    className="flex items-center gap-4 p-4 border rounded-lg"
                    onClick={() => setSelectedSession(session)}
                  >
                    <div className={cn("p-2 rounded-full", getSessionRiskColor(session.riskLevel))}>
                      <Users className="w-4 h-4" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-medium text-sm">{session.userEmail}</span>
                        <Badge 
                          variant={session.status === 'active' ? 'default' : 'secondary'} 
                          className="text-xs"
                        >
                          {session.status}
                        </Badge>
                        <Badge className={cn("text-xs", getSessionRiskColor(session.riskLevel))}>
                          {session.riskLevel} risk
                        </Badge>
                      </div>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <Globe className="w-3 h-3" />
                          {session.ip} • {session.location}
                        </div>
                        <div className="flex items-center gap-1">
                          <Activity className="w-3 h-3" />
                          {session.device}
                        </div>
                        <div className="flex items-center gap-1">
                          <Clock className="w-3 h-3" />
                          Active: {session.lastActivity.toLocaleTimeString()}
                        </div>
                      </div>
                      <div className="flex flex-wrap gap-1 mt-2">
                        {session.permissions.slice(0, 3).map((permission) => (
                          <Badge key={permission} variant="outline" className="text-xs">
                            {permission}
                          </Badge>
                        ))}
                        {session.permissions.length > 3 && (
                          <Badge variant="outline" className="text-xs">
                            +{session.permissions.length - 3} more
                          </Badge>
                        )}
                      </div>
                    </div>
                    
                    {session.status === 'active' && (
                      <Button 
                        size="sm" 
                        variant="destructive"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleTerminateSession(session.id);
                        }}
                      >
                        <Zap className="w-4 h-4 mr-1" />
                        Terminate
                      </Button>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {selectedSession && (
            <Card>
              <CardHeader>
                <CardTitle>Session Details</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold mb-2">Session Information</h4>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>User:</span>
                        <span>{selectedSession.userEmail}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Session ID:</span>
                        <span className="font-mono">{selectedSession.id}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Started:</span>
                        <span>{selectedSession.startTime.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Last Activity:</span>
                        <span>{selectedSession.lastActivity.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Status:</span>
                        <Badge variant={selectedSession.status === 'active' ? 'default' : 'secondary'}>
                          {selectedSession.status}
                        </Badge>
                      </div>
                    </div>
                  </div>
                  
                  <div>
                    <h4 className="font-semibold mb-2">Permissions</h4>
                    <div className="flex flex-wrap gap-2">
                      {selectedSession.permissions.map((permission) => (
                        <Badge key={permission} variant="outline" className="text-xs">
                          {permission}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* Permission Statistics */}
        <TabsContent value="permissions" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Permission Validation Statistics</CardTitle>
              <p className="text-sm text-muted-foreground">
                Analysis of permission checks and validation performance
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {permissionStats.map((stat) => (
                  <div key={stat.resource} className="flex items-center gap-4 p-4 border rounded-lg">
                    <div className="p-2 rounded-full bg-blue-500/10">
                      <Lock className="w-4 h-4 text-blue-500" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-medium text-sm font-mono">{stat.resource}</span>
                        <Badge variant="outline" className="text-xs">
                          {calculateSuccessRate(stat.granted, stat.totalChecks)}% success
                        </Badge>
                      </div>
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs text-muted-foreground">
                        <div>
                          <span className="font-medium text-green-500">{stat.granted}</span> granted
                        </div>
                        <div>
                          <span className="font-medium text-red-500">{stat.denied}</span> denied
                        </div>
                        <div>
                          <span className="font-medium">{stat.avgResponseTime}ms</span> avg response
                        </div>
                        <div>
                          <span className="font-medium text-orange-500">{stat.errorRate}%</span> error rate
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Analytics */}
        <TabsContent value="analytics" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>User Activity Analytics</CardTitle>
              <p className="text-sm text-muted-foreground">
                Detailed user behavior analysis and anomaly detection
              </p>
            </CardHeader>
            <CardContent>
              <UserActivityLog events={accessEvents} />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}