'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { Bell, Settings, CheckCheck, ArrowRight } from 'lucide-react';
import { useNotifications } from '@/context/notification-context';
import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger, DropdownMenuSeparator } from '@/components/ui/dropdown-menu';
import { ScrollArea } from '@/components/ui/scroll-area';
import { NotificationItem } from './NotificationItem';

interface NotificationDropdownProps {
  trigger?: React.ReactNode;
  maxItems?: number;
}

export function NotificationDropdown({ 
  trigger, 
  maxItems = 5 
}: NotificationDropdownProps) {
  const { 
    notifications, 
    unreadCount, 
    loading, 
    markAllRead,
    realtimeConnected 
  } = useNotifications();
  
  const [isMarkingAllRead, setIsMarkingAllRead] = useState(false);

  // Get recent notifications (unread first, then most recent)
  const recentNotifications = [...notifications]
    .sort((a, b) => {
      // Unread notifications first
      if (!a.read && b.read) return -1;
      if (a.read && !b.read) return 1;
      // Then by creation date (newest first)
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    })
    .slice(0, maxItems);

  const handleMarkAllRead = async () => {
    if (unreadCount === 0 || isMarkingAllRead) return;
    
    setIsMarkingAllRead(true);
    try {
      await markAllRead();
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error);
    } finally {
      setIsMarkingAllRead(false);
    }
  };

  const defaultTrigger = (
    <Button variant="ghost" size="sm" className="relative p-2 hover:bg-accent/10">
      <div className="relative">
        <Bell className={`h-5 w-5 ${realtimeConnected ? 'text-foreground' : 'text-muted-foreground'}`} />
        {unreadCount > 0 && (
          <div className="absolute -top-2 -right-2 h-5 w-5 bg-destructive text-destructive-foreground rounded-full text-xs font-medium flex items-center justify-center">
            {unreadCount > 99 ? '99+' : unreadCount}
          </div>
        )}
        {!realtimeConnected && (
          <div className="absolute -top-1 -right-1 h-2 w-2 bg-yellow-500 rounded-full animate-pulse" />
        )}
      </div>
    </Button>
  );

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        {trigger || defaultTrigger}
      </DropdownMenuTrigger>
      
      <DropdownMenuContent 
        align="end" 
        className="w-80 p-0"
        sideOffset={5}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <div className="flex items-center gap-2">
            <Bell className="h-4 w-4 text-muted-foreground" />
            <h3 className="font-semibold text-sm">Notifications</h3>
            {unreadCount > 0 && (
              <div className="bg-primary/10 text-primary px-2 py-0.5 rounded-full text-xs font-medium">
                {unreadCount} new
              </div>
            )}
          </div>
          
          <div className="flex items-center gap-1">
            {unreadCount > 0 && (
              <Button
                variant="ghost"
                size="sm"
                className="h-8 px-2 text-xs"
                onClick={handleMarkAllRead}
                disabled={isMarkingAllRead}
              >
                <CheckCheck className="h-3 w-3 mr-1" />
                Mark all read
              </Button>
            )}
            
            <Link href="/notifications">
              <Button variant="ghost" size="sm" className="h-8 px-2">
                <Settings className="h-3 w-3" />
              </Button>
            </Link>
          </div>
        </div>

        {/* Notifications List */}
        <ScrollArea className="max-h-96">
          {loading ? (
            <div className="p-8 text-center text-muted-foreground">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary mx-auto mb-2" />
              <p className="text-sm">Loading notifications...</p>
            </div>
          ) : recentNotifications.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground">
              <Bell className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p className="text-sm">No notifications yet</p>
              <p className="text-xs mt-1">We'll notify you when something happens</p>
            </div>
          ) : (
            <div className="p-2 space-y-2">
              {recentNotifications.map((notification) => (
                <NotificationItem
                  key={notification.id}
                  notification={notification}
                  compact={true}
                  showActions={false}
                />
              ))}
            </div>
          )}
        </ScrollArea>

        {/* Footer */}
        {recentNotifications.length > 0 && (
          <>
            <DropdownMenuSeparator />
            <div className="p-2">
              <Link href="/notifications" className="w-full">
                <Button variant="ghost" className="w-full justify-center text-sm">
                  View all notifications
                  <ArrowRight className="h-3 w-3 ml-1" />
                </Button>
              </Link>
            </div>
          </>
        )}

        {/* Connection Status */}
        {!realtimeConnected && (
          <div className="px-4 py-2 bg-yellow-50 border-t border-yellow-200">
            <div className="flex items-center gap-2 text-yellow-700">
              <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse" />
              <span className="text-xs">Reconnecting...</span>
            </div>
          </div>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export default NotificationDropdown;