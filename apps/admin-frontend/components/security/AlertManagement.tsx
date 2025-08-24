'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { AlertAcknowledgment } from './AlertAcknowledgment';
import { cn } from '@/lib/utils';
import {
  Bell,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  Settings,
  Zap,
  Shield,
  Activity,
  Plus,
  Edit,
  Trash2,
  Send,
  Globe,
  Webhook,
  Mail,
  MessageSquare,
  RefreshCw,
  Eye,
  Mute,
  Volume2
} from 'lucide-react';
import { toast } from 'react-hot-toast';

interface SecurityAlert {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  category: 'authentication' | 'access_control' | 'network' | 'system' | 'data_breach';
  status: 'active' | 'acknowledged' | 'resolved' | 'muted';
  createdAt: Date;
  updatedAt: Date;
  acknowledgedBy?: string;
  acknowledgedAt?: Date;
  source: string;
  affectedResources: string[];
  recommendations: string[];
  riskScore: number;
}

interface AlertRule {
  id: string;
  name: string;
  description: string;
  condition: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  enabled: boolean;
  thresholds: { [key: string]: number };
  actions: AlertAction[];
  createdAt: Date;
  lastTriggered?: Date;
  triggerCount: number;
}

interface AlertAction {
  type: 'webhook' | 'email' | 'slack' | 'sms';
  target: string;
  enabled: boolean;
  template?: string;
}

interface WebhookEndpoint {
  id: string;
  name: string;
  url: string;
  method: 'POST' | 'PUT' | 'PATCH';
  headers: { [key: string]: string };
  active: boolean;
  lastPing?: Date;
  responseTime?: number;
  status: 'healthy' | 'error' | 'timeout';
  retryCount: number;
  successRate: number;
}

