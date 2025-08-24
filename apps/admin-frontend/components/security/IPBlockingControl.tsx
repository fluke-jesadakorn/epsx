'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';
import {
  Shield,
  Ban,
  CheckCircle,
  XCircle,
  Plus,
  Search,
  Filter,
  Download,
  Upload,
  Clock,
  Globe,
  AlertTriangle,
  Eye,
  Trash2,
  RefreshCw,
  Target
} from 'lucide-react';
import { toast } from 'react-hot-toast';

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
  reputation: number;
}

interface BlockRule {
  id: string;
  ip: string;
  type: 'single' | 'range' | 'cidr';
  reason: string;
  blockedAt: Date;
  expiresAt?: Date;
  permanent: boolean;
  createdBy: string;
}

interface IPBlockingControlProps {
  threats: ThreatData[];
  onBlock: (ip: string) => Promise<void>;
  onUnblock: (ip: string) => Promise<void>;
}

export function IPBlockingControl({ threats, onBlock, onUnblock }: IPBlockingControlProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState<'all' | 'blocked' | 'active'>('all');
  const [newBlockIP, setNewBlockIP] = useState('');
  const [newBlockReason, setNewBlockReason] = useState('');
  const [newBlockType, setNewBlockType] = useState<'single' | 'range' | 'cidr'>('single');
  const [newBlockPermanent, setNewBlockPermanent] = useState(false);
  const [newBlockDuration, setNewBlockDuration] = useState('24');
  const [isLoading, setIsLoading] = useState(false);
  const [blockRules, setBlockRules] = useState<BlockRule[]>([]);
  const [bulkIPs, setBulkIPs] = useState('');
  const [autoBlockEnabled, setAutoBlockEnabled] = useState(true);
  const [autoBlockThreshold, setAutoBlockThreshold] = useState(50);

  // Initialize block rules from threats
  useEffect(() => {
    const rules = threats
      .filter(t => t.blocked)
      .map(t => ({
        id: t.id,
        ip: t.ip,
        type: 'single' as const,
        reason: `Auto-blocked: ${t.threatType} attacks (${t.attackCount} attempts)`,
        blockedAt: t.lastSeen,
        permanent: false,
        createdBy: 'System'
      }));
    
    setBlockRules(rules);
  }, [threats]);

  // Filter threats based on search and status
  const filteredThreats = threats.filter(threat => {
    const matchesSearch = threat.ip.includes(searchQuery) || 
                         threat.country.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         threat.city.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = filterStatus === 'all' || 
                         (filterStatus === 'blocked' && threat.blocked) ||
                         (filterStatus === 'active' && !threat.blocked);
    
    return matchesSearch && matchesStatus;
  });

  // Validate IP address format
  const validateIP = (ip: string) => {
    const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
    const cidrRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\/(?:3[0-2]|[0-2]?[0-9])$/;
    const rangeRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\s*-\s*(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;

    switch (newBlockType) {
      case 'single': return ipRegex.test(ip);
      case 'cidr': return cidrRegex.test(ip);
      case 'range': return rangeRegex.test(ip);
      default: return false;
    }
  };

  // Handle single IP block
  const handleBlockIP = async (ip: string, reason?: string) => {
    setIsLoading(true);
    try {
      await onBlock(ip);
      
      const newRule: BlockRule = {
        id: Date.now().toString(),
        ip,
        type: 'single',
        reason: reason || 'Manual block',
        blockedAt: new Date(),
        expiresAt: newBlockPermanent ? undefined : new Date(Date.now() + parseInt(newBlockDuration) * 60 * 60 * 1000),
        permanent: newBlockPermanent,
        createdBy: 'Admin'
      };
      
      setBlockRules(prev => [...prev, newRule]);
      toast.success(`IP ${ip} has been blocked`);
      setNewBlockIP('');
      setNewBlockReason('');
    } catch (error) {
      toast.error(`Failed to block IP ${ip}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Handle IP unblock
  const handleUnblockIP = async (ip: string) => {
    setIsLoading(true);
    try {
      await onUnblock(ip);
      setBlockRules(prev => prev.filter(rule => rule.ip !== ip));
      toast.success(`IP ${ip} has been unblocked`);
    } catch (error) {
      toast.error(`Failed to unblock IP ${ip}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Handle bulk IP operations
  const handleBulkBlock = async () => {
    if (!bulkIPs.trim()) return;

    const ips = bulkIPs.split('\n').map(ip => ip.trim()).filter(ip => ip);
    const validIPs = ips.filter(validateIP);
    
    if (validIPs.length === 0) {
      toast.error('No valid IP addresses found');
      return;
    }

    setIsLoading(true);
    try {
      for (const ip of validIPs) {
        await handleBlockIP(ip, 'Bulk block operation');
      }
      toast.success(`Blocked ${validIPs.length} IP addresses`);
      setBulkIPs('');
    } catch (error) {
      toast.error('Bulk block operation failed');
    } finally {
      setIsLoading(false);
    }
  };

  // Export block list
  const handleExport = () => {
    const exportData = blockRules.map(rule => ({
      ip: rule.ip,
      type: rule.type,
      reason: rule.reason,
      blocked_at: rule.blockedAt.toISOString(),
      expires_at: rule.expiresAt?.toISOString(),
      permanent: rule.permanent,
      created_by: rule.createdBy
    }));

    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `blocked-ips-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Handle file import
  const handleImport = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const importData = JSON.parse(e.target?.result as string);
        if (Array.isArray(importData)) {
          setBulkIPs(importData.map(item => item.ip).join('\n'));
          toast.success(`Loaded ${importData.length} IP addresses for import`);
        }
      } catch (error) {
        toast.error('Invalid JSON file format');
      }
    };
    reader.readAsText(file);
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

  const formatThreatType = (type: string) => {
    return type.split('_').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  const getReputationColor = (reputation: number) => {
    if (reputation < 25) return 'text-red-500';
    if (reputation < 50) return 'text-orange-500';
    if (reputation < 75) return 'text-yellow-500';
    return 'text-green-500';
  };

  return (
    <div className="space-y-6">
      {/* Controls Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">IP Address Management</h3>
          <p className="text-sm text-muted-foreground">
            Manage blocked IPs, reputation scores, and access control rules
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleExport}>
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
          <Label htmlFor="import-file" className="cursor-pointer">
            <Button variant="outline" size="sm" asChild>
              <span>
                <Upload className="w-4 h-4 mr-2" />
                Import
              </span>
            </Button>
          </Label>
          <input
            id="import-file"
            type="file"
            accept=".json"
            onChange={handleImport}
            className="hidden"
          />
        </div>
      </div>

      {/* Auto-blocking Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="w-5 h-5" />
            Auto-blocking Settings
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <Label>Enable Auto-blocking</Label>
              <p className="text-sm text-muted-foreground">
                Automatically block IPs after reaching attack threshold
              </p>
            </div>
            <Switch 
              checked={autoBlockEnabled} 
              onCheckedChange={setAutoBlockEnabled}
            />
          </div>
          
          {autoBlockEnabled && (
            <div className="mt-4 space-y-3">
              <div>
                <Label>Attack Threshold</Label>
                <Select value={autoBlockThreshold.toString()} onValueChange={(value) => setAutoBlockThreshold(parseInt(value))}>
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="10">10 attacks</SelectItem>
                    <SelectItem value="25">25 attacks</SelectItem>
                    <SelectItem value="50">50 attacks</SelectItem>
                    <SelectItem value="100">100 attacks</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Main Tabs */}
      <Tabs defaultValue="active-threats" className="space-y-4">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="active-threats">Active Threats</TabsTrigger>
          <TabsTrigger value="block-rules">Block Rules</TabsTrigger>
          <TabsTrigger value="manual-control">Manual Control</TabsTrigger>
        </TabsList>

        {/* Active Threats */}
        <TabsContent value="active-threats" className="space-y-4">
          {/* Filters */}
          <div className="flex flex-col sm:flex-row gap-4">
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
            <Select value={filterStatus} onValueChange={setFilterStatus as any}>
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

          {/* Threats List */}
          <div className="space-y-3">
            {filteredThreats.map((threat) => (
              <Card key={threat.id}>
                <CardContent className="p-4">
                  <div className="flex items-center gap-4">
                    <div className={cn("p-2 rounded-full", getSeverityColor(threat.severity))}>
                      <Target className="w-4 h-4" />
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
                          <Globe className="w-3 h-3" />
                          {threat.city}, {threat.country}
                        </div>
                        <div className="flex items-center gap-1">
                          <AlertTriangle className="w-3 h-3" />
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
                    
                    <div className="flex gap-2">
                      {threat.blocked ? (
                        <Button 
                          size="sm" 
                          variant="outline"
                          onClick={() => handleUnblockIP(threat.ip)}
                          disabled={isLoading}
                        >
                          <CheckCircle className="w-4 h-4 mr-1" />
                          Unblock
                        </Button>
                      ) : (
                        <Button 
                          size="sm" 
                          variant="destructive"
                          onClick={() => handleBlockIP(threat.ip)}
                          disabled={isLoading}
                        >
                          <Ban className="w-4 h-4 mr-1" />
                          Block
                        </Button>
                      )}
                      <Button size="sm" variant="ghost">
                        <Eye className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* Block Rules */}
        <TabsContent value="block-rules" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Active Block Rules</CardTitle>
              <p className="text-sm text-muted-foreground">
                Current IP blocking rules and their status
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {blockRules.map((rule) => (
                  <div key={rule.id} className="flex items-center gap-4 p-3 border rounded-lg">
                    <div className="p-2 rounded-full bg-red-500/10">
                      <Ban className="w-4 h-4 text-red-500" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-mono text-sm font-medium">{rule.ip}</span>
                        <Badge variant="outline" className="text-xs">
                          {rule.type}
                        </Badge>
                        {rule.permanent ? (
                          <Badge variant="secondary" className="text-xs">
                            Permanent
                          </Badge>
                        ) : rule.expiresAt ? (
                          <Badge variant="outline" className="text-xs">
                            Expires: {rule.expiresAt.toLocaleString()}
                          </Badge>
                        ) : null}
                      </div>
                      <div className="text-xs text-muted-foreground mb-1">
                        {rule.reason}
                      </div>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <span>Blocked: {rule.blockedAt.toLocaleString()}</span>
                        <span>By: {rule.createdBy}</span>
                      </div>
                    </div>
                    
                    <div className="flex gap-1">
                      <Button 
                        size="sm" 
                        variant="outline"
                        onClick={() => handleUnblockIP(rule.ip)}
                        disabled={isLoading}
                      >
                        <CheckCircle className="w-4 h-4" />
                      </Button>
                      <Button 
                        size="sm" 
                        variant="ghost"
                        onClick={() => setBlockRules(prev => prev.filter(r => r.id !== rule.id))}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Manual Control */}
        <TabsContent value="manual-control" className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Single IP Block */}
            <Card>
              <CardHeader>
                <CardTitle>Block Single IP</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>IP Address</Label>
                  <Input
                    placeholder="192.168.1.100"
                    value={newBlockIP}
                    onChange={(e) => setNewBlockIP(e.target.value)}
                  />
                </div>
                
                <div>
                  <Label>Block Type</Label>
                  <Select value={newBlockType} onValueChange={setNewBlockType as any}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="single">Single IP</SelectItem>
                      <SelectItem value="range">IP Range</SelectItem>
                      <SelectItem value="cidr">CIDR Block</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div>
                  <Label>Reason</Label>
                  <Input
                    placeholder="Manual security block"
                    value={newBlockReason}
                    onChange={(e) => setNewBlockReason(e.target.value)}
                  />
                </div>
                
                <div className="flex items-center space-x-2">
                  <Switch 
                    id="permanent"
                    checked={newBlockPermanent}
                    onCheckedChange={setNewBlockPermanent}
                  />
                  <Label htmlFor="permanent">Permanent block</Label>
                </div>
                
                {!newBlockPermanent && (
                  <div>
                    <Label>Duration (hours)</Label>
                    <Select value={newBlockDuration} onValueChange={setNewBlockDuration}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="1">1 hour</SelectItem>
                        <SelectItem value="6">6 hours</SelectItem>
                        <SelectItem value="24">24 hours</SelectItem>
                        <SelectItem value="168">7 days</SelectItem>
                        <SelectItem value="720">30 days</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                )}
                
                <Button 
                  onClick={() => handleBlockIP(newBlockIP, newBlockReason)}
                  disabled={!newBlockIP || !validateIP(newBlockIP) || isLoading}
                  className="w-full"
                >
                  {isLoading ? (
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <Ban className="w-4 h-4 mr-2" />
                  )}
                  Block IP
                </Button>
              </CardContent>
            </Card>

            {/* Bulk Operations */}
            <Card>
              <CardHeader>
                <CardTitle>Bulk Operations</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>IP Addresses (one per line)</Label>
                  <Textarea
                    placeholder="192.168.1.100&#10;203.0.113.0&#10;198.51.100.0"
                    value={bulkIPs}
                    onChange={(e) => setBulkIPs(e.target.value)}
                    rows={6}
                  />
                </div>
                
                <div className="text-sm text-muted-foreground">
                  {bulkIPs ? `${bulkIPs.split('\n').filter(ip => ip.trim()).length} IP addresses` : 'No IPs entered'}
                </div>
                
                <Button 
                  onClick={handleBulkBlock}
                  disabled={!bulkIPs.trim() || isLoading}
                  className="w-full"
                  variant="destructive"
                >
                  {isLoading ? (
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <Plus className="w-4 h-4 mr-2" />
                  )}
                  Bulk Block IPs
                </Button>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}