'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { cn } from '@/lib/utils';
import {
  Zap,
  AlertTriangle,
  Shield,
  Eye,
  Clock,
  Users,
  FileText,
  Search,
  Filter,
  Download,
  RefreshCw,
  Play,
  Pause,
  CheckCircle,
  XCircle,
  MessageSquare,
  Flag,
  Target,
  Activity,
  Globe,
  Lock,
  Database
} from 'lucide-react';
import { toast } from 'react-hot-toast';

interface SecurityIncident {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  status: 'open' | 'investigating' | 'contained' | 'resolved' | 'closed';
  category: 'breach' | 'malware' | 'phishing' | 'ddos' | 'insider_threat' | 'system_compromise';
  createdAt: Date;
  updatedAt: Date;
  assignedTo?: string;
  reporter: string;
  affectedSystems: string[];
  evidence: Evidence[];
  timeline: TimelineEvent[];
  impacts: Impact[];
  responseActions: ResponseAction[];
  riskScore: number;
}

interface Evidence {
  id: string;
  type: 'log' | 'screenshot' | 'file' | 'network_capture' | 'forensic_image';
  name: string;
  description: string;
  collectedAt: Date;
  collectedBy: string;
  hash?: string;
  size?: number;
  location: string;
}

interface TimelineEvent {
  id: string;
  timestamp: Date;
  event: string;
  description: string;
  actor: string;
  evidence?: string[];
}

interface Impact {
  type: 'confidentiality' | 'integrity' | 'availability' | 'financial' | 'reputation';
  severity: 'low' | 'medium' | 'high';
  description: string;
  estimated_cost?: number;
}

interface ResponseAction {
  id: string;
  action: string;
  description: string;
  assignedTo: string;
  status: 'pending' | 'in_progress' | 'completed' | 'blocked';
  createdAt: Date;
  completedAt?: Date;
  priority: 'low' | 'medium' | 'high' | 'critical';
}

interface InvestigationTool {
  name: string;
  description: string;
  status: 'available' | 'running' | 'error';
  lastUsed?: Date;
  icon: React.ComponentType<any>;
}

