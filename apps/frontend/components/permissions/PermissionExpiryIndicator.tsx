'use client';

import React, { useState, useEffect } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogHeader, 
  DialogTitle, 
  DialogTrigger 
} from '@/components/ui/dialog';
import { 
  Clock, 
  ShieldAlert, 
  ShieldCheck, 
  AlertTriangle,
  RefreshCw,
  Eye,
  Upgrade
} from 'lucide-react';
import { useGranularPermissions } from '@/hooks/useGranularPermissions';
import { formatDistanceToNow, format } from 'date-fns';

interface PermissionExpiryIndicatorProps {
  permission: string;
  className?: string;
  showCountdown?: boolean;
  showDetailedInfo?: boolean;
  variant?: 'badge' | 'inline' | 'full';
  onExpired?: () => void;
}

export function PermissionExpiryIndicator({ 
  permission,
  className = '',
  showCountdown = true,
  showDetailedInfo = false,
  variant = 'badge',
  onExpired
}: PermissionExpiryIndicatorProps) {
  const { 
    hasPermission, 
    getPermissionExpiry, 
    refreshPermissions,
    loading 
  } = useGranularPermissions();

  const [currentTime, setCurrentTime] = useState(Date.now());
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // Update current time every minute for live countdown
  useEffect(() => {
    if (!showCountdown) return;

    const interval = setInterval(() => {
      setCurrentTime(Date.now());
    }, 60000); // Update every minute

    return () => clearInterval(interval);
  }, [showCountdown]);

  // Check if user has the permission
  const hasAccess = hasPermission(permission);
  const expiryInfo = getPermissionExpiry(permission);

  // Handle expiration callback
  useEffect(() => {
    if (expiryInfo?.is_expired && onExpired) {
      onExpired();
    }
  }, [expiryInfo?.is_expired, onExpired]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshPermissions();
    } catch (err) {
      console.error('Failed to refresh permissions:', err);
    } finally {
      setIsRefreshing(false);
    }
  };

  // If no access or no expiry info, don't show anything
  if (!hasAccess || !expiryInfo) {
    return null;
  }

  // Calculate time remaining
  const getTimeRemaining = (): { 
    text: string; 
    urgent: boolean; 
    expired: boolean;
    exactTime?: string;
  } => {
    if (!expiryInfo.expires_at) {
      return { text: 'Permanent', urgent: false, expired: false };
    }

    const expiryTime = new Date(expiryInfo.expires_at * 1000);
    const now = new Date(currentTime);

    if (expiryTime <= now) {
      return { 
        text: `Expired ${formatDistanceToNow(expiryTime, { addSuffix: true })}`, 
        urgent: false, 
        expired: true,
        exactTime: format(expiryTime, 'PPpp')
      };
    }

    const msRemaining = expiryTime.getTime() - now.getTime();
    const hoursRemaining = msRemaining / (1000 * 60 * 60);
    
    // Determine urgency (less than 24 hours)
    const urgent = hoursRemaining < 24;

    return {
      text: `Expires ${formatDistanceToNow(expiryTime, { addSuffix: true })}`,
      urgent,
      expired: false,
      exactTime: format(expiryTime, 'PPpp')
    };
  };

  const timeInfo = getTimeRemaining();

  // Get appropriate styling based on status
  const getVariantStyles = () => {
    if (timeInfo.expired) {
      return {
        badgeVariant: 'destructive' as const,
        badgeClass: 'bg-red-100 text-red-800 border-red-300',
        icon: <ShieldAlert className="h-3 w-3" />,
        textClass: 'text-red-600'
      };
    }

    if (timeInfo.urgent) {
      return {
        badgeVariant: 'destructive' as const,
        badgeClass: 'bg-yellow-100 text-yellow-800 border-yellow-300',
        icon: <Clock className="h-3 w-3" />,
        textClass: 'text-yellow-600'
      };
    }

    return {
      badgeVariant: 'default' as const,
      badgeClass: 'bg-green-100 text-green-800 border-green-300',
      icon: <ShieldCheck className="h-3 w-3" />,
      textClass: 'text-green-600'
    };
  };

  const styles = getVariantStyles();

  // Badge variant (compact)
  if (variant === 'badge') {
    return (
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogTrigger asChild>
          <Badge 
            variant={styles.badgeVariant}
            className={`cursor-pointer hover:opacity-80 ${styles.badgeClass} ${className}`}
          >
            <div className="flex items-center gap-1">
              {styles.icon}
              {showCountdown ? timeInfo.text : (timeInfo.expired ? 'Expired' : 'Valid')}
            </div>
          </Badge>
        </DialogTrigger>
        
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              {styles.icon}
              Permission Expiry Details
            </DialogTitle>
            <DialogDescription>
              Information about the "{permission}" permission
            </DialogDescription>
          </DialogHeader>
          
          <PermissionDetailView 
            permission={permission}
            expiryInfo={expiryInfo}
            timeInfo={timeInfo}
            onRefresh={handleRefresh}
            isRefreshing={isRefreshing}
          />
        </DialogContent>
      </Dialog>
    );
  }

  // Inline variant (text with icon)
  if (variant === 'inline') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        {styles.icon}
        <span className={`text-sm ${styles.textClass}`}>
          {showCountdown ? timeInfo.text : (timeInfo.expired ? 'Expired' : 'Valid')}
        </span>
        {showDetailedInfo && (
          <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="ghost" size="sm" className="h-6 px-2">
                <Eye className="h-3 w-3" />
              </Button>
            </DialogTrigger>
            
            <DialogContent>
              <DialogHeader>
                <DialogTitle className="flex items-center gap-2">
                  {styles.icon}
                  Permission Expiry Details
                </DialogTitle>
                <DialogDescription>
                  Information about the "{permission}" permission
                </DialogDescription>
              </DialogHeader>
              
              <PermissionDetailView 
                permission={permission}
                expiryInfo={expiryInfo}
                timeInfo={timeInfo}
                onRefresh={handleRefresh}
                isRefreshing={isRefreshing}
              />
            </DialogContent>
          </Dialog>
        )}
      </div>
    );
  }

  // Full variant (alert-style display)
  if (variant === 'full') {
    return (
      <Alert 
        className={`${className} ${
          timeInfo.expired ? 'border-red-300 bg-red-50' :
          timeInfo.urgent ? 'border-yellow-300 bg-yellow-50' :
          'border-green-300 bg-green-50'
        }`}
      >
        {styles.icon}
        <AlertDescription>
          <div className="flex items-center justify-between">
            <div>
              <div className={`font-medium ${styles.textClass}`}>
                Permission: {permission}
              </div>
              <div className={`text-sm ${styles.textClass}`}>
                {timeInfo.text}
                {timeInfo.exactTime && (
                  <span className="ml-2 text-muted-foreground">
                    ({timeInfo.exactTime})
                  </span>
                )}
              </div>
            </div>
            <div className="flex gap-2">
              {showDetailedInfo && (
                <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                  <DialogTrigger asChild>
                    <Button variant="outline" size="sm">
                      <Eye className="h-3 w-3" />
                      Details
                    </Button>
                  </DialogTrigger>
                  
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle className="flex items-center gap-2">
                        {styles.icon}
                        Permission Expiry Details
                      </DialogTitle>
                      <DialogDescription>
                        Information about the "{permission}" permission
                      </DialogDescription>
                    </DialogHeader>
                    
                    <PermissionDetailView 
                      permission={permission}
                      expiryInfo={expiryInfo}
                      timeInfo={timeInfo}
                      onRefresh={handleRefresh}
                      isRefreshing={isRefreshing}
                    />
                  </DialogContent>
                </Dialog>
              )}
              
              <Button 
                variant="outline" 
                size="sm"
                onClick={handleRefresh}
                disabled={isRefreshing}
              >
                <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            </div>
          </div>
        </AlertDescription>
      </Alert>
    );
  }

  return null;
}

