'use client';

import { useEffect, useState, useCallback, useRef } from 'react';
import { RefreshCw, Wifi, WifiOff, AlertCircle } from 'lucide-react';
import { useAuth } from '@/lib/auth';
import { usePermissionExpiry } from '@/hooks/usePermissionExpiry';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';
import { useToast } from '@/components/ui/use-toast';

export interface AutoPermissionRefreshProps {
  /**
   * Refresh interval in milliseconds (default: 5 minutes)
   */
  interval?: number;
  
  /**
   * Enable automatic refresh (default: true)
   */
  enabled?: boolean;
  
  /**
   * Refresh when permissions are near expiry (default: true)
   */
  refreshOnExpiry?: boolean;
  
  /**
   * Show refresh status indicator (default: true)
   */
  showStatus?: boolean;
  
  /**
   * Show detailed status card (default: false)
   */
  showDetailedStatus?: boolean;
  
  /**
   * Callback when refresh occurs
   */
  onRefresh?: (success: boolean, error?: string) => void;
  
  /**
   * Callback when permissions expire
   */
  onPermissionExpired?: (permissions: string[]) => void;
  
  /**
   * Callback when permissions are expiring soon
   */
  onPermissionExpiring?: (permissions: string[], minutesUntilExpiry: number) => void;
  
  /**
   * Custom CSS classes
   */
  className?: string;
}

const DEFAULT_PROPS = {
  interval: 5 * 60 * 1000, // 5 minutes
  enabled: true,
  refreshOnExpiry: true,
  showStatus: true,
  showDetailedStatus: false,
};

