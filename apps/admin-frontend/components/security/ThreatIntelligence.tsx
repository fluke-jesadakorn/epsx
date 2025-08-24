'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { ThreatMap } from './ThreatMap';
import { SecurityMetricsChart } from './SecurityMetricsChart';
import { AttackTimeline } from './AttackTimeline';
import { IPBlockingControl } from './IPBlockingControl';
import { cn } from '@/lib/utils';
import {
  Target,
  Globe,
  Shield,
  AlertTriangle,
  Search,
  Filter,
  Download,
  RefreshCw,
  MapPin,
  Zap,
  Activity,
  Lock,
  Eye,
  Ban,
  CheckCircle,
  XCircle,
  Clock,
  TrendingUp
} from 'lucide-react';

interface ThreatData {
  id: string;
  ip: string;
  country: string;
  city: string;
  latitude: number;
  longitude: number;
  threatType: 'brute_force' | 'sql_injection' | 'ddos' | 'malware' | 'phishing';
  severity: 'low' | 'medium' | 'high' | 'critical';
  firstSeen: Date;
  lastSeen: Date;
  attackCount: number;
  blocked: boolean;
  reputation: number; // 0-100, lower is worse
}

interface AttackPattern {
  type: string;
  count: number;
  trend: 'increasing' | 'decreasing' | 'stable';
  peakHour: number;
  description: string;
}

