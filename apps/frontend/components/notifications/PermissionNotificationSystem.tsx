'use client';

import { useEffect, useState, useCallback } from 'react';
import { Bell, Clock, AlertTriangle, CheckCircle, XCircle, X, Timer } from 'lucide-react';
import { useRealtimePermissions, type PermissionUpdateEvent } from '@/hooks/useRealtimePermissions';
import { usePermissionExpiry } from '@/hooks/usePermissionExpiry';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

// Notification types and interfaces
export interface PermissionNotification {
  id: string;
  type: 'expiring_soon' | 'expired' | 'granted' | 'revoked' | 'extended';
  title: string;
  message: string;
  permission: string;
  basePermission: string;
  platform: string;
  timestamp: number;
  expiresAt?: number;
  isRead: boolean;
  priority: 'low' | 'medium' | 'high' | 'critical';
  metadata?: Record<string, any>;
}

export interface PermissionNotificationSystemProps {
  className?: string;
  maxNotifications?: number;
  autoHideDelay?: number;
  showUnreadCount?: boolean;
  enableSound?: boolean;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left';
  onNotificationClick?: (notification: PermissionNotification) => void;
}

const DEFAULT_PROPS = {
  maxNotifications: 5,
  autoHideDelay: 10000, // 10 seconds
  showUnreadCount: true,
  enableSound: false,
  position: 'top-right' as const,
};

const NOTIFICATION_ICONS = {
  expiring_soon: Timer,
  expired: AlertTriangle,
  granted: CheckCircle,
  revoked: XCircle,
  extended: Clock,
};

const NOTIFICATION_COLORS = {
  expiring_soon: 'text-yellow-600 bg-yellow-50 border-yellow-200',
  expired: 'text-red-600 bg-red-50 border-red-200',
  granted: 'text-green-600 bg-green-50 border-green-200',
  revoked: 'text-gray-600 bg-gray-50 border-gray-200',
  extended: 'text-blue-600 bg-blue-50 border-blue-200',
};