// Detail view component for permission information
function PermissionDetailView({
  permission,
  expiryInfo,
  timeInfo,
  onRefresh,
  isRefreshing
}: {
  permission: string;
  expiryInfo: any; // Would be properly typed in real implementation
  timeInfo: { text: string; urgent: boolean; expired: boolean; exactTime?: string };
  onRefresh: () => void;
  isRefreshing: boolean;
}) {
  const getSourceBadge = (source: string) => {
    const colors: Record<string, string> = {
      Subscription: 'bg-green-100 text-green-800',
      Trial: 'bg-purple-100 text-purple-800',
      Admin: 'bg-blue-100 text-blue-800',
      System: 'bg-indigo-100 text-indigo-800',
      Legacy: 'bg-gray-100 text-gray-800'
    };
    
    return (
      <Badge variant="outline" className={colors[source] || 'bg-gray-100 text-gray-800'}>
        {source}
      </Badge>
    );
  };

  return (
    <div className="space-y-4">
      {/* Permission Info */}
      <div className="space-y-2">
        <div>
          <span className="text-sm font-medium">Permission:</span>
          <code className="ml-2 text-sm bg-muted px-2 py-1 rounded">
            {permission}
          </code>
        </div>
        
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">Source:</span>
          {getSourceBadge(expiryInfo?.claim?.source || 'Unknown')}
        </div>
        
        {expiryInfo?.claim?.granted_at && (
          <div>
            <span className="text-sm font-medium">Granted:</span>
            <span className="ml-2 text-sm text-muted-foreground">
              {formatDistanceToNow(new Date(expiryInfo.claim.granted_at * 1000), { addSuffix: true })}
            </span>
          </div>
        )}
      </div>

      {/* Expiry Status */}
      <div className="p-3 border rounded-lg">
        <div className="flex items-center justify-between mb-2">
          <span className="font-medium">Expiry Status</span>
          <Badge 
            variant={timeInfo.expired ? 'destructive' : timeInfo.urgent ? 'destructive' : 'default'}
            className={
              timeInfo.expired ? 'bg-red-100 text-red-800' :
              timeInfo.urgent ? 'bg-yellow-100 text-yellow-800' :
              'bg-green-100 text-green-800'
            }
          >
            {timeInfo.expired ? 'Expired' : timeInfo.urgent ? 'Expiring Soon' : 'Active'}
          </Badge>
        </div>
        
        <div className="text-sm space-y-1">
          <div>{timeInfo.text}</div>
          {timeInfo.exactTime && (
            <div className="text-muted-foreground">
              Exact time: {timeInfo.exactTime}
            </div>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="flex gap-2">
        <Button 
          variant="outline" 
          onClick={onRefresh}
          disabled={isRefreshing}
          className="flex-1"
        >
          <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh Status
        </Button>
        
        {(timeInfo.expired || timeInfo.urgent) && (
          <Button 
            onClick={() => window.open('/upgrade', '_blank')}
            className="flex-1"
          >
            <Upgrade className="h-4 w-4" />
            Extend Access
          </Button>
        )}
      </div>

      {/* Warning Messages */}
      {timeInfo.expired && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            This permission has expired. Some features may not be available until renewed.
          </AlertDescription>
        </Alert>
      )}
      
      {timeInfo.urgent && !timeInfo.expired && (
        <Alert>
          <Clock className="h-4 w-4" />
          <AlertDescription>
            This permission will expire soon. Consider renewing to maintain access.
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}

export default PermissionExpiryIndicator;