export function IncidentResponse() {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [selectedIncident, setSelectedIncident] = useState<SecurityIncident | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [activePlaybook, setActivePlaybook] = useState<string | null>(null);

  const [incidents, setIncidents] = useState<SecurityIncident[]>([
    {
      id: '1',
      title: 'Suspected Data Breach - Customer Database',
      description: 'Unusual database access patterns detected. Large volume of customer data accessed by compromised admin account.',
      severity: 'critical',
      status: 'investigating',
      category: 'breach',
      createdAt: new Date(Date.now() - 7200000),
      updatedAt: new Date(Date.now() - 1800000),
      assignedTo: 'security-team-lead@epsx.com',
      reporter: 'automated-system',
      affectedSystems: ['customer_database', 'admin_portal', 'api_gateway'],
      evidence: [
        {
          id: 'e1',
          type: 'log',
          name: 'Database Access Logs',
          description: 'Suspicious queries executed between 14:30-15:45 UTC',
          collectedAt: new Date(Date.now() - 5400000),
          collectedBy: 'security-team@epsx.com',
          location: '/var/log/database/access.log',
          size: 2048576
        }
      ],
      timeline: [
        {
          id: 't1',
          timestamp: new Date(Date.now() - 7200000),
          event: 'Anomaly Detected',
          description: 'Automated systems detected unusual database access patterns',
          actor: 'Security Monitoring System'
        },
        {
          id: 't2',
          timestamp: new Date(Date.now() - 6600000),
          event: 'Investigation Started',
          description: 'Security team began investigating the anomaly',
          actor: 'security-team-lead@epsx.com'
        }
      ],
      impacts: [
        {
          type: 'confidentiality',
          severity: 'high',
          description: 'Potential exposure of 10,000+ customer records',
          estimated_cost: 500000
        }
      ],
      responseActions: [
        {
          id: 'a1',
          action: 'Isolate Compromised Account',
          description: 'Disable the compromised admin account and revoke all sessions',
          assignedTo: 'admin-team@epsx.com',
          status: 'completed',
          createdAt: new Date(Date.now() - 6000000),
          completedAt: new Date(Date.now() - 5400000),
          priority: 'critical'
        },
        {
          id: 'a2',
          action: 'Forensic Data Collection',
          description: 'Collect database logs and system artifacts for analysis',
          assignedTo: 'forensics-team@epsx.com',
          status: 'in_progress',
          createdAt: new Date(Date.now() - 5400000),
          priority: 'high'
        }
      ],
      riskScore: 95
    },
    {
      id: '2',
      title: 'DDoS Attack on API Gateway',
      description: 'Distributed denial of service attack targeting our API endpoints. Traffic volume increased 1000x normal levels.',
      severity: 'high',
      status: 'contained',
      category: 'ddos',
      createdAt: new Date(Date.now() - 14400000),
      updatedAt: new Date(Date.now() - 3600000),
      assignedTo: 'network-team@epsx.com',
      reporter: 'monitoring-system',
      affectedSystems: ['api_gateway', 'load_balancer', 'cdn'],
      evidence: [],
      timeline: [],
      impacts: [
        {
          type: 'availability',
          severity: 'medium',
          description: 'API response times increased by 300%'
        }
      ],
      responseActions: [],
      riskScore: 70
    }
  ]);

  const [investigationTools, setInvestigationTools] = useState<InvestigationTool[]>([
    {
      name: 'Network Analyzer',
      description: 'Analyze network traffic and detect anomalies',
      status: 'available',
      icon: Globe
    },
    {
      name: 'Log Correlator',
      description: 'Correlate events across multiple log sources',
      status: 'running',
      lastUsed: new Date(Date.now() - 1800000),
      icon: FileText
    },
    {
      name: 'Malware Scanner',
      description: 'Scan systems for malicious software',
      status: 'available',
      icon: Shield
    },
    {
      name: 'Forensic Imager',
      description: 'Create forensic images of compromised systems',
      status: 'available',
      icon: Database
    },
    {
      name: 'Threat Intelligence',
      description: 'Query external threat intelligence feeds',
      status: 'available',
      icon: Target
    }
  ]);

  // Filter incidents
  const filteredIncidents = incidents.filter(incident => {
    const matchesSearch = incident.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         incident.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = filterStatus === 'all' || incident.status === filterStatus;
    const matchesSeverity = filterSeverity === 'all' || incident.severity === filterSeverity;
    
    return matchesSearch && matchesStatus && matchesSeverity;
  });

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  const handleStatusChange = async (incidentId: string, newStatus: string) => {
    setIncidents(prev => prev.map(incident => 
      incident.id === incidentId 
        ? { ...incident, status: newStatus as any, updatedAt: new Date() }
        : incident
    ));
    toast.success('Incident status updated');
  };

  const handleAssignIncident = async (incidentId: string, assignee: string) => {
    setIncidents(prev => prev.map(incident => 
      incident.id === incidentId 
        ? { ...incident, assignedTo: assignee, updatedAt: new Date() }
        : incident
    ));
    toast.success('Incident assigned');
  };

  const startPlaybook = (playbookType: string) => {
    setActivePlaybook(playbookType);
    toast.success(`${playbookType} playbook activated`);
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

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'open': return 'text-red-500 bg-red-500/10';
      case 'investigating': return 'text-yellow-500 bg-yellow-500/10';
      case 'contained': return 'text-blue-500 bg-blue-500/10';
      case 'resolved': return 'text-green-500 bg-green-500/10';
      case 'closed': return 'text-gray-500 bg-gray-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'breach': return Shield;
      case 'malware': return AlertTriangle;
      case 'ddos': return Zap;
      case 'phishing': return Target;
      case 'insider_threat': return Users;
      case 'system_compromise': return Lock;
      default: return AlertTriangle;
    }
  };

  const getActionStatusIcon = (status: string) => {
    switch (status) {
      case 'completed': return CheckCircle;
      case 'in_progress': return Play;
      case 'blocked': return XCircle;
      default: return Pause;
    }
  };

  const formatCategory = (category: string) => {
    return category.split('_').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Incident Response</h1>
          <p className="text-muted-foreground mt-1">
            Manage security incidents, coordinate response, and conduct investigations
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
          <Button size="sm" variant="destructive">
            <AlertTriangle className="w-4 h-4 mr-2" />
            Create Incident
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
                  {incidents.filter(i => i.status === 'open' || i.status === 'investigating').length}
                </p>
                <p className="text-sm text-muted-foreground">Active Incidents</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-orange-500/10">
                <Flag className="w-5 h-5 text-orange-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {incidents.filter(i => i.severity === 'critical').length}
                </p>
                <p className="text-sm text-muted-foreground">Critical Severity</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-blue-500/10">
                <Clock className="w-5 h-5 text-blue-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">2.5h</p>
                <p className="text-sm text-muted-foreground">Avg Response Time</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-green-500/10">
                <CheckCircle className="w-5 h-5 text-green-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">
                  {incidents.filter(i => i.status === 'resolved' || i.status === 'closed').length}
                </p>
                <p className="text-sm text-muted-foreground">Resolved Today</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs defaultValue="incidents" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="incidents">Active Incidents</TabsTrigger>
          <TabsTrigger value="investigation">Investigation Tools</TabsTrigger>
          <TabsTrigger value="playbooks">Response Playbooks</TabsTrigger>
          <TabsTrigger value="reports">Incident Reports</TabsTrigger>
        </TabsList>

        {/* Active Incidents */}
        <TabsContent value="incidents" className="space-y-4">
          {/* Filters */}
          <Card>
            <CardHeader>
              <CardTitle>Security Incidents</CardTitle>
              <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <div className="flex-1">
                  <div className="relative">
                    <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      placeholder="Search incidents..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-9"
                    />
                  </div>
                </div>
                <Select value={filterStatus} onValueChange={setFilterStatus}>
                  <SelectTrigger className="w-40">
                    <SelectValue placeholder="Status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Status</SelectItem>
                    <SelectItem value="open">Open</SelectItem>
                    <SelectItem value="investigating">Investigating</SelectItem>
                    <SelectItem value="contained">Contained</SelectItem>
                    <SelectItem value="resolved">Resolved</SelectItem>
                  </SelectContent>
                </Select>
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
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {filteredIncidents.map((incident) => {
                  const CategoryIcon = getCategoryIcon(incident.category);
                  
                  return (
                    <Card 
                      key={incident.id}
                      className={cn(
                        "transition-all hover:shadow-md cursor-pointer",
                        selectedIncident?.id === incident.id && "ring-2 ring-primary",
                        incident.severity === 'critical' && incident.status === 'open' && "animate-pulse"
                      )}
                      onClick={() => setSelectedIncident(selectedIncident?.id === incident.id ? null : incident)}
                    >
                      <CardContent className="p-4">
                        <div className="flex items-start gap-4">
                          <div className={cn("p-2 rounded-full", getSeverityColor(incident.severity))}>
                            <CategoryIcon className="w-5 h-5" />
                          </div>
                          
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-2">
                              <h4 className="font-semibold">{incident.title}</h4>
                              <Badge className={cn("text-xs", getSeverityColor(incident.severity))}>
                                {incident.severity}
                              </Badge>
                              <Badge className={cn("text-xs", getStatusColor(incident.status))}>
                                {incident.status}
                              </Badge>
                              <Badge variant="outline" className="text-xs">
                                {formatCategory(incident.category)}
                              </Badge>
                              <Badge variant="outline" className="text-xs">
                                Risk: {incident.riskScore}
                              </Badge>
                            </div>
                            
                            <p className="text-sm text-muted-foreground mb-2">
                              {incident.description}
                            </p>
                            
                            <div className="flex items-center gap-4 text-xs text-muted-foreground">
                              <div className="flex items-center gap-1">
                                <Clock className="w-3 h-3" />
                                Created: {incident.createdAt.toLocaleString()}
                              </div>
                              <div className="flex items-center gap-1">
                                <Activity className="w-3 h-3" />
                                Updated: {incident.updatedAt.toLocaleString()}
                              </div>
                              {incident.assignedTo && (
                                <div className="flex items-center gap-1">
                                  <Users className="w-3 h-3" />
                                  Assigned: {incident.assignedTo}
                                </div>
                              )}
                              <div className="flex items-center gap-1">
                                <Shield className="w-3 h-3" />
                                Systems: {incident.affectedSystems.length}
                              </div>
                            </div>
                          </div>
                          
                          <div className="flex flex-col gap-1">
                            <Select 
                              defaultValue={incident.status} 
                              onValueChange={(value) => handleStatusChange(incident.id, value)}
                            >
                              <SelectTrigger className="w-32">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="open">Open</SelectItem>
                                <SelectItem value="investigating">Investigating</SelectItem>
                                <SelectItem value="contained">Contained</SelectItem>
                                <SelectItem value="resolved">Resolved</SelectItem>
                                <SelectItem value="closed">Closed</SelectItem>
                              </SelectContent>
                            </Select>
                            <Button variant="ghost" size="sm">
                              <Eye className="w-4 h-4" />
                            </Button>
                          </div>
                        </div>
                        
                        {selectedIncident?.id === incident.id && (
                          <div className="mt-4 pt-4 border-t border-border space-y-4">
                            {/* Affected Systems */}
                            <div>
                              <h5 className="font-semibold mb-2">Affected Systems</h5>
                              <div className="flex flex-wrap gap-2">
                                {incident.affectedSystems.map((system, index) => (
                                  <Badge key={index} variant="outline" className="text-xs">
                                    {system}
                                  </Badge>
                                ))}
                              </div>
                            </div>
                            
                            {/* Response Actions */}
                            {incident.responseActions.length > 0 && (
                              <div>
                                <h5 className="font-semibold mb-2">Response Actions</h5>
                                <div className="space-y-2">
                                  {incident.responseActions.map((action) => {
                                    const StatusIcon = getActionStatusIcon(action.status);
                                    
                                    return (
                                      <div key={action.id} className="flex items-center gap-3 p-2 border rounded text-sm">
                                        <StatusIcon className={cn(
                                          "w-4 h-4",
                                          action.status === 'completed' ? "text-green-500" :
                                          action.status === 'in_progress' ? "text-blue-500" :
                                          action.status === 'blocked' ? "text-red-500" : "text-gray-500"
                                        )} />
                                        <div className="flex-1">
                                          <div className="font-medium">{action.action}</div>
                                          <div className="text-muted-foreground text-xs">
                                            Assigned to: {action.assignedTo}
                                          </div>
                                        </div>
                                        <Badge 
                                          variant="outline" 
                                          className={cn(
                                            "text-xs",
                                            action.priority === 'critical' ? "border-red-500 text-red-500" :
                                            action.priority === 'high' ? "border-orange-500 text-orange-500" : ""
                                          )}
                                        >
                                          {action.priority}
                                        </Badge>
                                      </div>
                                    );
                                  })}
                                </div>
                              </div>
                            )}
                            
                            {/* Evidence */}
                            {incident.evidence.length > 0 && (
                              <div>
                                <h5 className="font-semibold mb-2">Evidence Collected</h5>
                                <div className="space-y-2">
                                  {incident.evidence.map((evidence) => (
                                    <div key={evidence.id} className="flex items-center gap-3 p-2 border rounded text-sm">
                                      <FileText className="w-4 h-4 text-blue-500" />
                                      <div className="flex-1">
                                        <div className="font-medium">{evidence.name}</div>
                                        <div className="text-muted-foreground text-xs">
                                          {evidence.description}
                                        </div>
                                      </div>
                                      <Badge variant="outline" className="text-xs">
                                        {evidence.type}
                                      </Badge>
                                      <Button variant="ghost" size="sm">
                                        <Download className="w-4 h-4" />
                                      </Button>
                                    </div>
                                  ))}
                                </div>
                              </div>
                            )}
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

        {/* Investigation Tools */}
        <TabsContent value="investigation" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Eye className="w-5 h-5" />
                Investigation & Forensic Tools
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Digital forensics and investigation tools for incident analysis
              </p>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {investigationTools.map((tool) => {
                  const ToolIcon = tool.icon;
                  
                  return (
                    <Card key={tool.name} className="hover:shadow-md transition-all cursor-pointer">
                      <CardContent className="p-4">
                        <div className="flex items-center gap-3 mb-3">
                          <div className="p-2 rounded-full bg-blue-500/10">
                            <ToolIcon className="w-5 h-5 text-blue-500" />
                          </div>
                          <div className="flex-1">
                            <h4 className="font-semibold">{tool.name}</h4>
                            <Badge 
                              variant="outline"
                              className={cn(
                                "text-xs",
                                tool.status === 'running' ? "border-green-500 text-green-500" :
                                tool.status === 'error' ? "border-red-500 text-red-500" : ""
                              )}
                            >
                              {tool.status}
                            </Badge>
                          </div>
                        </div>
                        
                        <p className="text-sm text-muted-foreground mb-3">
                          {tool.description}
                        </p>
                        
                        {tool.lastUsed && (
                          <div className="text-xs text-muted-foreground mb-3">
                            Last used: {tool.lastUsed.toLocaleString()}
                          </div>
                        )}
                        
                        <Button 
                          size="sm" 
                          className="w-full"
                          disabled={tool.status === 'running'}
                        >
                          {tool.status === 'running' ? (
                            <>
                              <Pause className="w-4 h-4 mr-2" />
                              Running
                            </>
                          ) : (
                            <>
                              <Play className="w-4 h-4 mr-2" />
                              Launch Tool
                            </>
                          )}
                        </Button>
                      </CardContent>
                    </Card>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Response Playbooks */}
        <TabsContent value="playbooks" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <FileText className="w-5 h-5" />
                Incident Response Playbooks
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Standardized response procedures for different types of security incidents
              </p>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {[
                  { name: 'Data Breach Response', description: 'Comprehensive response to data breaches and unauthorized access', steps: 12, severity: 'critical' },
                  { name: 'Malware Incident', description: 'Detection, containment, and removal of malicious software', steps: 8, severity: 'high' },
                  { name: 'DDoS Attack Response', description: 'Mitigation strategies for distributed denial of service attacks', steps: 6, severity: 'medium' },
                  { name: 'Phishing Campaign', description: 'Response to phishing attacks and email-based threats', steps: 10, severity: 'medium' },
                  { name: 'Insider Threat', description: 'Investigation and response to internal security threats', steps: 15, severity: 'high' },
                  { name: 'System Compromise', description: 'Response to compromised systems and unauthorized access', steps: 11, severity: 'critical' }
                ].map((playbook) => (
                  <Card key={playbook.name} className="hover:shadow-md transition-all">
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between mb-3">
                        <div>
                          <h4 className="font-semibold">{playbook.name}</h4>
                          <Badge className={cn("text-xs mt-1", getSeverityColor(playbook.severity))}>
                            {playbook.severity} priority
                          </Badge>
                        </div>
                        <Badge variant="outline" className="text-xs">
                          {playbook.steps} steps
                        </Badge>
                      </div>
                      
                      <p className="text-sm text-muted-foreground mb-4">
                        {playbook.description}
                      </p>
                      
                      <div className="flex gap-2">
                        <Button 
                          size="sm" 
                          onClick={() => startPlaybook(playbook.name)}
                          disabled={activePlaybook === playbook.name}
                        >
                          <Play className="w-4 h-4 mr-2" />
                          {activePlaybook === playbook.name ? 'Active' : 'Start Playbook'}
                        </Button>
                        <Button size="sm" variant="outline">
                          <Eye className="w-4 h-4 mr-2" />
                          Preview
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Incident Reports */}
        <TabsContent value="reports" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <FileText className="w-5 h-5" />
                Incident Reports & Documentation
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Generate comprehensive reports for incidents, compliance, and post-mortem analysis
              </p>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <h4 className="font-semibold mb-4">Report Templates</h4>
                  <div className="space-y-3">
                    {[
                      'Executive Summary Report',
                      'Technical Incident Report',
                      'Compliance Audit Report',
                      'Post-Mortem Analysis',
                      'Timeline Documentation',
                      'Evidence Chain of Custody'
                    ].map((template) => (
                      <div key={template} className="flex items-center justify-between p-3 border rounded-lg">
                        <span className="text-sm">{template}</span>
                        <Button size="sm" variant="outline">
                          <Download className="w-4 h-4 mr-1" />
                          Generate
                        </Button>
                      </div>
                    ))}
                  </div>
                </div>
                
                <div>
                  <h4 className="font-semibold mb-4">Recent Reports</h4>
                  <div className="space-y-3">
                    {[
                      { name: 'Q4 Security Incidents Summary', date: '2024-01-15', type: 'summary' },
                      { name: 'Data Breach Investigation Report', date: '2024-01-12', type: 'investigation' },
                      { name: 'Compliance Audit Results', date: '2024-01-10', type: 'compliance' }
                    ].map((report) => (
                      <div key={report.name} className="flex items-center justify-between p-3 border rounded-lg">
                        <div>
                          <div className="text-sm font-medium">{report.name}</div>
                          <div className="text-xs text-muted-foreground">{report.date}</div>
                        </div>
                        <div className="flex gap-1">
                          <Button size="sm" variant="ghost">
                            <Eye className="w-4 h-4" />
                          </Button>
                          <Button size="sm" variant="ghost">
                            <Download className="w-4 h-4" />
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}