export function AlertManagement() {
  const [activeTab, setActiveTab] = useState('alerts');
  const [searchQuery, setSearchQuery] = useState('');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedAlert, setSelectedAlert] = useState<SecurityAlert | null>(null);

  const [alerts, setAlerts] = useState<SecurityAlert[]>([
    {
      id: '1',
      title: 'Multiple Failed Login Attempts',
      description: 'Detected 15 failed login attempts from IP 192.168.1.100 in the last 5 minutes',
      severity: 'high',
      category: 'authentication',
      status: 'active',
      createdAt: new Date(Date.now() - 300000),
      updatedAt: new Date(Date.now() - 300000),
      source: 'Authentication System',
      affectedResources: ['/admin/login', 'user_authentication_service'],
      recommendations: ['Block IP address', 'Review user accounts', 'Enable MFA'],
      riskScore: 75
    },
    {
      id: '2',
      title: 'Suspicious API Access Pattern',
      description: 'Unusual API access pattern detected: 500 requests in 1 minute from automated client',
      severity: 'medium',
      category: 'access_control',
      status: 'acknowledged',
      createdAt: new Date(Date.now() - 1800000),
      updatedAt: new Date(Date.now() - 900000),
      acknowledgedBy: 'admin@epsx.com',
      acknowledgedAt: new Date(Date.now() - 900000),
      source: 'API Gateway',
      affectedResources: ['/api/v1/trading/data', 'rate_limiter'],
      recommendations: ['Review API keys', 'Adjust rate limits', 'Monitor client behavior'],
      riskScore: 45
    },
    {
      id: '3',
      title: 'Critical System Resource Usage',
      description: 'Server CPU usage exceeded 95% for more than 10 minutes',
      severity: 'critical',
      category: 'system',
      status: 'active',
      createdAt: new Date(Date.now() - 600000),
      updatedAt: new Date(Date.now() - 600000),
      source: 'System Monitor',
      affectedResources: ['main_application_server', 'database_connections'],
      recommendations: ['Scale infrastructure', 'Optimize queries', 'Check for memory leaks'],
      riskScore: 90
    },
    {
      id: '4',
      title: 'Potential SQL Injection Attempt',
      description: 'Malicious SQL patterns detected in user input parameters',
      severity: 'high',
      category: 'data_breach',
      status: 'resolved',
      createdAt: new Date(Date.now() - 3600000),
      updatedAt: new Date(Date.now() - 1800000),
      acknowledgedBy: 'security@epsx.com',
      acknowledgedAt: new Date(Date.now() - 3000000),
      source: 'WAF',
      affectedResources: ['/api/search', 'user_input_validation'],
      recommendations: ['Update input validation', 'Review query parameters', 'Audit database logs'],
      riskScore: 80
    }
  ]);

  const [alertRules, setAlertRules] = useState<AlertRule[]>([
    {
      id: '1',
      name: 'Failed Login Threshold',
      description: 'Trigger alert when failed login attempts exceed threshold',
      condition: 'failed_logins > threshold',
      severity: 'high',
      enabled: true,
      thresholds: { failed_logins: 10, time_window: 300 },
      actions: [
        { type: 'webhook', target: 'security_webhook', enabled: true },
        { type: 'email', target: 'security@epsx.com', enabled: true }
      ],
      createdAt: new Date(Date.now() - 86400000),
      lastTriggered: new Date(Date.now() - 300000),
      triggerCount: 15
    },
    {
      id: '2',
      name: 'High API Usage',
      description: 'Monitor for unusual API usage patterns',
      condition: 'api_requests_per_minute > threshold',
      severity: 'medium',
      enabled: true,
      thresholds: { api_requests_per_minute: 100, consecutive_minutes: 5 },
      actions: [
        { type: 'webhook', target: 'monitoring_webhook', enabled: true },
        { type: 'slack', target: '#alerts-channel', enabled: false }
      ],
      createdAt: new Date(Date.now() - 172800000),
      lastTriggered: new Date(Date.now() - 1800000),
      triggerCount: 8
    }
  ]);

  const [webhookEndpoints, setWebhookEndpoints] = useState<WebhookEndpoint[]>([
    {
      id: '1',
      name: 'Security Webhook',
      url: 'https://hooks.epsx.com/security',
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-API-Key': '***' },
      active: true,
      lastPing: new Date(Date.now() - 300000),
      responseTime: 150,
      status: 'healthy',
      retryCount: 0,
      successRate: 99.2
    },
    {
      id: '2',
      name: 'Monitoring Webhook',
      url: 'https://monitoring.epsx.com/webhook',
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      active: true,
      lastPing: new Date(Date.now() - 1200000),
      responseTime: 2500,
      status: 'timeout',
      retryCount: 3,
      successRate: 87.5
    }
  ]);

  // Filter alerts
  const filteredAlerts = alerts.filter(alert => {
    const matchesSearch = alert.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         alert.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesSeverity = filterSeverity === 'all' || alert.severity === filterSeverity;
    const matchesStatus = filterStatus === 'all' || alert.status === filterStatus;
    
    return matchesSearch && matchesSeverity && matchesStatus;
  });

  const handleRefresh = async () => {
    setIsRefreshing(true);
    // Simulate API refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  const handleAcknowledgeAlert = async (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId 
        ? { 
            ...alert, 
            status: 'acknowledged' as const,
            acknowledgedBy: 'current_user@epsx.com',
            acknowledgedAt: new Date(),
            updatedAt: new Date()
          }
        : alert
    ));
    toast.success('Alert acknowledged');
  };

  const handleResolveAlert = async (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId 
        ? { 
            ...alert, 
            status: 'resolved' as const,
            updatedAt: new Date()
          }
        : alert
    ));
    toast.success('Alert resolved');
  };

  const handleMuteAlert = async (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId 
        ? { 
            ...alert, 
            status: 'muted' as const,
            updatedAt: new Date()
          }
        : alert
    ));
    toast.success('Alert muted');
  };

  const handleToggleRule = async (ruleId: string) => {
    setAlertRules(prev => prev.map(rule => 
      rule.id === ruleId 
        ? { ...rule, enabled: !rule.enabled }
        : rule
    ));
    toast.success('Alert rule updated');
  };

  const handleTestWebhook = async (webhookId: string) => {
    const webhook = webhookEndpoints.find(w => w.id === webhookId);
    if (!webhook) return;

    try {
      // Simulate webhook test
      await new Promise(resolve => setTimeout(resolve, 1000));
      setWebhookEndpoints(prev => prev.map(w => 
        w.id === webhookId 
          ? { 
              ...w, 
              lastPing: new Date(),
              responseTime: Math.floor(Math.random() * 500) + 100,
              status: 'healthy' as const
            }
          : w
      ));
      toast.success('Webhook test successful');
    } catch (error) {
      toast.error('Webhook test failed');
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'text-red-500 bg-red-500/10 border-red-500/20';
      case 'high': return 'text-orange-500 bg-orange-500/10 border-orange-500/20';
      case 'medium': return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/20';
      case 'low': return 'text-blue-500 bg-blue-500/10 border-blue-500/20';
      default: return 'text-gray-500 bg-gray-500/10 border-gray-500/20';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return AlertTriangle;
      case 'acknowledged': return CheckCircle;
      case 'resolved': return XCircle;
      case 'muted': return Mute;
      default: return Bell;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-red-500';
      case 'acknowledged': return 'text-yellow-500';
      case 'resolved': return 'text-green-500';
      case 'muted': return 'text-gray-500';
      default: return 'text-blue-500';
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'authentication': return Shield;
      case 'access_control': return Lock;
      case 'network': return Globe;
      case 'system': return Activity;
      case 'data_breach': return AlertTriangle;
      default: return Bell;
    }
  };

  const getWebhookStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'text-green-500 bg-green-500/10';
      case 'error': return 'text-red-500 bg-red-500/10';
      case 'timeout': return 'text-orange-500 bg-orange-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Alert Management</h1>
          <p className="text-muted-foreground mt-1">
            Monitor security alerts, configure notifications, and manage webhook endpoints
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
          <Button size="sm">
            <Plus className="w-4 h-4 mr-2" />
            New Rule
          </Button>
        </div>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-red-500/10">
                <AlertTriangle className="w-5 h-5 text-red-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {alerts.filter(a => a.status === 'active').length}
                </p>
                <p className="text-sm text-muted-foreground">Active Alerts</p>
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
                  {alerts.filter(a => a.severity === 'critical').length}
                </p>
                <p className="text-sm text-muted-foreground">Critical Alerts</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-blue-500/10">
                <Settings className="w-5 h-5 text-blue-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {alertRules.filter(r => r.enabled).length}
                </p>
                <p className="text-sm text-muted-foreground">Active Rules</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-green-500/10">
                <Webhook className="w-5 h-5 text-green-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {webhookEndpoints.filter(w => w.active && w.status === 'healthy').length}
                </p>
                <p className="text-sm text-muted-foreground">Healthy Webhooks</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs defaultValue="alerts" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="alerts">Active Alerts</TabsTrigger>
          <TabsTrigger value="rules">Alert Rules</TabsTrigger>
          <TabsTrigger value="webhooks">Webhooks</TabsTrigger>
          <TabsTrigger value="notifications">Notifications</TabsTrigger>
        </TabsList>

        {/* Active Alerts */}
        <TabsContent value="alerts" className="space-y-4">
          {/* Filters */}
          <Card>
            <CardHeader>
              <CardTitle>Security Alerts</CardTitle>
              <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <div className="flex-1">
                  <div className="relative">
                    <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      placeholder="Search alerts..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-9"
                    />
                  </div>
                </div>
                <Select value={filterSeverity} onValueChange={setFilterSeverity}>
                  <SelectTrigger className="w-32">
                    <SelectValue placeholder="Severity" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Severities</SelectItem>
                    <SelectItem value="critical">Critical</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="low">Low</SelectItem>
                  </SelectContent>
                </Select>
                <Select value={filterStatus} onValueChange={setFilterStatus}>
                  <SelectTrigger className="w-32">
                    <SelectValue placeholder="Status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Status</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="acknowledged">Acknowledged</SelectItem>
                    <SelectItem value="resolved">Resolved</SelectItem>
                    <SelectItem value="muted">Muted</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {filteredAlerts.map((alert) => {
                  const StatusIcon = getStatusIcon(alert.status);
                  const CategoryIcon = getCategoryIcon(alert.category);
                  
                  return (
                    <Card 
                      key={alert.id}
                      className={cn(
                        "transition-all hover:shadow-md cursor-pointer",
                        alert.status === 'active' && alert.severity === 'critical' && "animate-pulse"
                      )}
                      onClick={() => setSelectedAlert(selectedAlert?.id === alert.id ? null : alert)}
                    >
                      <CardContent className="p-4">
                        <div className="flex items-start gap-4">
                          <div className={cn("p-2 rounded-full", getSeverityColor(alert.severity))}>
                            <CategoryIcon className="w-4 h-4" />
                          </div>
                          
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-2">
                              <h4 className="font-semibold">{alert.title}</h4>
                              <Badge className={cn("text-xs", getSeverityColor(alert.severity))}>
                                {alert.severity}
                              </Badge>
                              <Badge 
                                variant="outline" 
                                className={cn("text-xs", getStatusColor(alert.status))}
                              >
                                <StatusIcon className="w-3 h-3 mr-1" />
                                {alert.status}
                              </Badge>
                              <Badge variant="outline" className="text-xs">
                                Risk: {alert.riskScore}
                              </Badge>
                            </div>
                            <p className="text-sm text-muted-foreground mb-2">
                              {alert.description}
                            </p>
                            <div className="flex items-center gap-4 text-xs text-muted-foreground">
                              <div className="flex items-center gap-1">
                                <Clock className="w-3 h-3" />
                                {alert.createdAt.toLocaleString()}
                              </div>
                              <div className="flex items-center gap-1">
                                <Shield className="w-3 h-3" />
                                {alert.source}
                              </div>
                              {alert.acknowledgedBy && (
                                <div className="flex items-center gap-1">
                                  <CheckCircle className="w-3 h-3" />
                                  Ack by {alert.acknowledgedBy}
                                </div>
                              )}
                            </div>
                          </div>
                          
                          <AlertAcknowledgment
                            alert={alert}
                            onAcknowledge={handleAcknowledgeAlert}
                            onResolve={handleResolveAlert}
                            onMute={handleMuteAlert}
                          />
                        </div>
                        
                        {selectedAlert?.id === alert.id && (
                          <div className="mt-4 pt-4 border-t border-border space-y-4">
                            <div>
                              <h5 className="font-semibold mb-2">Affected Resources</h5>
                              <div className="flex flex-wrap gap-2">
                                {alert.affectedResources.map((resource, index) => (
                                  <Badge key={index} variant="outline" className="text-xs">
                                    {resource}
                                  </Badge>
                                ))}
                              </div>
                            </div>
                            
                            <div>
                              <h5 className="font-semibold mb-2">Recommendations</h5>
                              <ul className="text-sm text-muted-foreground space-y-1">
                                {alert.recommendations.map((rec, index) => (
                                  <li key={index} className="flex items-start gap-2">
                                    <span className="text-primary">•</span>
                                    {rec}
                                  </li>
                                ))}
                              </ul>
                            </div>
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Alert Rules */}
        <TabsContent value="rules" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Alert Rules Configuration</CardTitle>
              <p className="text-sm text-muted-foreground">
                Define conditions and thresholds that trigger security alerts
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {alertRules.map((rule) => (
                  <div key={rule.id} className="flex items-center gap-4 p-4 border rounded-lg">
                    <div className="p-2 rounded-full bg-blue-500/10">
                      <Settings className="w-4 h-4 text-blue-500" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <h4 className="font-semibold">{rule.name}</h4>
                        <Badge className={cn("text-xs", getSeverityColor(rule.severity))}>
                          {rule.severity}
                        </Badge>
                        <Badge 
                          variant={rule.enabled ? 'default' : 'secondary'} 
                          className="text-xs"
                        >
                          {rule.enabled ? 'Enabled' : 'Disabled'}
                        </Badge>
                        {rule.lastTriggered && (
                          <Badge variant="outline" className="text-xs">
                            Triggered {rule.triggerCount} times
                          </Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground mb-2">
                        {rule.description}
                      </p>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <Activity className="w-3 h-3" />
                          Condition: {rule.condition}
                        </div>
                        <div className="flex items-center gap-1">
                          <Zap className="w-3 h-3" />
                          {rule.actions.length} actions
                        </div>
                        {rule.lastTriggered && (
                          <div className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            Last: {rule.lastTriggered.toLocaleString()}
                          </div>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <Switch
                        checked={rule.enabled}
                        onCheckedChange={() => handleToggleRule(rule.id)}
                      />
                      <Button variant="ghost" size="sm">
                        <Edit className="w-4 h-4" />
                      </Button>
                      <Button variant="ghost" size="sm">
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Webhooks */}
        <TabsContent value="webhooks" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Webhook Endpoints</CardTitle>
              <p className="text-sm text-muted-foreground">
                Configure and monitor webhook endpoints for alert notifications
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {webhookEndpoints.map((webhook) => (
                  <div key={webhook.id} className="flex items-center gap-4 p-4 border rounded-lg">
                    <div className={cn("p-2 rounded-full", getWebhookStatusColor(webhook.status))}>
                      <Webhook className="w-4 h-4" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <h4 className="font-semibold">{webhook.name}</h4>
                        <Badge 
                          className={cn("text-xs", getWebhookStatusColor(webhook.status))}
                        >
                          {webhook.status}
                        </Badge>
                        <Badge 
                          variant={webhook.active ? 'default' : 'secondary'} 
                          className="text-xs"
                        >
                          {webhook.active ? 'Active' : 'Inactive'}
                        </Badge>
                      </div>
                      <div className="text-sm text-muted-foreground mb-2 font-mono">
                        {webhook.method} {webhook.url}
                      </div>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        {webhook.lastPing && (
                          <div className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            Last ping: {webhook.lastPing.toLocaleString()}
                          </div>
                        )}
                        {webhook.responseTime && (
                          <div className="flex items-center gap-1">
                            <Activity className="w-3 h-3" />
                            {webhook.responseTime}ms
                          </div>
                        )}
                        <div className="flex items-center gap-1">
                          <CheckCircle className="w-3 h-3" />
                          {webhook.successRate}% success
                        </div>
                        {webhook.retryCount > 0 && (
                          <div className="flex items-center gap-1 text-orange-500">
                            <RefreshCw className="w-3 h-3" />
                            {webhook.retryCount} retries
                          </div>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <Button 
                        variant="outline" 
                        size="sm"
                        onClick={() => handleTestWebhook(webhook.id)}
                      >
                        <Send className="w-4 h-4 mr-1" />
                        Test
                      </Button>
                      <Button variant="ghost" size="sm">
                        <Edit className="w-4 h-4" />
                      </Button>
                      <Button variant="ghost" size="sm">
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Notifications */}
        <TabsContent value="notifications" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Notification Channels</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center gap-3">
                    <Mail className="w-5 h-5 text-blue-500" />
                    <div>
                      <div className="font-medium">Email Notifications</div>
                      <div className="text-sm text-muted-foreground">security@epsx.com</div>
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
                
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center gap-3">
                    <MessageSquare className="w-5 h-5 text-green-500" />
                    <div>
                      <div className="font-medium">Slack Integration</div>
                      <div className="text-sm text-muted-foreground">#security-alerts</div>
                    </div>
                  </div>
                  <Switch />
                </div>
                
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center gap-3">
                    <Bell className="w-5 h-5 text-purple-500" />
                    <div>
                      <div className="font-medium">Push Notifications</div>
                      <div className="text-sm text-muted-foreground">Browser notifications</div>
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Notification Settings</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>Minimum Severity Level</Label>
                  <Select defaultValue="medium">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="low">Low</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="high">High</SelectItem>
                      <SelectItem value="critical">Critical Only</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="flex items-center justify-between">
                  <div>
                    <Label>Quiet Hours</Label>
                    <div className="text-sm text-muted-foreground">
                      Suppress non-critical alerts during quiet hours
                    </div>
                  </div>
                  <Switch />
                </div>
                
                <div className="flex items-center justify-between">
                  <div>
                    <Label>Alert Grouping</Label>
                    <div className="text-sm text-muted-foreground">
                      Group similar alerts to reduce noise
                    </div>
                  </div>
                  <Switch defaultChecked />
                </div>
                
                <div>
                  <Label>Auto-resolve After</Label>
                  <Select defaultValue="24">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">1 hour</SelectItem>
                      <SelectItem value="6">6 hours</SelectItem>
                      <SelectItem value="24">24 hours</SelectItem>
                      <SelectItem value="168">7 days</SelectItem>
                      <SelectItem value="never">Never</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}