export function AutoPermissionRefresh(props: AutoPermissionRefreshProps) {
  const config = { ...DEFAULT_PROPS, ...props };
  
  const { user, refreshUser, isLoading: authLoading } = useAuth();
  const expiry = usePermissionExpiry();
  const { toast } = useToast();
  
  const [lastRefresh, setLastRefresh] = useState<Date | null>(null);
  const [nextRefresh, setNextRefresh] = useState<Date | null>(null);
  const [refreshCount, setRefreshCount] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [refreshError, setRefreshError] = useState<string | null>(null);
  const [isOnline, setIsOnline] = useState(typeof navigator !== 'undefined' ? navigator.onLine : true);
  const [refreshProgress, setRefreshProgress] = useState(0);
  
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const mountedRef = useRef(true);

  // Track online/offline status
  useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);
    
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  // Calculate next refresh time
  const calculateNextRefresh = useCallback(() => {
    if (!config.enabled) return null;
    return new Date(Date.now() + config.interval);
  }, [config.enabled, config.interval]);

  // Update progress bar
  const updateProgress = useCallback(() => {
    if (!nextRefresh) {
      setRefreshProgress(0);
      return;
    }
    
    const now = Date.now();
    const nextRefreshTime = nextRefresh.getTime();
    const totalDuration = config.interval;
    const elapsed = totalDuration - (nextRefreshTime - now);
    const progress = Math.max(0, Math.min(100, (elapsed / totalDuration) * 100));
    
    setRefreshProgress(progress);
  }, [nextRefresh, config.interval]);

  // Perform permission refresh
  const performRefresh = useCallback(async (reason: 'scheduled' | 'manual' | 'expiry' | 'online' = 'scheduled') => {
    if (isRefreshing || !user) return;
    
    setIsRefreshing(true);
    setRefreshError(null);
    
    try {
      // Refresh user data and permissions
      await Promise.all([
        refreshUser(),
        expiry.refresh()
      ]);
      
      const now = new Date();
      setLastRefresh(now);
      setRefreshCount(prev => prev + 1);
      setNextRefresh(calculateNextRefresh());
      
      config.onRefresh?.(true);
      
      // Only show toast for manual refreshes to avoid spam
      if (reason === 'manual') {
        toast({
          title: 'Permissions Refreshed',
          description: 'Your permissions have been updated successfully.',
        });
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to refresh permissions';
      setRefreshError(errorMessage);
      config.onRefresh?.(false, errorMessage);
      
      // Show error toast
      toast({
        title: 'Refresh Failed',
        description: errorMessage,
        variant: 'destructive',
      });
      
    } finally {
      setIsRefreshing(false);
    }
  }, [isRefreshing, user, refreshUser, expiry, config, calculateNextRefresh]);

  // Check for expired/expiring permissions
  useEffect(() => {
    if (!user || !config.refreshOnExpiry) return;
    
    // Check for expired permissions
    if (expiry.hasExpired && expiry.expiryInfo.expired.length > 0) {
      const expiredPermissions = expiry.expiryInfo.expired.map(p => p.basePermission);
      config.onPermissionExpired?.(expiredPermissions);
      
      // Auto-refresh to get updated permissions
      performRefresh('expiry');
    }
    
    // Check for expiring permissions
    if (expiry.hasExpiringSoon && expiry.expiryInfo.expiringSoon.length > 0) {
      const expiringPermissions = expiry.expiryInfo.expiringSoon.map(p => p.basePermission);
      const minutesUntilExpiry = Math.min(
        ...expiry.expiryInfo.expiringSoon
          .filter(p => p.timeRemaining)
          .map(p => Math.floor(p.timeRemaining! / (1000 * 60)))
      );
      
      config.onPermissionExpiring?.(expiringPermissions, minutesUntilExpiry);
    }
  }, [user, config, expiry.hasExpired, expiry.hasExpiringSoon, expiry.expiryInfo, performRefresh]);

  // Setup automatic refresh interval
  useEffect(() => {
    if (!config.enabled || !user || !isOnline) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }
    
    // Initial setup
    if (!nextRefresh) {
      setNextRefresh(calculateNextRefresh());
    }
    
    // Setup refresh interval
    intervalRef.current = setInterval(() => {
      if (mountedRef.current && isOnline) {
        performRefresh('scheduled');
      }
    }, config.interval);
    
    // Setup progress update interval
    if (progressIntervalRef.current) {
      clearInterval(progressIntervalRef.current);
    }
    progressIntervalRef.current = setInterval(updateProgress, 1000);
    updateProgress();
    
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
        progressIntervalRef.current = null;
      }
    };
  }, [config.enabled, config.interval, user, isOnline, nextRefresh, calculateNextRefresh, performRefresh, updateProgress]);

  // Refresh when coming back online
  useEffect(() => {
    if (isOnline && user && lastRefresh) {
      const timeSinceLastRefresh = Date.now() - lastRefresh.getTime();
      const shouldRefresh = timeSinceLastRefresh > config.interval;
      
      if (shouldRefresh) {
        performRefresh('online');
      }
    }
  }, [isOnline, user, lastRefresh, config.interval, performRefresh]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
    };
  }, []);

  // Don't render anything if status display is disabled
  if (!config.showStatus && !config.showDetailedStatus) {
    return null;
  }

  // Detailed status card
  if (config.showDetailedStatus) {
    return (
      <Card className={cn('w-full max-w-md', config.className)}>
        <CardContent className="p-4">
          <div className="space-y-4">
            {/* Header */}
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
                <span className="font-medium">Permission Sync</span>
              </div>
              
              <div className="flex items-center space-x-2">
                {isOnline ? (
                  <Wifi className="h-4 w-4 text-green-500" />
                ) : (
                  <WifiOff className="h-4 w-4 text-red-500" />
                )}
                
                <Badge variant={config.enabled ? 'default' : 'secondary'}>
                  {config.enabled ? 'Auto' : 'Manual'}
                </Badge>
              </div>
            </div>
            
            {/* Status */}
            <div className="space-y-2">
              {refreshError ? (
                <div className="flex items-center space-x-2 text-red-600">
                  <AlertCircle className="h-4 w-4" />
                  <span className="text-sm">{refreshError}</span>
                </div>
              ) : (
                <div className="text-sm text-gray-600">
                  {isRefreshing ? (
                    'Refreshing permissions...'
                  ) : lastRefresh ? (
                    `Last updated: ${lastRefresh.toLocaleTimeString()}`
                  ) : (
                    'No recent updates'
                  )}
                </div>
              )}
              
              {/* Progress bar */}
              {config.enabled && nextRefresh && !isRefreshing && (
                <div className="space-y-1">
                  <Progress value={refreshProgress} className="h-1" />
                  <div className="text-xs text-gray-500">
                    Next refresh: {nextRefresh.toLocaleTimeString()}
                  </div>
                </div>
              )}
            </div>
            
            {/* Stats */}
            <div className="flex items-center justify-between text-sm text-gray-500">
              <span>Refreshes: {refreshCount}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => performRefresh('manual')}
                disabled={isRefreshing || !isOnline}
                className="h-6 px-2"
              >
                <RefreshCw className={cn('h-3 w-3 mr-1', isRefreshing && 'animate-spin')} />
                Refresh
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  // Simple status indicator
  return (
    <div className={cn('flex items-center space-x-2', config.className)}>
      {/* Connection status */}
      {isOnline ? (
        <Wifi className="h-4 w-4 text-green-500" />
      ) : (
        <WifiOff className="h-4 w-4 text-red-500" />
      )}
      
      {/* Refresh button */}
      <Button
        variant="ghost"
        size="sm"
        onClick={() => performRefresh('manual')}
        disabled={isRefreshing || !isOnline}
        className="h-6 px-2"
        title={
          isRefreshing ? 'Refreshing...' : 
          !isOnline ? 'Offline' :
          lastRefresh ? `Last updated: ${lastRefresh.toLocaleTimeString()}` :
          'Refresh permissions'
        }
      >
        <RefreshCw className={cn('h-3 w-3', isRefreshing && 'animate-spin')} />
      </Button>
      
      {/* Status badge */}
      {refreshError && (
        <Badge variant="destructive" className="text-xs">
          Error
        </Badge>
      )}
      
      {config.enabled && (
        <Badge variant="secondary" className="text-xs">
          Auto
        </Badge>
      )}
    </div>
  );
}