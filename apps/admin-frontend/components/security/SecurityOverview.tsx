'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/utils';
import {
  Shield,
  AlertTriangle,
  Target,
  Lock,
  Activity,
  Zap,
  TrendingUp,
  TrendingDown,
  Clock,
  Globe,
  Server,
  Users,
  Eye,
  RefreshCw,
  Bell,
  ChevronRight
} from 'lucide-react';
import Link from 'next/link';

interface SecurityMetric {
  title: string;
  value: string;
  change: string;
  trend: 'up' | 'down' | 'stable';
  icon: React.ComponentType<any>;
  critical?: boolean;
  description: string;
}

interface SecurityEvent {
  id: string;
  timestamp: Date;
  type: 'attack' | 'alert' | 'access_denied' | 'system';
  severity: 'low' | 'medium' | 'high' | 'critical';
  source: string;
  description: string;
  location?: string;
}

interface ThreatSummary {
  blockedIPs: number;
  activeThreats: number;
  blockedAttacks: number;
  riskScore: number;
}

export function SecurityOverview() {
  const [currentTime, setCurrentTime] = useState(new Date());
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [threatSummary, setThreatSummary] = useState<ThreatSummary>({
    blockedIPs: 1247,
    activeThreats: 3,
    blockedAttacks: 8934,
    riskScore: 23
  });

  const [recentEvents, setRecentEvents] = useState<SecurityEvent[]>([
    {
      id: '1',
      timestamp: new Date(Date.now() - 300000), // 5 minutes ago
      type: 'attack',
      severity: 'high',
      source: '192.168.1.100',
      description: 'Brute force login attempt detected',
      location: 'New York, US'
    },
    {
      id: '2',
      timestamp: new Date(Date.now() - 600000), // 10 minutes ago
      type: 'alert',
      severity: 'medium',
      source: 'System',
      description: 'Unusual API access pattern detected',
      location: 'London, UK'
    },
    {
      id: '3',
      timestamp: new Date(Date.now() - 900000), // 15 minutes ago
      type: 'access_denied',
      severity: 'low',
      source: '203.0.113.0',
      description: 'Access attempt to restricted endpoint',
      location: 'Tokyo, JP'
    },
    {
      id: '4',
      timestamp: new Date(Date.now() - 1200000), // 20 minutes ago
      type: 'system',
      severity: 'medium',
      source: 'Security Engine',
      description: 'Webhook endpoint health check failed',
    },
    {
      id: '5',
      timestamp: new Date(Date.now() - 1800000), // 30 minutes ago
      type: 'attack',
      severity: 'critical',
      source: '198.51.100.0',
      description: 'SQL injection attempt blocked',
      location: 'São Paulo, BR'
    }
  ]);

  // Update current time
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  // Simulate real-time data updates
  useEffect(() => {
    const interval = setInterval(() => {
      setThreatSummary(prev => ({
        ...prev,
        blockedIPs: prev.blockedIPs + Math.floor(Math.random() * 3),
        activeThreats: Math.max(0, prev.activeThreats + Math.floor(Math.random() * 3) - 1),
        blockedAttacks: prev.blockedAttacks + Math.floor(Math.random() * 10),
        riskScore: Math.max(0, Math.min(100, prev.riskScore + Math.floor(Math.random() * 10) - 5))
      }));
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, []);

  const securityMetrics: SecurityMetric[] = [
    {
      title: 'Blocked IPs',
      value: threatSummary.blockedIPs.toLocaleString(),
      change: '+12 today',
      trend: 'up',
      icon: Target,
      description: 'IP addresses currently blocked by security system'
    },
    {
      title: 'Active Threats',
      value: threatSummary.activeThreats.toString(),
      change: '-2 this hour',
      trend: 'down',
      icon: AlertTriangle,
      critical: threatSummary.activeThreats > 5,
      description: 'Currently active security threats requiring attention'
    },
    {
      title: 'Blocked Attacks',
      value: threatSummary.blockedAttacks.toLocaleString(),
      change: '+156 today',
      trend: 'up',
      icon: Shield,
      description: 'Total attack attempts blocked since last reset'
    },
    {
      title: 'Risk Score',
      value: `${threatSummary.riskScore}/100`,
      change: threatSummary.riskScore > 50 ? 'High' : threatSummary.riskScore > 25 ? 'Medium' : 'Low',
      trend: threatSummary.riskScore > 50 ? 'up' : 'stable',
      icon: Activity,
      critical: threatSummary.riskScore > 75,
      description: 'Overall system security risk assessment'
    }
  ];

  const handleRefresh = async () => {
    setIsRefreshing(true);
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500';
      case 'high': return 'bg-orange-500';
      case 'medium': return 'bg-yellow-500';
      case 'low': return 'bg-blue-500';
      default: return 'bg-gray-500';
    }
  };

  const getSeverityBadgeVariant = (severity: string) => {
    switch (severity) {
      case 'critical': return 'destructive';
      case 'high': return 'destructive';
      case 'medium': return 'secondary';
      case 'low': return 'outline';
      default: return 'outline';
    }
  };

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'attack': return Shield;
      case 'alert': return Bell;
      case 'access_denied': return Lock;
      case 'system': return Server;
      default: return Activity;
    }
  };

  const getRiskScoreColor = (score: number) => {
    if (score > 75) return 'text-red-500';
    if (score > 50) return 'text-orange-500';
    if (score > 25) return 'text-yellow-500';
    return 'text-green-500';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Security Overview</h1>
          <p className="text-muted-foreground mt-1">
            Real-time security monitoring and threat detection dashboard
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
          <Badge variant="outline" className="flex items-center gap-1">
            <Clock className="w-3 h-3" />
            Last updated: {currentTime.toLocaleTimeString()}
          </Badge>
        </div>
      </div>

      {/* Security Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {securityMetrics.map((metric) => {
          const MetricIcon = metric.icon;
          const TrendIcon = metric.trend === 'up' ? TrendingUp : metric.trend === 'down' ? TrendingDown : Activity;
          
          return (
            <Card key={metric.title} className={cn(
              "transition-all hover:shadow-md",
              metric.critical && "ring-2 ring-red-500/20 bg-red-500/5"
            )}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {metric.title}
                </CardTitle>
                <MetricIcon className={cn(
                  "h-4 w-4",
                  metric.critical ? "text-red-500" : "text-muted-foreground"
                )} />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{metric.value}</div>
                <div className="flex items-center gap-1 text-xs text-muted-foreground mt-1">
                  <TrendIcon className={cn(
                    "w-3 h-3",
                    metric.trend === 'up' && metric.critical ? "text-red-500" : 
                    metric.trend === 'up' ? "text-green-500" :
                    metric.trend === 'down' ? "text-red-500" : "text-muted-foreground"
                  )} />
                  <span>{metric.change}</span>
                </div>
                <p className="text-xs text-muted-foreground mt-2 leading-relaxed">
                  {metric.description}
                </p>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Main Dashboard Tabs */}
      <Tabs defaultValue="events" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="events">Recent Events</TabsTrigger>
          <TabsTrigger value="threats">Threat Map</TabsTrigger>
          <TabsTrigger value="performance">Performance</TabsTrigger>
          <TabsTrigger value="alerts">Active Alerts</TabsTrigger>
        </TabsList>

        {/* Recent Security Events */}
        <TabsContent value="events" className="space-y-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>Recent Security Events</CardTitle>
                <p className="text-sm text-muted-foreground">
                  Latest security events and incidents detected across the platform
                </p>
              </div>
              <Link href="/security/incidents">
                <Button variant="outline" size="sm">
                  View All <ChevronRight className="w-4 h-4 ml-1" />
                </Button>
              </Link>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {recentEvents.map((event) => {
                  const EventIcon = getEventIcon(event.type);
                  
                  return (
                    <div key={event.id} className="flex items-start gap-4 p-4 border rounded-lg hover:bg-muted/50 transition-colors">
                      <div className={cn(
                        "p-2 rounded-full",
                        getSeverityColor(event.severity) + '/10'
                      )}>
                        <EventIcon className={cn("w-4 h-4", getSeverityColor(event.severity).replace('bg-', 'text-'))} />
                      </div>
                      
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <Badge variant={getSeverityBadgeVariant(event.severity) as any} className="text-xs">
                            {event.severity}
                          </Badge>
                          <span className="text-xs text-muted-foreground">
                            {event.timestamp.toLocaleTimeString()}
                          </span>
                          {event.location && (
                            <div className="flex items-center gap-1 text-xs text-muted-foreground">
                              <Globe className="w-3 h-3" />
                              {event.location}
                            </div>
                          )}
                        </div>
                        <p className="font-medium text-sm">{event.description}</p>
                        <p className="text-xs text-muted-foreground">Source: {event.source}</p>
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

        {/* Threat Map Placeholder */}
        <TabsContent value="threats" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Global Threat Map</CardTitle>
              <p className="text-sm text-muted-foreground">
                Geographic visualization of security threats and attack origins
              </p>
            </CardHeader>
            <CardContent>
              <div className="h-96 bg-muted/20 rounded-lg flex items-center justify-center">
                <div className="text-center">
                  <Globe className="w-16 h-16 mx-auto mb-4 text-muted-foreground" />
                  <p className="text-muted-foreground">Interactive threat map will be displayed here</p>
                  <Link href="/security/threats/map">
                    <Button className="mt-4">
                      View Full Threat Map
                    </Button>
                  </Link>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Performance Metrics */}
        <TabsContent value="performance" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>System Performance</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Security Engine Response Time</span>
                    <span className="font-medium">12ms</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Threat Detection Accuracy</span>
                    <span className="font-medium text-green-500">99.7%</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">False Positive Rate</span>
                    <span className="font-medium">0.3%</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm">Average Incident Response</span>
                    <span className="font-medium">45s</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Risk Assessment</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="text-center">
                    <div className={cn(
                      "text-4xl font-bold",
                      getRiskScoreColor(threatSummary.riskScore)
                    )}>
                      {threatSummary.riskScore}/100
                    </div>
                    <p className="text-sm text-muted-foreground">Overall Risk Score</p>
                  </div>
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span>Network Security</span>
                      <span className="text-green-500">Good</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span>Access Control</span>
                      <span className="text-green-500">Excellent</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span>Threat Detection</span>
                      <span className="text-yellow-500">Warning</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span>Incident Response</span>
                      <span className="text-green-500">Good</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Active Alerts */}
        <TabsContent value="alerts" className="space-y-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>Active Alerts</CardTitle>
                <p className="text-sm text-muted-foreground">
                  Security alerts requiring immediate attention
                </p>
              </div>
              <Link href="/security/alerts">
                <Button variant="outline" size="sm">
                  Manage Alerts <ChevronRight className="w-4 h-4 ml-1" />
                </Button>
              </Link>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {threatSummary.activeThreats > 0 ? (
                  Array.from({ length: Math.min(5, threatSummary.activeThreats) }).map((_, index) => (
                    <div key={index} className="flex items-center gap-3 p-3 border rounded-lg bg-red-500/5">
                      <AlertTriangle className="w-5 h-5 text-red-500" />
                      <div className="flex-1">
                        <p className="font-medium text-sm">Critical Security Alert #{index + 1}</p>
                        <p className="text-xs text-muted-foreground">
                          Suspicious activity detected - requires immediate review
                        </p>
                      </div>
                      <Button size="sm" variant="outline">Acknowledge</Button>
                    </div>
                  ))
                ) : (
                  <div className="text-center py-8">
                    <Shield className="w-12 h-12 mx-auto mb-2 text-green-500" />
                    <p className="text-sm text-muted-foreground">No active alerts</p>
                    <p className="text-xs text-muted-foreground">All security systems are operating normally</p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Quick Action Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Link href="/security/threats">
          <Card className="hover:shadow-md transition-all cursor-pointer">
            <CardContent className="flex items-center gap-4 p-6">
              <div className="p-3 rounded-full bg-red-500/10">
                <Target className="w-6 h-6 text-red-500" />
              </div>
              <div>
                <h3 className="font-semibold">Threat Intelligence</h3>
                <p className="text-sm text-muted-foreground">Monitor threats and attack patterns</p>
              </div>
            </CardContent>
          </Card>
        </Link>

        <Link href="/security/access">
          <Card className="hover:shadow-md transition-all cursor-pointer">
            <CardContent className="flex items-center gap-4 p-6">
              <div className="p-3 rounded-full bg-blue-500/10">
                <Lock className="w-6 h-6 text-blue-500" />
              </div>
              <div>
                <h3 className="font-semibold">Access Control</h3>
                <p className="text-sm text-muted-foreground">Manage permissions and access logs</p>
              </div>
            </CardContent>
          </Card>
        </Link>

        <Link href="/security/incidents">
          <Card className="hover:shadow-md transition-all cursor-pointer">
            <CardContent className="flex items-center gap-4 p-6">
              <div className="p-3 rounded-full bg-orange-500/10">
                <Zap className="w-6 h-6 text-orange-500" />
              </div>
              <div>
                <h3 className="font-semibold">Incident Response</h3>
                <p className="text-sm text-muted-foreground">Handle security incidents</p>
              </div>
            </CardContent>
          </Card>
        </Link>
      </div>
    </div>
  );
}