export function ThreatIntelligence() {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedThreat, setSelectedThreat] = useState<ThreatData | null>(null);

  const [threatData, setThreatData] = useState<ThreatData[]>([
    {
      id: '1',
      ip: '192.168.1.100',
      country: 'United States',
      city: 'New York',
      latitude: 40.7128,
      longitude: -74.0060,
      threatType: 'brute_force',
      severity: 'high',
      firstSeen: new Date(Date.now() - 86400000),
      lastSeen: new Date(Date.now() - 300000),
      attackCount: 247,
      blocked: true,
      reputation: 15
    },
    {
      id: '2',
      ip: '203.0.113.0',
      country: 'China',
      city: 'Beijing',
      latitude: 39.9042,
      longitude: 116.4074,
      threatType: 'sql_injection',
      severity: 'critical',
      firstSeen: new Date(Date.now() - 172800000),
      lastSeen: new Date(Date.now() - 600000),
      attackCount: 89,
      blocked: true,
      reputation: 5
    },
    {
      id: '3',
      ip: '198.51.100.0',
      country: 'Russia',
      city: 'Moscow',
      latitude: 55.7558,
      longitude: 37.6176,
      threatType: 'ddos',
      severity: 'medium',
      firstSeen: new Date(Date.now() - 259200000),
      lastSeen: new Date(Date.now() - 1800000),
      attackCount: 156,
      blocked: false,
      reputation: 25
    },
    {
      id: '4',
      ip: '233.252.0.0',
      country: 'Brazil',
      city: 'São Paulo',
      latitude: -23.5505,
      longitude: -46.6333,
      threatType: 'malware',
      severity: 'high',
      firstSeen: new Date(Date.now() - 345600000),
      lastSeen: new Date(Date.now() - 900000),
      attackCount: 67,
      blocked: true,
      reputation: 8
    }
  ]);

  const [attackPatterns, setAttackPatterns] = useState<AttackPattern[]>([
    {
      type: 'Brute Force',
      count: 1247,
      trend: 'increasing',
      peakHour: 14,
      description: 'Login credential attacks'
    },
    {
      type: 'SQL Injection',
      count: 356,
      trend: 'decreasing',
      peakHour: 9,
      description: 'Database query attacks'
    },
    {
      type: 'DDoS',
      count: 89,
      trend: 'stable',
      peakHour: 22,
      description: 'Distributed denial of service'
    },
    {
      type: 'Malware',
      count: 234,
      trend: 'increasing',
      peakHour: 11,
      description: 'Malicious software distribution'
    }
  ]);

  // Filter threats based on search and filters
  const filteredThreats = threatData.filter(threat => {
    const matchesSearch = threat.ip.includes(searchQuery) || 
                         threat.country.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         threat.city.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesSeverity = filterSeverity === 'all' || threat.severity === filterSeverity;
    const matchesStatus = filterStatus === 'all' || 
                         (filterStatus === 'blocked' && threat.blocked) ||
                         (filterStatus === 'active' && !threat.blocked);
    
    return matchesSearch && matchesSeverity && matchesStatus;
  });

  const handleRefresh = async () => {
    setIsRefreshing(true);
    // Simulate API refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    setIsRefreshing(false);
  };

  const handleBlockIP = async (ip: string) => {
    setThreatData(prev => prev.map(threat => 
      threat.ip === ip ? { ...threat, blocked: true } : threat
    ));
  };

  const handleUnblockIP = async (ip: string) => {
    setThreatData(prev => prev.map(threat => 
      threat.ip === ip ? { ...threat, blocked: false } : threat
    ));
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'text-red-500 bg-red-500/10';
      case 'high': return 'text-orange-500 bg-orange-500/10';
      case 'medium': return 'text-yellow-500 bg-yellow-500/10';
      case 'low': return 'text-blue-500 bg-blue-500/10';
      default: return 'text-gray-500 bg-gray-500/10';
    }
  };

  const getThreatTypeIcon = (type: string) => {
    switch (type) {
      case 'brute_force': return Lock;
      case 'sql_injection': return Shield;
      case 'ddos': return Zap;
      case 'malware': return AlertTriangle;
      default: return Target;
    }
  };

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'increasing': return TrendingUp;
      case 'decreasing': return TrendingUp;
      case 'stable': return Activity;
      default: return Activity;
    }
  };

  const getTrendColor = (trend: string) => {
    switch (trend) {
      case 'increasing': return 'text-red-500';
      case 'decreasing': return 'text-green-500';
      case 'stable': return 'text-yellow-500';
      default: return 'text-gray-500';
    }
  };

  const getReputationColor = (reputation: number) => {
    if (reputation < 25) return 'text-red-500';
    if (reputation < 50) return 'text-orange-500';
    if (reputation < 75) return 'text-yellow-500';
    return 'text-green-500';
  };

  const formatThreatType = (type: string) => {
    return type.split('_').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Threat Intelligence</h1>
          <p className="text-muted-foreground mt-1">
            Monitor threats, analyze attack patterns, and manage IP reputation
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
              <div className="p-2 rounded-full bg-red-500/10">
                <Target className="w-5 h-5 text-red-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">{threatData.filter(t => t.blocked).length}</p>
                <p className="text-sm text-muted-foreground">Blocked IPs</p>
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
                <p className="text-2xl font-bold">{threatData.filter(t => t.severity === 'critical' || t.severity === 'high').length}</p>
                <p className="text-sm text-muted-foreground">High Threats</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-blue-500/10">
                <Globe className="w-5 h-5 text-blue-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">{new Set(threatData.map(t => t.country)).size}</p>
                <p className="text-sm text-muted-foreground">Countries</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center gap-4">
              <div className="p-2 rounded-full bg-green-500/10">
                <Activity className="w-5 h-5 text-green-500" />
              </div>
              <div>
                <p className="text-2xl font-bold">{threatData.reduce((sum, t) => sum + t.attackCount, 0).toLocaleString()}</p>
                <p className="text-sm text-muted-foreground">Total Attacks</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs defaultValue="map" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="map">Threat Map</TabsTrigger>
          <TabsTrigger value="intelligence">Intelligence</TabsTrigger>
          <TabsTrigger value="patterns">Attack Patterns</TabsTrigger>
          <TabsTrigger value="blocking">IP Management</TabsTrigger>
        </TabsList>

        {/* Threat Map */}
        <TabsContent value="map" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Globe className="w-5 h-5" />
                Global Threat Map
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                Geographic visualization of threats and their origins
              </p>
            </CardHeader>
            <CardContent>
              <ThreatMap threats={threatData} onThreatSelect={setSelectedThreat} />
            </CardContent>
          </Card>

          {selectedThreat && (
            <Card>
              <CardHeader>
                <CardTitle>Threat Details</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <h4 className="font-semibold mb-2">Basic Information</h4>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>IP Address:</span>
                        <span className="font-mono">{selectedThreat.ip}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Location:</span>
                        <span>{selectedThreat.city}, {selectedThreat.country}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Threat Type:</span>
                        <Badge className={getSeverityColor(selectedThreat.severity)}>
                          {formatThreatType(selectedThreat.threatType)}
                        </Badge>
                      </div>
                      <div className="flex justify-between">
                        <span>Status:</span>
                        {selectedThreat.blocked ? (
                          <Badge variant="destructive">Blocked</Badge>
                        ) : (
                          <Badge variant="outline">Active</Badge>
                        )}
                      </div>
                    </div>
                  </div>
                  
                  <div>
                    <h4 className="font-semibold mb-2">Attack Statistics</h4>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Attack Count:</span>
                        <span className="font-bold">{selectedThreat.attackCount}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Reputation Score:</span>
                        <span className={getReputationColor(selectedThreat.reputation)}>
                          {selectedThreat.reputation}/100
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span>First Seen:</span>
                        <span>{selectedThreat.firstSeen.toLocaleDateString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Last Seen:</span>
                        <span>{selectedThreat.lastSeen.toLocaleString()}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="mt-4 flex gap-2">
                  {selectedThreat.blocked ? (
                    <Button 
                      size="sm" 
                      variant="outline"
                      onClick={() => handleUnblockIP(selectedThreat.ip)}
                    >
                      <CheckCircle className="w-4 h-4 mr-2" />
                      Unblock IP
                    </Button>
                  ) : (
                    <Button 
                      size="sm" 
                      variant="destructive"
                      onClick={() => handleBlockIP(selectedThreat.ip)}
                    >
                      <Ban className="w-4 h-4 mr-2" />
                      Block IP
                    </Button>
                  )}
                  <Button size="sm" variant="outline">
                    <Eye className="w-4 h-4 mr-2" />
                    View Details
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* Intelligence Table */}
        <TabsContent value="intelligence" className="space-y-4">
          {/* Filters */}
          <Card>
            <CardHeader>
              <CardTitle>Threat Intelligence</CardTitle>
              <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <div className="flex-1">
                  <div className="relative">
                    <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      placeholder="Search by IP, country, or city..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-9"
                    />
                  </div>
                </div>
                <Select value={filterSeverity} onValueChange={setFilterSeverity}>
                  <SelectTrigger className="w-40">
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
                    <SelectItem value="blocked">Blocked</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {filteredThreats.map((threat) => {
                  const ThreatIcon = getThreatTypeIcon(threat.threatType);
                  
                  return (
                    <div 
                      key={threat.id} 
                      className="flex items-center gap-4 p-4 border rounded-lg hover:bg-muted/50 transition-colors cursor-pointer"
                      onClick={() => setSelectedThreat(threat)}
                    >
                      <div className={cn("p-2 rounded-full", getSeverityColor(threat.severity))}>
                        <ThreatIcon className="w-4 h-4" />
                      </div>
                      
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="font-mono text-sm font-medium">{threat.ip}</span>
                          <Badge variant="outline" className="text-xs">
                            {formatThreatType(threat.threatType)}
                          </Badge>
                          <Badge className={cn("text-xs", getSeverityColor(threat.severity))}>
                            {threat.severity}
                          </Badge>
                          {threat.blocked && (
                            <Badge variant="destructive" className="text-xs">
                              Blocked
                            </Badge>
                          )}
                        </div>
                        <div className="flex items-center gap-4 text-xs text-muted-foreground">
                          <div className="flex items-center gap-1">
                            <MapPin className="w-3 h-3" />
                            {threat.city}, {threat.country}
                          </div>
                          <div className="flex items-center gap-1">
                            <Zap className="w-3 h-3" />
                            {threat.attackCount} attacks
                          </div>
                          <div className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            Last: {threat.lastSeen.toLocaleString()}
                          </div>
                          <div className={cn("flex items-center gap-1", getReputationColor(threat.reputation))}>
                            <Target className="w-3 h-3" />
                            {threat.reputation}/100 reputation
                          </div>
                        </div>
                      </div>
                      
                      <div className="flex gap-1">
                        {threat.blocked ? (
                          <Button 
                            size="sm" 
                            variant="outline"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleUnblockIP(threat.ip);
                            }}
                          >
                            <CheckCircle className="w-4 h-4" />
                          </Button>
                        ) : (
                          <Button 
                            size="sm" 
                            variant="destructive"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleBlockIP(threat.ip);
                            }}
                          >
                            <Ban className="w-4 h-4" />
                          </Button>
                        )}
                        <Button size="sm" variant="ghost">
                          <Eye className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Attack Patterns */}
        <TabsContent value="patterns" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Attack Patterns</CardTitle>
                <p className="text-sm text-muted-foreground">
                  Analysis of attack types and trends over time
                </p>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {attackPatterns.map((pattern) => {
                    const TrendIcon = getTrendIcon(pattern.trend);
                    
                    return (
                      <div key={pattern.type} className="flex items-center gap-4 p-3 border rounded-lg">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="font-medium">{pattern.type}</span>
                            <Badge variant="outline" className="text-xs">
                              {pattern.count.toLocaleString()}
                            </Badge>
                          </div>
                          <p className="text-xs text-muted-foreground mb-2">{pattern.description}</p>
                          <div className="flex items-center gap-4 text-xs text-muted-foreground">
                            <span>Peak: {pattern.peakHour}:00</span>
                            <div className={cn("flex items-center gap-1", getTrendColor(pattern.trend))}>
                              <TrendIcon className="w-3 h-3" />
                              <span className="capitalize">{pattern.trend}</span>
                            </div>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Attack Timeline</CardTitle>
                <p className="text-sm text-muted-foreground">
                  24-hour attack pattern visualization
                </p>
              </CardHeader>
              <CardContent>
                <AttackTimeline patterns={attackPatterns} />
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Security Metrics</CardTitle>
              <p className="text-sm text-muted-foreground">
                Real-time security metrics and performance indicators
              </p>
            </CardHeader>
            <CardContent>
              <SecurityMetricsChart />
            </CardContent>
          </Card>
        </TabsContent>

        {/* IP Management */}
        <TabsContent value="blocking" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>IP Address Management</CardTitle>
              <p className="text-sm text-muted-foreground">
                Manage IP blocking, reputation scores, and access control
              </p>
            </CardHeader>
            <CardContent>
              <IPBlockingControl 
                threats={threatData}
                onBlock={handleBlockIP}
                onUnblock={handleUnblockIP}
              />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}