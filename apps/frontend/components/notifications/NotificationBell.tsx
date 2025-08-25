'use client';

import React from 'react';
import Link from 'next/link';
import { Bell } from 'lucide-react';
import { useNotifications } from '@/context/notification-context';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface NotificationBellProps {
  variant?: 'default' | 'mobile';
  showBadge?: boolean;
  onClick?: () => void;
  href?: string;
}

export function NotificationBell({ 
  variant = 'default', 
  showBadge = true, 
  onClick,
  href = '/notifications' 
}: NotificationBellProps) {
  const { unreadCount, realtimeConnected } = useNotifications();

  const BellIcon = (
    <div className="relative">
      <Bell 
        className={`
          ${variant === 'mobile' ? 'h-5 w-5' : 'h-5 w-5'} 
          ${realtimeConnected ? 'text-foreground' : 'text-muted-foreground'}
          transition-colors duration-200
        `} 
      />
      {showBadge && unreadCount > 0 && (
        <Badge 
          variant="destructive" 
          className="absolute -top-2 -right-2 h-5 w-5 rounded-full p-0 flex items-center justify-center text-xs font-medium"
        >
          {unreadCount > 99 ? '99+' : unreadCount}
        </Badge>
      )}
      {!realtimeConnected && (
        <div className="absolute -top-1 -right-1 h-2 w-2 bg-yellow-500 rounded-full animate-pulse" />
      )}
    </div>
  );

  if (onClick) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size={variant === 'mobile' ? 'sm' : 'sm'}
              onClick={onClick}
              className="relative p-2 hover:bg-accent/10"
            >
              {BellIcon}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>
              {unreadCount > 0 
                ? `${unreadCount} unread notification${unreadCount > 1 ? 's' : ''}`
                : 'No new notifications'
              }
            </p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Link href={href} className="inline-block">
            <Button
              variant="ghost"
              size={variant === 'mobile' ? 'sm' : 'sm'}
              className="relative p-2 hover:bg-accent/10"
            >
              {BellIcon}
            </Button>
          </Link>
        </TooltipTrigger>
        <TooltipContent>
          <p>
            {unreadCount > 0 
              ? `${unreadCount} unread notification${unreadCount > 1 ? 's' : ''}`
              : 'No new notifications'
            }
          </p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export default NotificationBell;