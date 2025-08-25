'use client';

import React, { useState, useCallback, useEffect } from 'react';
import { Bell, Filter, Search, CheckCheck, Trash2, RefreshCw } from 'lucide-react';
import { useNotifications } from '@/context/notification-context';
import type { Notification } from '@/lib/state/types';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import { NotificationItem } from './NotificationItem';

interface NotificationListProps {
  showFilters?: boolean;
  showTabs?: boolean;
  showBulkActions?: boolean;
  itemsPerPage?: number;
}

export function NotificationList({
  showFilters = true,
  showTabs = true,
  showBulkActions = true,
  itemsPerPage = 20
}: NotificationListProps) {
  const { 
    notifications, 
    loading, 
    error, 
    refreshNotifications,
    markAllRead,
    deleteNotification,
    hasUnreadNotifications,
    getNotificationsByType,
    getUnreadNotifications
  } = useNotifications();

  const [selectedTab, setSelectedTab] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [typeFilter, setTypeFilter] = useState<string>('all');
  const [selectedNotifications, setSelectedNotifications] = useState<Set<string>>(new Set());
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isBulkActionLoading, setIsBulkActionLoading] = useState(false);

  // Filter notifications based on active tab and filters
  const filteredNotifications = React.useMemo(() => {
    let filtered = notifications;

    // Filter by tab
    switch (selectedTab) {
      case 'unread':
        filtered = getUnreadNotifications();
        break;
      case 'trading':
        filtered = getNotificationsByType('trading');
        break;
      case 'system':
        filtered = getNotificationsByType('system');
        break;
      case 'account':
        filtered = getNotificationsByType('account');
        break;
      default:
        filtered = notifications;
    }

    // Filter by type
    if (typeFilter !== 'all') {
      filtered = filtered.filter(n => n.type === typeFilter);
    }

    // Filter by search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(n => 
        n.title.toLowerCase().includes(query) ||
        n.message.toLowerCase().includes(query)
      );
    }

    // Sort: unread first, then by date
    return filtered.sort((a, b) => {
      if (!a.read && b.read) return -1;
      if (a.read && !b.read) return 1;
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    });
  }, [notifications, selectedTab, typeFilter, searchQuery, getUnreadNotifications, getNotificationsByType]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshNotifications();
    } catch (error) {
      console.error('Failed to refresh notifications:', error);
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedNotifications(new Set(filteredNotifications.map(n => n.id)));
    } else {
      setSelectedNotifications(new Set());
    }
  };

  const handleSelectNotification = (id: string, checked: boolean) => {
    const newSelected = new Set(selectedNotifications);
    if (checked) {
      newSelected.add(id);
    } else {
      newSelected.delete(id);
    }
    setSelectedNotifications(newSelected);
  };

  const handleBulkMarkRead = async () => {
    if (selectedNotifications.size === 0 || isBulkActionLoading) return;
    
    setIsBulkActionLoading(true);
    try {
      if (selectedNotifications.size === filteredNotifications.length && selectedTab === 'all') {
        // Mark all notifications as read
        await markAllRead();
      } else {
        // Mark selected notifications as read
        // TODO: Implement bulk mark read for specific notifications
        console.log('Bulk mark read for selected notifications:', Array.from(selectedNotifications));
      }
      setSelectedNotifications(new Set());
    } catch (error) {
      console.error('Failed to mark notifications as read:', error);
    } finally {
      setIsBulkActionLoading(false);
    }
  };

  const handleBulkDelete = async () => {
    if (selectedNotifications.size === 0 || isBulkActionLoading) return;
    
    setIsBulkActionLoading(true);
    try {
      // Delete selected notifications one by one
      const deletePromises = Array.from(selectedNotifications).map(id => 
        deleteNotification(id)
      );
      await Promise.all(deletePromises);
      setSelectedNotifications(new Set());
    } catch (error) {
      console.error('Failed to delete notifications:', error);
    } finally {
      setIsBulkActionLoading(false);
    }
  };

  const unreadCount = getUnreadNotifications().length;
  const tradingCount = getNotificationsByType('trading').length;
  const systemCount = getNotificationsByType('system').length;
  const accountCount = getNotificationsByType('account').length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Bell className="h-5 w-5 text-muted-foreground" />
          <h1 className="text-2xl font-bold">Notifications</h1>
          {hasUnreadNotifications() && (
            <Badge variant="secondary">{unreadCount} unread</Badge>
          )}
        </div>
        
        <Button 
          variant="outline" 
          size="sm"
          onClick={handleRefresh}
          disabled={isRefreshing}
        >
          <RefreshCw className={`h-4 w-4 mr-2 ${isRefreshing ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Filters */}
      {showFilters && (
        <Card>
          <CardContent className="p-4">
            <div className="flex flex-col sm:flex-row gap-4">
              <div className="flex-1">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search notifications..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-10"
                  />
                </div>
              </div>
              
              <Select value={typeFilter} onValueChange={setTypeFilter}>
                <SelectTrigger className="w-full sm:w-48">
                  <SelectValue placeholder="Filter by type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All types</SelectItem>
                  <SelectItem value="trading">Trading</SelectItem>
                  <SelectItem value="system">System</SelectItem>
                  <SelectItem value="account">Account</SelectItem>
                  <SelectItem value="price_alert">Price Alerts</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Tabs */}
      {showTabs ? (
        <Tabs value={selectedTab} onValueChange={setSelectedTab}>
          <TabsList className="grid w-full grid-cols-5">
            <TabsTrigger value="all">
              All ({notifications.length})
            </TabsTrigger>
            <TabsTrigger value="unread">
              Unread ({unreadCount})
            </TabsTrigger>
            <TabsTrigger value="trading">
              Trading ({tradingCount})
            </TabsTrigger>
            <TabsTrigger value="system">
              System ({systemCount})
            </TabsTrigger>
            <TabsTrigger value="account">
              Account ({accountCount})
            </TabsTrigger>
          </TabsList>

          <TabsContent value={selectedTab} className="mt-6">
            <NotificationListContent 
              notifications={filteredNotifications}
              selectedNotifications={selectedNotifications}
              onSelectNotification={handleSelectNotification}
              onSelectAll={handleSelectAll}
              onBulkMarkRead={handleBulkMarkRead}
              onBulkDelete={handleBulkDelete}
              showBulkActions={showBulkActions}
              loading={loading}
              error={error}
              isBulkActionLoading={isBulkActionLoading}
            />
          </TabsContent>
        </Tabs>
      ) : (
        <NotificationListContent 
          notifications={filteredNotifications}
          selectedNotifications={selectedNotifications}
          onSelectNotification={handleSelectNotification}
          onSelectAll={handleSelectAll}
          onBulkMarkRead={handleBulkMarkRead}
          onBulkDelete={handleBulkDelete}
          showBulkActions={showBulkActions}
          loading={loading}
          error={error}
          isBulkActionLoading={isBulkActionLoading}
        />
      )}
    </div>
  );
}

interface NotificationListContentProps {
  notifications: Notification[];
  selectedNotifications: Set<string>;
  onSelectNotification: (id: string, checked: boolean) => void;
  onSelectAll: (checked: boolean) => void;
  onBulkMarkRead: () => void;
  onBulkDelete: () => void;
  showBulkActions: boolean;
  loading: boolean;
  error: string | null;
  isBulkActionLoading: boolean;
}

function NotificationListContent({
  notifications,
  selectedNotifications,
  onSelectNotification,
  onSelectAll,
  onBulkMarkRead,
  onBulkDelete,
  showBulkActions,
  loading,
  error,
  isBulkActionLoading
}: NotificationListContentProps) {
  const allSelected = notifications.length > 0 && selectedNotifications.size === notifications.length;
  const someSelected = selectedNotifications.size > 0 && selectedNotifications.size < notifications.length;

  if (loading) {
    return (
      <div className="space-y-4">
        {Array.from({ length: 5 }).map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardContent className="p-4">
              <div className="flex items-start gap-3">
                <div className="w-8 h-8 bg-muted rounded-full" />
                <div className="flex-1 space-y-2">
                  <div className="h-4 bg-muted rounded w-3/4" />
                  <div className="h-3 bg-muted rounded w-full" />
                  <div className="h-3 bg-muted rounded w-1/2" />
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <Card>
        <CardContent className="p-8 text-center">
          <div className="text-destructive mb-2">Failed to load notifications</div>
          <div className="text-muted-foreground text-sm">{error}</div>
        </CardContent>
      </Card>
    );
  }

  if (notifications.length === 0) {
    return (
      <Card>
        <CardContent className="p-8 text-center text-muted-foreground">
          <Bell className="h-12 w-12 mx-auto mb-4 opacity-50" />
          <h3 className="text-lg font-medium mb-2">No notifications found</h3>
          <p className="text-sm">
            We'll notify you when something important happens.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {/* Bulk Actions */}
      {showBulkActions && (
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Checkbox
                  checked={allSelected}
                  ref={(el) => {
                    if (el) el.indeterminate = someSelected;
                  }}
                  onCheckedChange={onSelectAll}
                />
                <span className="text-sm text-muted-foreground">
                  {selectedNotifications.size > 0 
                    ? `${selectedNotifications.size} selected`
                    : 'Select all'
                  }
                </span>
              </div>

              {selectedNotifications.size > 0 && (
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={onBulkMarkRead}
                    disabled={isBulkActionLoading}
                  >
                    <CheckCheck className="h-4 w-4 mr-1" />
                    Mark as read
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={onBulkDelete}
                    disabled={isBulkActionLoading}
                  >
                    <Trash2 className="h-4 w-4 mr-1" />
                    Delete
                  </Button>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Notifications */}
      <div className="space-y-3">
        {notifications.map((notification) => (
          <div key={notification.id} className="flex items-start gap-3">
            {showBulkActions && (
              <div className="pt-4">
                <Checkbox
                  checked={selectedNotifications.has(notification.id)}
                  onCheckedChange={(checked) => 
                    onSelectNotification(notification.id, checked as boolean)
                  }
                />
              </div>
            )}
            <div className="flex-1">
              <NotificationItem
                notification={notification}
                showActions={!showBulkActions}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default NotificationList;