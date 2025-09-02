'use client';

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { fcmClient, FCMNotificationPayload } from '@/lib/fcm-client';
import { Bell, BellOff, Settings } from 'lucide-react';
import { useRouter } from 'next/navigation';

interface FCMNotificationBellProps {
  className?: string;
}

export function FCMNotificationBell({ className }: FCMNotificationBellProps) {
  const [isSubscribed, setIsSubscribed] = useState(false);
  const [isSupported, setIsSupported] = useState(false);
  const [hasPermission, setHasPermission] = useState(false);
  const [recentNotifications, setRecentNotifications] = useState<FCMNotificationPayload[]>([]);
  const router = useRouter();

  // Initialize FCM status
  useEffect(() => {
    const initializeFCM = async () => {
      try {
        const status = await fcmClient.getSubscriptionStatus();
        setIsSupported(status.isSupported);
        setHasPermission(status.permission === 'granted');
        setIsSubscribed(status.isSubscribed);
      } catch (error) {
        console.error('Error initializing FCM status:', error);
      }
    };

    initializeFCM();
  }, []);

  // Set up FCM message listener
  useEffect(() => {
    if (!isSupported) return;

    const unsubscribe = fcmClient.onMessage((payload) => {
      // Add to recent notifications (keep last 5)
      setRecentNotifications(prev => {
        const updated = [payload, ...prev].slice(0, 5);
        return updated;
      });
    });

    return unsubscribe;
  }, [isSupported]);

  // Handle notification enable/disable
  const handleToggleNotifications = async () => {
    try {
      if (isSubscribed) {
        await fcmClient.unsubscribe();
        setIsSubscribed(false);
      } else {
        await fcmClient.subscribe();
        setIsSubscribed(true);
        setHasPermission(true);
      }
    } catch (error) {
      console.error('Error toggling FCM subscription:', error);
    }
  };

  // Handle notification click
  const handleNotificationClick = (payload: FCMNotificationPayload) => {
    const url = payload.data?.url || payload.fcmOptions?.link || '/notifications';
    router.push(url);
  };

  // Don't render if FCM is not supported
  if (!isSupported) {
    return null;
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button 
          variant="ghost" 
          size="sm" 
          className={`relative p-2 ${className}`}
        >
          {isSubscribed && hasPermission ? (
            <Bell className="h-4 w-4" />
          ) : (
            <BellOff className="h-4 w-4 text-muted-foreground" />
          )}
          
          {/* Notification count badge */}
          {recentNotifications.length > 0 && (
            <Badge 
              className="absolute -top-1 -right-1 h-5 w-5 p-0 text-xs flex items-center justify-center bg-red-500 hover:bg-red-600"
            >
              {recentNotifications.length}
            </Badge>
          )}
        </Button>
      </DropdownMenuTrigger>
      
      <DropdownMenuContent align="end" className="w-80">
        <DropdownMenuLabel className="flex items-center justify-between">
          Notifications
          <Button
            variant="ghost"
            size="sm"
            onClick={() => router.push('/notifications')}
          >
            <Settings className="h-3 w-3" />
          </Button>
        </DropdownMenuLabel>
        
        <DropdownMenuSeparator />
        
        {/* Subscription Status */}
        <DropdownMenuItem onClick={handleToggleNotifications}>
          <div className="flex items-center justify-between w-full">
            <span className="text-sm">
              {isSubscribed ? 'Disable Notifications' : 'Enable Notifications'}
            </span>
            {isSubscribed ? (
              <Bell className="h-4 w-4" />
            ) : (
              <BellOff className="h-4 w-4" />
            )}
          </div>
        </DropdownMenuItem>
        
        <DropdownMenuSeparator />
        
        {/* Recent Notifications */}
        {recentNotifications.length > 0 ? (
          <>
            <DropdownMenuLabel className="text-xs text-muted-foreground">
              Recent Notifications
            </DropdownMenuLabel>
            {recentNotifications.map((notification, index) => (
              <DropdownMenuItem 
                key={index}
                onClick={() => handleNotificationClick(notification)}
                className="flex flex-col items-start space-y-1 h-auto py-3"
              >
                <div className="font-medium text-sm">
                  {notification.notification?.title || notification.data?.title || 'Notification'}
                </div>
                <div className="text-xs text-muted-foreground line-clamp-2">
                  {notification.notification?.body || notification.data?.body || 'New notification received'}
                </div>
              </DropdownMenuItem>
            ))}
            <DropdownMenuSeparator />
          </>
        ) : (
          <DropdownMenuItem disabled>
            <span className="text-sm text-muted-foreground">No recent notifications</span>
          </DropdownMenuItem>
        )}
        
        {/* View All Notifications */}
        <DropdownMenuItem onClick={() => router.push('/notifications')}>
          <span className="text-sm font-medium">View All Notifications</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export default FCMNotificationBell;