export function PermissionNotificationSystem({
  className = '',
  ...props
}: PermissionNotificationSystemProps) {
  const config = { ...DEFAULT_PROPS, ...props };
  
  const [notifications, setNotifications] = useState<PermissionNotification[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  
  const expiry = usePermissionExpiry();
  
  // Real-time permission monitoring
  const realtime = useRealtimePermissions({
    enabled: true,
    onPermissionUpdated: handlePermissionUpdate,
    onPermissionExpired: handlePermissionExpired,
    onPermissionExpiring: handlePermissionExpiring,
  });

  // Generate unique notification ID
  const generateNotificationId = useCallback(() => {
    return `perm-notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }, []);

  // Format permission for display
  const formatPermission = useCallback((basePermission: string): string => {
    const parts = basePermission.split(':');
    if (parts.length >= 3) {
      return `${parts[1]?.toUpperCase()} ${parts[2]?.toLowerCase()}`;
    }
    return basePermission;
  }, []);

  // Format time remaining
  const formatTimeRemaining = useCallback((timestamp: number): string => {
    const now = Date.now();
    const remaining = timestamp * 1000 - now;
    
    if (remaining <= 0) return 'Expired';
    
    const minutes = Math.floor(remaining / (1000 * 60));
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    
    if (days > 0) return `${days}d`;
    if (hours > 0) return `${hours}h`;
    if (minutes > 0) return `${minutes}m`;
    return '<1m';
  }, []);

  // Handle permission granted/revoked/extended
  function handlePermissionUpdate(event: PermissionUpdateEvent) {
    const notification: PermissionNotification = {
      id: generateNotificationId(),
      type: event.type === 'permission_granted' ? 'granted' : 
            event.type === 'permission_extended' ? 'extended' : 'revoked',
      title: event.type === 'permission_granted' ? 'Permission Granted' :
             event.type === 'permission_extended' ? 'Permission Extended' : 'Permission Revoked',
      message: `${formatPermission(event.basePermission)} ${
        event.type === 'permission_granted' ? `granted until ${new Date(event.expiryTimestamp! * 1000).toLocaleDateString()}` :
        event.type === 'permission_extended' ? `extended until ${new Date(event.newExpiry! * 1000).toLocaleDateString()}` :
        'has been revoked'
      }`,
      permission: event.permission,
      basePermission: event.basePermission,
      platform: event.platform,
      timestamp: event.timestamp,
      expiresAt: event.expiryTimestamp || event.newExpiry,
      isRead: false,
      priority: event.type === 'permission_revoked' ? 'high' : 'medium',
      metadata: event.metadata,
    };

    addNotification(notification);
  }

  // Handle permission expired
  function handlePermissionExpired(event: PermissionUpdateEvent) {
    const notification: PermissionNotification = {
      id: generateNotificationId(),
      type: 'expired',
      title: 'Permission Expired',
      message: `${formatPermission(event.basePermission)} has expired`,
      permission: event.permission,
      basePermission: event.basePermission,
      platform: event.platform,
      timestamp: event.timestamp,
      isRead: false,
      priority: 'critical',
      metadata: event.metadata,
    };

    addNotification(notification);
  }

  // Handle permission expiring soon
  function handlePermissionExpiring(event: PermissionUpdateEvent) {
    const notification: PermissionNotification = {
      id: generateNotificationId(),
      type: 'expiring_soon',
      title: 'Permission Expiring Soon',
      message: `${formatPermission(event.basePermission)} expires in ${formatTimeRemaining(event.expiryTimestamp!)}`,
      permission: event.permission,
      basePermission: event.basePermission,
      platform: event.platform,
      timestamp: event.timestamp,
      expiresAt: event.expiryTimestamp,
      isRead: false,
      priority: 'high',
      metadata: event.metadata,
    };

    addNotification(notification);
  }

  // Add notification with deduplication
  const addNotification = useCallback((notification: PermissionNotification) => {
    setNotifications(prev => {
      // Remove duplicate notifications for the same permission
      const filtered = prev.filter(n => 
        n.basePermission !== notification.basePermission || 
        n.type !== notification.type
      );
      
      // Add new notification and limit total count
      const updated = [notification, ...filtered].slice(0, config.maxNotifications);
      
      // Auto-hide notification after delay
      if (config.autoHideDelay > 0) {
        setTimeout(() => {
          setNotifications(current => current.filter(n => n.id !== notification.id));
        }, config.autoHideDelay);
      }
      
      return updated;
    });

    // Play notification sound if enabled
    if (config.enableSound && notification.priority === 'critical') {
      try {
        const audio = new Audio('/sounds/notification.mp3');
        audio.play().catch(() => {}); // Ignore errors
      } catch (error) {
        // Ignore audio errors
      }
    }
  }, [config.maxNotifications, config.autoHideDelay, config.enableSound]);

  // Remove notification
  const removeNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  }, []);

  // Mark notification as read
  const markAsRead = useCallback((id: string) => {
    setNotifications(prev => 
      prev.map(n => n.id === id ? { ...n, isRead: true } : n)
    );
  }, []);

  // Clear all notifications
  const clearAll = useCallback(() => {
    setNotifications([]);
  }, []);

  // Handle notification click
  const handleNotificationClick = useCallback((notification: PermissionNotification) => {
    markAsRead(notification.id);
    config.onNotificationClick?.(notification);
  }, [markAsRead, config]);

  // Check for expiring permissions periodically
  useEffect(() => {
    const checkExpiringPermissions = () => {
      if (!expiry.hasExpiringSoon) return;
      
      expiry.expiryInfo.expiringSoon.forEach(permission => {
        const existingNotification = notifications.find(n => 
          n.basePermission === permission.basePermission && n.type === 'expiring_soon'
        );
        
        if (!existingNotification && permission.expiresAt) {
          const notification: PermissionNotification = {
            id: generateNotificationId(),
            type: 'expiring_soon',
            title: 'Permission Expiring Soon',
            message: `${formatPermission(permission.basePermission)} expires in ${permission.expiresIn}`,
            permission: permission.permission,
            basePermission: permission.basePermission,
            platform: permission.basePermission.split(':')[0] || 'epsx',
            timestamp: Date.now(),
            expiresAt: permission.expiresAt,
            isRead: false,
            priority: 'high',
          };
          
          addNotification(notification);
        }
      });
    };

    const interval = setInterval(checkExpiringPermissions, 60000); // Check every minute
    checkExpiringPermissions(); // Check immediately
    
    return () => clearInterval(interval);
  }, [expiry.hasExpiringSoon, expiry.expiryInfo.expiringSoon, notifications, addNotification, generateNotificationId, formatPermission]);

  const unreadCount = notifications.filter(n => !n.isRead).length;
  const hasUnread = unreadCount > 0;

  const positionClasses = {
    'top-right': 'top-4 right-4',
    'top-left': 'top-4 left-4',
    'bottom-right': 'bottom-4 right-4',
    'bottom-left': 'bottom-4 left-4',
  };

  return (
    <div className={cn('fixed z-50', positionClasses[config.position], className)}>
      {/* Notification Bell */}
      <div className="relative mb-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setIsOpen(!isOpen)}
          className={cn(
            'relative p-2 shadow-lg transition-colors',
            hasUnread && 'bg-yellow-50 border-yellow-200'
          )}
        >
          <Bell className={cn('h-4 w-4', hasUnread && 'text-yellow-600')} />
          {config.showUnreadCount && hasUnread && (
            <Badge 
              variant="destructive" 
              className="absolute -top-1 -right-1 h-5 w-5 p-0 flex items-center justify-center text-xs"
            >
              {unreadCount > 99 ? '99+' : unreadCount}
            </Badge>
          )}
        </Button>
        
        {/* Connection Status */}
        <div className="absolute -bottom-1 -right-1">
          <div className={cn(
            'h-3 w-3 rounded-full border-2 border-white',
            realtime.isConnected ? 'bg-green-500' : 'bg-gray-400'
          )} />
        </div>
      </div>

      {/* Notifications Panel */}
      {isOpen && (
        <div className="bg-white border border-gray-200 rounded-lg shadow-xl w-80 max-h-96 overflow-y-auto">
          {/* Header */}
          <div className="flex items-center justify-between p-3 border-b border-gray-100">
            <h3 className="font-medium text-gray-900">Permission Updates</h3>
            {notifications.length > 0 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={clearAll}
                className="text-xs text-gray-500 hover:text-gray-700"
              >
                Clear All
              </Button>
            )}
          </div>

          {/* Notifications List */}
          <div className="max-h-80 overflow-y-auto">
            {notifications.length === 0 ? (
              <div className="p-6 text-center text-gray-500">
                <Bell className="h-8 w-8 mx-auto mb-2 text-gray-300" />
                <p className="text-sm">No permission updates</p>
              </div>
            ) : (
              <div className="divide-y divide-gray-100">
                {notifications.map((notification) => {
                  const Icon = NOTIFICATION_ICONS[notification.type];
                  return (
                    <div
                      key={notification.id}
                      className={cn(
                        'p-3 cursor-pointer hover:bg-gray-50 transition-colors relative',
                        !notification.isRead && 'bg-blue-50'
                      )}
                      onClick={() => handleNotificationClick(notification)}
                    >
                      {/* Remove Button */}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          removeNotification(notification.id);
                        }}
                        className="absolute top-2 right-2 text-gray-400 hover:text-gray-600"
                      >
                        <X className="h-3 w-3" />
                      </button>

                      {/* Notification Content */}
                      <div className="flex items-start space-x-3 pr-6">
                        <div className={cn(
                          'flex-shrink-0 p-1 rounded-full',
                          NOTIFICATION_COLORS[notification.type]
                        )}>
                          <Icon className="h-4 w-4" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="font-medium text-gray-900 text-sm">
                            {notification.title}
                          </p>
                          <p className="text-gray-600 text-xs mt-1 line-clamp-2">
                            {notification.message}
                          </p>
                          <div className="flex items-center justify-between mt-2">
                            <Badge variant="outline" className="text-xs">
                              {notification.platform.toUpperCase()}
                            </Badge>
                            <span className="text-xs text-gray-400">
                              {new Date(notification.timestamp).toLocaleTimeString()}
                            </span>
                          </div>
                        </div>
                      </div>
                      
                      {/* Unread indicator */}
                      {!notification.isRead && (
                        <div className="absolute left-1 top-1/2 transform -translate-y-1/2 h-2 w-2 bg-blue-500 rounded-full" />
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}