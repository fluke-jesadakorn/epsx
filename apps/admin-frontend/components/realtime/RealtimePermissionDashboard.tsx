'use client';

import { useEffect, useState, useCallback, useRef } from 'react';
import { 
  Activity, 
  Users, 
  AlertTriangle, 
  CheckCircle, 
  Clock, 
  XCircle, 
  Timer,
  Pause,
  Play,
  RotateCcw,
  Filter,
  Download
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';

// Types for real-time admin events
export interface AdminPermissionEvent {
  id: string;
  type: 'permission_granted' | 'permission_revoked' | 'permission_expired' | 'permission_expiring_soon' | 'permission_extended' | 'bulk_operation';
  userId: string;
  userName: string;
  userEmail: string;
  permission: string;
  basePermission: string;
  platform: string;
  resource: string;
  action: string;
  expiryTimestamp?: number;
  previousExpiry?: number;
  newExpiry?: number;
  reason?: string;
  grantedBy?: string;
  operatorId?: string;
  operatorName?: string;
  timestamp: number;
  batchId?: string;
  batchSize?: number;
  metadata?: Record<string, any>;
}

export interface RealtimeDashboardStats {
  totalEvents: number;
  activeUsers: number;
  recentGrants: number;
  recentRevokes: number;
  expiredPermissions: number;
  expiringPermissions: number;
  lastUpdateTime: number;
}

export interface RealtimePermissionDashboardProps {
  className?: string;
  maxEvents?: number;
  autoRefreshInterval?: number;
  enableFiltering?: boolean;
  enableExport?: boolean;
}

const DEFAULT_PROPS = {
  maxEvents: 100,
  autoRefreshInterval: 30000, // 30 seconds
  enableFiltering: true,
  enableExport: true,
};

const EVENT_ICONS = {
  permission_granted: CheckCircle,
  permission_revoked: XCircle,
  permission_expired: AlertTriangle,
  permission_expiring_soon: Timer,
  permission_extended: Clock,
  bulk_operation: Users,
};

const EVENT_COLORS = {
  permission_granted: 'text-green-600 bg-green-50',
  permission_revoked: 'text-gray-600 bg-gray-50',
  permission_expired: 'text-red-600 bg-red-50',
  permission_expiring_soon: 'text-yellow-600 bg-yellow-50',
  permission_extended: 'text-blue-600 bg-blue-50',
  bulk_operation: 'text-purple-600 bg-purple-50',
};

const EVENT_LABELS = {
  permission_granted: 'Granted',
  permission_revoked: 'Revoked',
  permission_expired: 'Expired',
  permission_expiring_soon: 'Expiring',
  permission_extended: 'Extended',
  bulk_operation: 'Bulk Op',
};

export function RealtimePermissionDashboard({
  className = '',
  ...props
}: RealtimePermissionDashboardProps) {
  const config = { ...DEFAULT_PROPS, ...props };
  
  const [events, setEvents] = useState<AdminPermissionEvent[]>([]);
  const [stats, setStats] = useState<RealtimeDashboardStats>({
    totalEvents: 0,
    activeUsers: 0,
    recentGrants: 0,
    recentRevokes: 0,
    expiredPermissions: 0,
    expiringPermissions: 0,
    lastUpdateTime: Date.now(),
  });
  
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [filters, setFilters] = useState({
    platform: 'all',
    eventType: 'all',
    user: '',
  });
  
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const mountedRef = useRef(true);

  // Connect to admin WebSocket
  const connectWebSocket = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.CONNECTING || wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const wsUrl = `${process.env.NEXT_PUBLIC_WS_URL || 'wss://api.epsx.io'}/ws/admin/permissions`;
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        if (!mountedRef.current) return;
        console.log('Admin permission WebSocket connected');
        setIsConnected(true);
        
        // Request initial stats
        ws.send(JSON.stringify({ type: 'get_stats' }));
      };

      ws.onmessage = (event) => {
        if (!mountedRef.current || isPaused) return;
        
        try {
          const data = JSON.parse(event.data);
          
          if (data.type === 'permission_event') {
            const permissionEvent: AdminPermissionEvent = data.event;
            addEvent(permissionEvent);
          } else if (data.type === 'stats_update') {
            setStats(data.stats);
          }
        } catch (error) {
          console.error('Error parsing WebSocket message:', error);
        }
      };

      ws.onclose = () => {
        if (!mountedRef.current) return;
        console.log('Admin permission WebSocket closed');
        setIsConnected(false);
        
        // Attempt to reconnect after 3 seconds
        reconnectTimeoutRef.current = setTimeout(() => {
          if (mountedRef.current) {
            connectWebSocket();
          }
        }, 3000);
      };

      ws.onerror = (error) => {
        console.error('Admin permission WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to create admin WebSocket connection:', error);
    }
  }, [isPaused]);

  // Add new event with deduplication and limits
  const addEvent = useCallback((event: AdminPermissionEvent) => {
    setEvents(prev => {
      // Check for duplicate events
      const isDuplicate = prev.some(e => 
        e.id === event.id || 
        (e.userId === event.userId && 
         e.basePermission === event.basePermission && 
         e.type === event.type &&
         Math.abs(e.timestamp - event.timestamp) < 1000) // Within 1 second
      );
      
      if (isDuplicate) return prev;
      
      // Add new event and limit total count
      return [event, ...prev].slice(0, config.maxEvents);
    });
  }, [config.maxEvents]);

  // Filter events based on current filters
  const filteredEvents = events.filter(event => {
    if (filters.platform !== 'all' && event.platform !== filters.platform) return false;
    if (filters.eventType !== 'all' && event.type !== filters.eventType) return false;
    if (filters.user && !event.userName.toLowerCase().includes(filters.user.toLowerCase()) && 
        !event.userEmail.toLowerCase().includes(filters.user.toLowerCase())) return false;
    return true;
  });

  // Format timestamp for display
  const formatTimestamp = useCallback((timestamp: number) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMins / 60);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return date.toLocaleDateString();
  }, []);

  // Format permission for display
  const formatPermission = useCallback((basePermission: string) => {
    const parts = basePermission.split(':');
    if (parts.length >= 3) {
      return `${parts[1]?.toUpperCase()} ${parts[2]?.toLowerCase()}`;
    }
    return basePermission;
  }, []);

  // Export events to CSV
  const exportEvents = useCallback(() => {
    const csvContent = [
      'Timestamp,Type,User Name,User Email,Permission,Platform,Reason,Operator',
      ...filteredEvents.map(event => [
        new Date(event.timestamp).toISOString(),
        event.type,
        event.userName,
        event.userEmail,
        event.basePermission,
        event.platform,
        event.reason || '',
        event.operatorName || ''
      ].map(field => `"${field}"`).join(','))
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `permission-events-${new Date().toISOString().split('T')[0]}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  }, [filteredEvents]);

  // Clear all events
  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  // Connect on mount
  useEffect(() => {
    connectWebSocket();
    
    return () => {
      mountedRef.current = false;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connectWebSocket]);

  return (
    <div className={cn('space-y-6', className)}>
      {/* Stats Cards */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Events</p>
                <p className="text-2xl font-bold">{stats.totalEvents}</p>
              </div>
              <Activity className="h-8 w-8 text-gray-400" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Users</p>
                <p className="text-2xl font-bold">{stats.activeUsers}</p>
              </div>
              <Users className="h-8 w-8 text-gray-400" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-green-600">Granted</p>
                <p className="text-2xl font-bold text-green-600">{stats.recentGrants}</p>
              </div>
              <CheckCircle className="h-8 w-8 text-green-400" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Revoked</p>
                <p className="text-2xl font-bold">{stats.recentRevokes}</p>
              </div>
              <XCircle className="h-8 w-8 text-gray-400" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-red-600">Expired</p>
                <p className="text-2xl font-bold text-red-600">{stats.expiredPermissions}</p>
              </div>
              <AlertTriangle className="h-8 w-8 text-red-400" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-yellow-600">Expiring</p>
                <p className="text-2xl font-bold text-yellow-600">{stats.expiringPermissions}</p>
              </div>
              <Timer className="h-8 w-8 text-yellow-400" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Controls and Filters */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center space-x-2">
              <Activity className="h-5 w-5" />
              <span>Real-time Permission Events</span>
              <Badge variant={isConnected ? 'default' : 'destructive'}>
                {isConnected ? 'Connected' : 'Disconnected'}
              </Badge>
            </CardTitle>
            
            <div className="flex items-center space-x-2">
              {/* Pause/Resume */}
              <Button
                variant="outline"
                size="sm"
                onClick={() => setIsPaused(!isPaused)}
                className="flex items-center space-x-1"
              >
                {isPaused ? <Play className="h-4 w-4" /> : <Pause className="h-4 w-4" />}
                <span>{isPaused ? 'Resume' : 'Pause'}</span>
              </Button>
              
              {/* Clear Events */}
              <Button
                variant="outline"
                size="sm"
                onClick={clearEvents}
                className="flex items-center space-x-1"
              >
                <RotateCcw className="h-4 w-4" />
                <span>Clear</span>
              </Button>
              
              {/* Export */}
              {config.enableExport && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={exportEvents}
                  className="flex items-center space-x-1"
                >
                  <Download className="h-4 w-4" />
                  <span>Export</span>
                </Button>
              )}
            </div>
          </div>
          
          {/* Filters */}
          {config.enableFiltering && (
            <div className="flex items-center space-x-4 pt-4">
              <div className="flex items-center space-x-2">
                <Filter className="h-4 w-4 text-gray-500" />
                <span className="text-sm text-gray-500">Filters:</span>
              </div>
              
              <Select
                value={filters.platform}
                onValueChange={(value) => setFilters(prev => ({ ...prev, platform: value }))}
              >
                <SelectTrigger className="w-32">
                  <SelectValue placeholder="Platform" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Platforms</SelectItem>
                  <SelectItem value="epsx">EPSX</SelectItem>
                  <SelectItem value="admin">Admin</SelectItem>
                  <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                  <SelectItem value="epsx-token">EPSX Token</SelectItem>
                </SelectContent>
              </Select>
              
              <Select
                value={filters.eventType}
                onValueChange={(value) => setFilters(prev => ({ ...prev, eventType: value }))}
              >
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="Event Type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Events</SelectItem>
                  <SelectItem value="permission_granted">Granted</SelectItem>
                  <SelectItem value="permission_revoked">Revoked</SelectItem>
                  <SelectItem value="permission_expired">Expired</SelectItem>
                  <SelectItem value="permission_expiring_soon">Expiring</SelectItem>
                  <SelectItem value="permission_extended">Extended</SelectItem>
                  <SelectItem value="bulk_operation">Bulk Operations</SelectItem>
                </SelectContent>
              </Select>
              
              <Input
                placeholder="Search users..."
                value={filters.user}
                onChange={(e) => setFilters(prev => ({ ...prev, user: e.target.value }))}
                className="w-48"
              />
            </div>
          )}
        </CardHeader>
        
        <CardContent>
          {/* Events List */}
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {filteredEvents.length === 0 ? (
              <div className="text-center py-8 text-gray-500">
                <Activity className="h-12 w-12 mx-auto mb-4 text-gray-300" />
                <p>No permission events to display</p>
                {isPaused && (
                  <p className="text-sm mt-2">Events are paused. Click Resume to continue.</p>
                )}
              </div>
            ) : (
              filteredEvents.map((event) => {
                const Icon = EVENT_ICONS[event.type];
                return (
                  <div
                    key={event.id}
                    className="flex items-center space-x-3 p-3 border border-gray-100 rounded-lg hover:bg-gray-50"
                  >
                    <div className={cn(
                      'flex-shrink-0 p-2 rounded-full',
                      EVENT_COLORS[event.type]
                    )}>
                      <Icon className="h-4 w-4" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center space-x-2">
                        <Badge variant="outline" className="text-xs">
                          {EVENT_LABELS[event.type]}
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          {event.platform.toUpperCase()}
                        </Badge>
                        <span className="text-xs text-gray-500">
                          {formatTimestamp(event.timestamp)}
                        </span>
                      </div>
                      
                      <div className="mt-1">
                        <p className="text-sm font-medium text-gray-900">
                          {event.userName}
                        </p>
                        <p className="text-xs text-gray-500">
                          {formatPermission(event.basePermission)}
                          {event.reason && ` • ${event.reason}`}
                        </p>
                      </div>
                    </div>
                    
                    {event.batchSize && (
                      <Badge variant="outline" className="text-xs">
                        Batch: {event.batchSize}
                      </Badge>
                    )}
                  </div>
                );
              })
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}