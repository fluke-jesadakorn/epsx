'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { formatDistanceToNow } from 'date-fns';
import { 
  Bell, 
  TrendingUp, 
  AlertTriangle, 
  CreditCard, 
  Settings,
  Eye,
  EyeOff,
  Trash2,
  ExternalLink,
  MoreHorizontal
} from 'lucide-react';
import type { Notification } from '@/lib/state/types';
import { useNotifications } from '@/context/notification-context';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';

interface NotificationItemProps {
  notification: Notification;
  compact?: boolean;
  showActions?: boolean;
  onClick?: (notification: Notification) => void;
}

const notificationIcons = {
  trading: TrendingUp,
  system: Settings,
  account: CreditCard,
  price_alert: AlertTriangle,
};

const notificationColors = {
  trading: 'text-green-600 bg-green-50',
  system: 'text-blue-600 bg-blue-50', 
  account: 'text-purple-600 bg-purple-50',
  price_alert: 'text-orange-600 bg-orange-50',
};

export function NotificationItem({ 
  notification, 
  compact = false, 
  showActions = true,
  onClick 
}: NotificationItemProps) {
  const { markRead, deleteNotification } = useNotifications();
  const [isLoading, setIsLoading] = useState(false);

  const Icon = notificationIcons[notification.type] || Bell;
  const iconColor = notificationColors[notification.type] || 'text-gray-600 bg-gray-50';

  const handleMarkRead = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (notification.read || isLoading) return;

    setIsLoading(true);
    try {
      await markRead(notification.id);
    } catch (error) {
      console.error('Failed to mark notification as read:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isLoading) return;

    setIsLoading(true);
    try {
      await deleteNotification(notification.id);
    } catch (error) {
      console.error('Failed to delete notification:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClick = () => {
    if (onClick) {
      onClick(notification);
    }
    
    // Auto-mark as read when clicked
    if (!notification.read) {
      markRead(notification.id);
    }

    // Navigate to action URL if provided
    if (notification.actionUrl) {
      window.open(notification.actionUrl, '_blank');
    }
  };

  const formattedTime = formatDistanceToNow(new Date(notification.createdAt), { 
    addSuffix: true 
  });

  return (
    <Card 
      className={`
        group transition-all duration-200 cursor-pointer hover:shadow-md
        ${notification.read 
          ? 'bg-background border-border/50' 
          : 'bg-accent/5 border-primary/20 shadow-sm'
        }
        ${compact ? 'p-2' : ''}
      `}
      onClick={handleClick}
    >
      <CardContent className={`flex items-start gap-3 ${compact ? 'p-3' : 'p-4'}`}>
        {/* Icon */}
        <div className={`
          flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center
          ${iconColor}
        `}>
          <Icon className="h-4 w-4" />
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <div className="flex-1 min-w-0">
              <h3 className={`
                font-medium text-sm line-clamp-1
                ${notification.read ? 'text-muted-foreground' : 'text-foreground'}
              `}>
                {notification.title}
              </h3>
              <p className={`
                text-sm mt-1 
                ${compact ? 'line-clamp-1' : 'line-clamp-2'}
                ${notification.read ? 'text-muted-foreground/70' : 'text-muted-foreground'}
              `}>
                {notification.message}
              </p>
            </div>

            {/* Actions */}
            {showActions && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
                  <Button 
                    variant="ghost" 
                    size="sm" 
                    className="h-8 w-8 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
                    disabled={isLoading}
                  >
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {!notification.read && (
                    <DropdownMenuItem onClick={handleMarkRead} disabled={isLoading}>
                      <Eye className="h-4 w-4 mr-2" />
                      Mark as read
                    </DropdownMenuItem>
                  )}
                  {notification.actionUrl && (
                    <DropdownMenuItem 
                      onClick={(e) => {
                        e.stopPropagation();
                        window.open(notification.actionUrl, '_blank');
                      }}
                    >
                      <ExternalLink className="h-4 w-4 mr-2" />
                      Open link
                    </DropdownMenuItem>
                  )}
                  <DropdownMenuItem 
                    onClick={handleDelete} 
                    disabled={isLoading}
                    className="text-destructive focus:text-destructive"
                  >
                    <Trash2 className="h-4 w-4 mr-2" />
                    Delete
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-between mt-2">
            <div className="flex items-center gap-2">
              <Badge variant="outline" className="text-xs">
                {notification.type}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {formattedTime}
              </span>
            </div>
            
            {!notification.read && (
              <div className="w-2 h-2 bg-primary rounded-full flex-shrink-0" />
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export default NotificationItem;