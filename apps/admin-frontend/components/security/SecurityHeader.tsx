'use client';

import { useState, useEffect } from 'react';
import { useAuth } from '@/lib/auth';
import { cn } from '@/lib/utils';
import { 
  Menu, 
  Bell, 
  Settings, 
  RefreshCw, 
  Download, 
  Shield,
  AlertTriangle,
  Clock,
  Users,
  Activity,
  Search
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';

interface SecurityHeaderProps {
  onSidebarToggle: () => void;
  criticalAlerts: number;
  systemStatus: 'normal' | 'warning' | 'critical';
}

interface SecurityMetric {
  label: string;
  value: string;
  trend: 'up' | 'down' | 'stable';
  critical?: boolean;
}

export function SecurityHeader({ 
  onSidebarToggle, 
  criticalAlerts, 
  systemStatus 
}: SecurityHeaderProps) {
  const { user } = useAuth();
  const [lastUpdate, setLastUpdate] = useState(new Date());
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeIncidents, setActiveIncidents] = useState(2);
  const [blockedIPs, setBlockedIPs] = useState(156);
  const [activeSessions, setActiveSessions] = useState(1247);

  // Real-time metrics simulation
  useEffect(() => {
    const interval = setInterval(() => {
      setLastUpdate(new Date());
      setActiveIncidents(Math.floor(Math.random() * 5));
      setBlockedIPs(prev => prev + Math.floor(Math.random() * 3) - 1);
      setActiveSessions(prev => Math.max(1000, prev + Math.floor(Math.random() * 100) - 50));
    }, 15000); // Update every 15 seconds

    return () => clearInterval(interval);
  }, []);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    // Simulate API refresh
    await new Promise(resolve => setTimeout(resolve, 1000));
    setLastUpdate(new Date());
    setIsRefreshing(false);
  };

  const handleExport = () => {
    // Export security data
    console.log('Exporting security data...');
  };

  const metrics: SecurityMetric[] = [
    {
      label: 'Critical Alerts',
      value: criticalAlerts.toString(),
      trend: criticalAlerts > 2 ? 'up' : 'stable',
      critical: criticalAlerts > 0
    },
    {
      label: 'Active Incidents',
      value: activeIncidents.toString(),
      trend: activeIncidents > 1 ? 'up' : 'down'
    },
    {
      label: 'Blocked IPs',
      value: blockedIPs.toLocaleString(),
      trend: 'up'
    },
    {
      label: 'Active Sessions',
      value: activeSessions.toLocaleString(),
      trend: 'stable'
    }
  ];

  const getStatusColor = () => {
    switch (systemStatus) {
      case 'critical': return 'text-red-500 bg-red-500/10 border-red-500/20';
      case 'warning': return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/20';
      default: return 'text-green-500 bg-green-500/10 border-green-500/20';
    }
  };

  const getTrendIcon = (trend: 'up' | 'down' | 'stable') => {
    switch (trend) {
      case 'up': return '↗';
      case 'down': return '↘';
      default: return '→';
    }
  };

  const getTrendColor = (trend: 'up' | 'down' | 'stable', critical?: boolean) => {
    if (critical && trend === 'up') return 'text-red-500';
    switch (trend) {
      case 'up': return 'text-green-500';
      case 'down': return 'text-red-500';
      default: return 'text-muted-foreground';
    }
  };

  return (
    <header className="bg-card border-b border-border px-6 py-4">
      <div className="flex items-center justify-between">
        {/* Left Section */}
        <div className="flex items-center gap-4">
          {/* Mobile Menu Toggle */}
          <Button
            variant="ghost"
            size="sm"
            onClick={onSidebarToggle}
            className="md:hidden"
          >
            <Menu className="w-4 h-4" />
          </Button>

          {/* System Status */}
          <div className={cn(
            "flex items-center gap-2 px-3 py-1.5 rounded-full border text-sm font-medium",
            getStatusColor(),
            systemStatus === 'critical' && 'animate-pulse'
          )}>
            <Shield className="w-4 h-4" />
            <span className="capitalize">{systemStatus}</span>
          </div>

          {/* Search */}
          <div className="hidden md:flex relative">
            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search security events..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9 w-64"
            />
          </div>
        </div>

        {/* Center Section - Quick Metrics */}
        <div className="hidden lg:flex items-center gap-6">
          {metrics.map((metric) => (
            <div key={metric.label} className="text-center">
              <div className="flex items-center gap-1 justify-center">
                <span className={cn(
                  "text-lg font-bold",
                  metric.critical ? 'text-red-500' : 'text-foreground'
                )}>
                  {metric.value}
                </span>
                <span className={cn(
                  "text-xs",
                  getTrendColor(metric.trend, metric.critical)
                )}>
                  {getTrendIcon(metric.trend)}
                </span>
              </div>
              <div className="text-xs text-muted-foreground">{metric.label}</div>
            </div>
          ))}
        </div>

        {/* Right Section */}
        <div className="flex items-center gap-2">
          {/* Notifications */}
          <Button variant="ghost" size="sm" className="relative">
            <Bell className="w-4 h-4" />
            {criticalAlerts > 0 && (
              <Badge 
                variant="destructive" 
                className="absolute -top-1 -right-1 w-5 h-5 text-xs p-0 flex items-center justify-center"
              >
                {criticalAlerts}
              </Badge>
            )}
          </Button>

          {/* Refresh */}
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={handleRefresh}
            disabled={isRefreshing}
          >
            <RefreshCw className={cn(
              "w-4 h-4",
              isRefreshing && "animate-spin"
            )} />
          </Button>

          {/* Export */}
          <Button variant="ghost" size="sm" onClick={handleExport}>
            <Download className="w-4 h-4" />
          </Button>

          {/* Settings */}
          <Button variant="ghost" size="sm">
            <Settings className="w-4 h-4" />
          </Button>

          {/* User Info */}
          <div className="hidden md:flex items-center gap-2 ml-4 pl-4 border-l border-border">
            <div className="text-right">
              <div className="text-sm font-medium">{user?.email}</div>
              <div className="text-xs text-muted-foreground">
                Security Admin
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Secondary Header with Last Update */}
      <div className="flex items-center justify-between mt-4 pt-4 border-t border-border/50">
        <div className="flex items-center gap-4 text-sm text-muted-foreground">
          <div className="flex items-center gap-1">
            <Clock className="w-4 h-4" />
            <span>Last updated: {lastUpdate.toLocaleTimeString()}</span>
          </div>
          <div className="flex items-center gap-1">
            <Users className="w-4 h-4" />
            <span>{activeSessions.toLocaleString()} active sessions</span>
          </div>
          {criticalAlerts > 0 && (
            <div className="flex items-center gap-1 text-red-500">
              <AlertTriangle className="w-4 h-4" />
              <span>{criticalAlerts} critical alert{criticalAlerts !== 1 ? 's' : ''}</span>
            </div>
          )}
        </div>

        {/* Mobile Metrics */}
        <div className="flex lg:hidden items-center gap-4">
          {metrics.slice(0, 2).map((metric) => (
            <div key={metric.label} className="text-center">
              <div className="flex items-center gap-1 justify-center">
                <span className={cn(
                  "text-sm font-bold",
                  metric.critical ? 'text-red-500' : 'text-foreground'
                )}>
                  {metric.value}
                </span>
                <span className={cn(
                  "text-xs",
                  getTrendColor(metric.trend, metric.critical)
                )}>
                  {getTrendIcon(metric.trend)}
                </span>
              </div>
              <div className="text-xs text-muted-foreground">{metric.label}</div>
            </div>
          ))}
        </div>
      </div>
    </header>
  );
}