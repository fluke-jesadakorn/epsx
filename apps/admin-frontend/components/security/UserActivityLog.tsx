'use client';

import { useState, useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
  Activity,
  TrendingUp,
  TrendingDown,
  Clock,
  Globe,
  Users,
  AlertTriangle,
  Eye,
  Filter,
  Calendar
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

interface UserActivityPattern {
  userId: string;
  userEmail: string;
  totalEvents: number;
  successfulLogins: number;
  failedLogins: number;
  apiCalls: number;
  permissionChecks: number;
  avgRiskScore: number;
  locations: string[];
  devices: string[];
  peakHours: number[];
  anomalies: string[];
}

interface UserActivityLogProps {
  events: AccessEvent[];
}

export function UserActivityLog({ events }: UserActivityLogProps) {
  const [selectedTimeframe, setSelectedTimeframe] = useState<'1h' | '6h' | '24h' | '7d'>('24h');
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [showAnomaliesOnly, setShowAnomaliesOnly] = useState(false);

  // Analyze user activity patterns
  const userPatterns = useMemo(() => {
    const patterns = new Map<string, UserActivityPattern>();
    
    events.forEach(event => {
      const key = event.userId;
      
      if (!patterns.has(key)) {
        patterns.set(key, {
          userId: event.userId,
          userEmail: event.userEmail,
          totalEvents: 0,
          successfulLogins: 0,
          failedLogins: 0,
          apiCalls: 0,
          permissionChecks: 0,
          avgRiskScore: 0,
          locations: [],
          devices: [],
          peakHours: [],
          anomalies: []
        });
      }
      
      const pattern = patterns.get(key)!;
      pattern.totalEvents++;
      
      // Count event types
      switch (event.action) {
        case 'login':
          if (event.result === 'granted') pattern.successfulLogins++;
          break;
        case 'failed_login':
          pattern.failedLogins++;
          break;
        case 'api_access':
          pattern.apiCalls++;
          break;
        case 'permission_check':
          pattern.permissionChecks++;
          break;
      }
      
      // Track locations and devices
      if (event.location && !pattern.locations.includes(event.location)) {
        pattern.locations.push(event.location);
      }
      
      const device = event.userAgent.split(' ')[0]; // Simplified device detection
      if (!pattern.devices.includes(device)) {
        pattern.devices.push(device);
      }
      
      // Track peak hours
      const hour = event.timestamp.getHours();
      pattern.peakHours.push(hour);
      
      // Calculate average risk score
      pattern.avgRiskScore = ((pattern.avgRiskScore * (pattern.totalEvents - 1)) + event.riskScore) / pattern.totalEvents;
    });
    
    // Detect anomalies
    patterns.forEach(pattern => {
      // High failure rate
      if (pattern.failedLogins > 0 && (pattern.failedLogins / pattern.totalEvents) > 0.3) {
        pattern.anomalies.push('High failure rate');
      }
      
      // Multiple locations
      if (pattern.locations.length > 3) {
        pattern.anomalies.push('Multiple locations');
      }
      
      // High risk score
      if (pattern.avgRiskScore > 50) {
        pattern.anomalies.push('High risk activity');
      }
      
      // Unusual hours (activity outside 6 AM - 10 PM)
      const unusualHours = pattern.peakHours.filter(hour => hour < 6 || hour > 22).length;
      if (unusualHours > pattern.totalEvents * 0.3) {
        pattern.anomalies.push('Off-hours activity');
      }
    });
    
    return Array.from(patterns.values()).sort((a, b) => b.totalEvents - a.totalEvents);
  }, [events]);

  // Filter patterns based on settings
  const filteredPatterns = userPatterns.filter(pattern => {
    if (showAnomaliesOnly && pattern.anomalies.length === 0) return false;
    return true;
  });

  // Get hourly activity for selected user
  const getHourlyActivity = (userId: string) => {
    const userEvents = events.filter(e => e.userId === userId);
    const hourlyData = Array.from({ length: 24 }, (_, hour) => ({
      hour,
      count: userEvents.filter(e => e.timestamp.getHours() === hour).length
    }));
    return hourlyData;
  };

  const getAnomalyColor = (anomalyCount: number) => {
    if (anomalyCount >= 3) return 'text-red-500 bg-red-500/10';
    if (anomalyCount >= 2) return 'text-orange-500 bg-orange-500/10';
    if (anomalyCount >= 1) return 'text-yellow-500 bg-yellow-500/10';
    return 'text-green-500 bg-green-500/10';
  };

  const getRiskColor = (score: number) => {
    if (score >= 70) return 'text-red-500';
    if (score >= 40) return 'text-orange-500';
    if (score >= 20) return 'text-yellow-500';
    return 'text-green-500';
  };

  const formatSuccessRate = (successful: number, total: number) => {
    if (total === 0) return '0%';
    return `${Math.round((successful / total) * 100)}%`;
  };

  return (
    <div className="space-y-6">
      {/* Controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex rounded-lg border p-1">
            {(['1h', '6h', '24h', '7d'] as const).map((timeframe) => (
              <Button
                key={timeframe}
                size="sm"
                variant={selectedTimeframe === timeframe ? 'default' : 'ghost'}
                onClick={() => setSelectedTimeframe(timeframe)}
                className="h-8 px-3"
              >
                {timeframe}
              </Button>
            ))}
          </div>
          
          <Button
            size="sm"
            variant={showAnomaliesOnly ? 'default' : 'outline'}
            onClick={() => setShowAnomaliesOnly(!showAnomaliesOnly)}
          >
            <Filter className="w-4 h-4 mr-2" />
            {showAnomaliesOnly ? 'Show All' : 'Anomalies Only'}
          </Button>
        </div>
        
        <Badge variant="outline" className="text-sm">
          {filteredPatterns.length} users analyzed
        </Badge>
      </div>

      {/* User Activity Patterns */}
      <div className="grid grid-cols-1 gap-4">
        {filteredPatterns.map((pattern) => (
          <Card 
            key={pattern.userId}
            className={cn(
              "cursor-pointer transition-all hover:shadow-md",
              selectedUser === pattern.userId && "ring-2 ring-primary",
              pattern.anomalies.length > 0 && "border-orange-200 dark:border-orange-800"
            )}
            onClick={() => setSelectedUser(selectedUser === pattern.userId ? null : pattern.userId)}
          >
            <CardContent className="p-4">
              <div className="flex items-center gap-4">
                <div className={cn("p-2 rounded-full", getAnomalyColor(pattern.anomalies.length))}>
                  <Users className="w-5 h-5" />
                </div>
                
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="font-medium">{pattern.userEmail}</span>
                    <Badge variant="outline" className="text-xs">
                      {pattern.totalEvents} events
                    </Badge>
                    {pattern.anomalies.length > 0 && (
                      <Badge 
                        className={cn("text-xs", getAnomalyColor(pattern.anomalies.length))}
                      >
                        {pattern.anomalies.length} anomal{pattern.anomalies.length === 1 ? 'y' : 'ies'}
                      </Badge>
                    )}
                    <Badge 
                      variant="outline" 
                      className={cn("text-xs", getRiskColor(pattern.avgRiskScore))}
                    >
                      {Math.round(pattern.avgRiskScore)} risk
                    </Badge>
                  </div>
                  
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <div className="text-muted-foreground text-xs">Success Rate</div>
                      <div className="font-medium text-green-500">
                        {formatSuccessRate(pattern.successfulLogins, pattern.successfulLogins + pattern.failedLogins)}
                      </div>
                    </div>
                    <div>
                      <div className="text-muted-foreground text-xs">API Calls</div>
                      <div className="font-medium">{pattern.apiCalls}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground text-xs">Locations</div>
                      <div className="font-medium">{pattern.locations.length}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground text-xs">Devices</div>
                      <div className="font-medium">{pattern.devices.length}</div>
                    </div>
                  </div>
                  
                  {pattern.anomalies.length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {pattern.anomalies.map((anomaly, index) => (
                        <Badge key={index} variant="destructive" className="text-xs">
                          <AlertTriangle className="w-3 h-3 mr-1" />
                          {anomaly}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
                
                <Button variant="ghost" size="sm">
                  <Eye className="w-4 h-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Selected User Details */}
      {selectedUser && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="w-5 h-5" />
              User Activity Details
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {/* 24-Hour Activity Pattern */}
              <div>
                <h4 className="font-semibold mb-3">24-Hour Activity Pattern</h4>
                <div className="relative">
                  <svg width="100%" height="80" className="overflow-visible">
                    {/* Background grid */}
                    <g stroke="hsl(var(--border))" strokeOpacity="0.2">
                      <line x1="0" y1="20" x2="100%" y2="20" />
                      <line x1="0" y1="40" x2="100%" y2="40" />
                      <line x1="0" y1="60" x2="100%" y2="60" />
                    </g>

                    {/* Activity bars */}
                    {getHourlyActivity(selectedUser).map((data, index) => {
                      const x = (index / 24) * 100;
                      const maxActivity = Math.max(...getHourlyActivity(selectedUser).map(d => d.count));
                      const height = maxActivity > 0 ? (data.count / maxActivity) * 60 : 0;
                      
                      return (
                        <rect
                          key={data.hour}
                          x={`${x}%`}
                          y={80 - height}
                          width={`${100/24 * 0.8}%`}
                          height={height}
                          fill="hsl(var(--primary))"
                          opacity="0.7"
                          className="hover:opacity-100 transition-all"
                        />
                      );
                    })}
                  </svg>
                  
                  {/* Hour labels */}
                  <div className="flex justify-between text-xs text-muted-foreground mt-2">
                    <span>00:00</span>
                    <span>06:00</span>
                    <span>12:00</span>
                    <span>18:00</span>
                    <span>23:59</span>
                  </div>
                </div>
              </div>

              {/* Recent Events */}
              <div>
                <h4 className="font-semibold mb-3">Recent Events</h4>
                <div className="space-y-2">
                  {events
                    .filter(e => e.userId === selectedUser)
                    .slice(0, 5)
                    .map((event) => (
                      <div key={event.id} className="flex items-center gap-3 p-2 border rounded text-sm">
                        <Badge 
                          variant={event.result === 'granted' ? 'default' : 'destructive'}
                          className="text-xs"
                        >
                          {event.action}
                        </Badge>
                        <span className="flex-1 font-mono text-xs">{event.resource}</span>
                        <div className="flex items-center gap-1 text-muted-foreground text-xs">
                          <Clock className="w-3 h-3" />
                          {event.timestamp.toLocaleTimeString()}
                        </div>
                        <div className="flex items-center gap-1 text-muted-foreground text-xs">
                          <Globe className="w-3 h-3" />
                          {event.location || 'Unknown'}
                        </div>
                      </div>
                    ))}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold text-green-500">
              {userPatterns.filter(p => p.anomalies.length === 0).length}
            </div>
            <div className="text-sm text-muted-foreground">Normal Users</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold text-orange-500">
              {userPatterns.filter(p => p.anomalies.length > 0 && p.anomalies.length < 3).length}
            </div>
            <div className="text-sm text-muted-foreground">Users with Anomalies</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4 text-center">
            <div className="text-2xl font-bold text-red-500">
              {userPatterns.filter(p => p.anomalies.length >= 3).length}
            </div>
            <div className="text-sm text-muted-foreground">High Risk Users</div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}