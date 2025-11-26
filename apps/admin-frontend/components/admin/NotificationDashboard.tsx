'use client';

import {
  Bell,
  Plus,
  Search,
  Filter,
  Send,
  Users,
  BarChart3,
  Settings,
  AlertTriangle,
  CheckCircle,
  Clock,
  Trash2,
  Edit,
  Eye
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import React, { useState, useEffect } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Select } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { SendNotificationForm } from '@/components/notifications/SendNotificationForm';
import {
  createNotificationsClient,
} from '@/shared/api/notifications';
import type {
  Notification as ApiNotification,
  NotificationFilters,
  NotificationPriority,
  NotificationType,
} from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';

interface NotificationDashboardProps {
  className?: string;
}

interface AdminNotification {
  id: string;
  title: string;
  message: string;
  notificationType: NotificationType;
  priority: NotificationPriority;
  createdAt: string;
  read: boolean;
  walletAddress?: string;
}

interface DashboardStats {
  total: number;
  unread: number;
  last24Hours: number;
  lastWeek: number;
  byType: Partial<Record<NotificationType, number>>;
  byPriority: Partial<Record<NotificationPriority, number>>;
}

/**
 *
 * @param root0
 * @param root0.className
 */
export function NotificationDashboard({ className }: NotificationDashboardProps) {
  const [notifications, setNotifications] = useState<AdminNotification[]>([]);
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState<NotificationType | 'all'>('all');
  const [selectedPriority, setSelectedPriority] = useState<NotificationPriority | 'all'>('all');
  const router = useRouter();
  const [selectedNotifications, setSelectedNotifications] = useState<Set<string>>(new Set());
  const [activeTab, setActiveTab] = useState('overview');
  const [showSendDialog, setShowSendDialog] = useState(false);

  // Pagination
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const itemsPerPage = 20;

  const mapNotification = (notification: ApiNotification): AdminNotification => ({
    id: notification.id,
    title: notification.title,
    message: notification.message,
    notificationType: notification.notification_type,
    priority: notification.priority,
    createdAt: notification.timestamp,
    read: Boolean(notification.read_at),
    walletAddress: notification.wallet_address,
  });

  useEffect(() => {
    loadNotifications();
    loadStats();
  }, [currentPage, selectedType, selectedPriority, searchQuery]);

  const loadNotifications = async () => {
    try {
      setLoading(true);
      const client = createNotificationsClient(createAdminApiClient());
      const filters: NotificationFilters = {
        page: currentPage,
        limit: itemsPerPage,
      };

      if (selectedType !== 'all') {
        filters.type = selectedType;
      }

      if (selectedPriority !== 'all') {
        filters.priority = selectedPriority;
      }

      const response = await client.getAllNotifications(filters);
      const apiNotifications = response.data.notifications.map(mapNotification);

      setNotifications(apiNotifications);
      setTotalPages(response.data.total_pages);
      setStats((previous) => {
        if (previous) {
          return {
            ...previous,
            total: response.data.total_count,
            unread: response.data.unread_count,
          };
        }

        return {
          total: response.data.total_count,
          unread: response.data.unread_count,
          last24Hours: 0,
          lastWeek: 0,
          byType: {},
          byPriority: {},
        };
      });
      setError(null);
    } catch (err) {
      console.error('Failed to load notifications:', err);
      setError('Failed to load notifications');
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const client = createNotificationsClient(createAdminApiClient());
      const response = await client.getNotificationStats();

      // Map backend response to frontend format
      const backendStats = response.data;
      setStats((previous) => ({
        total: backendStats.total_notifications,
        unread: previous?.unread ?? 0,
        last24Hours: backendStats.sent_today,
        lastWeek: backendStats.sent_this_week,
        byType: backendStats.by_type || {},
        byPriority: backendStats.by_priority || {},
      }));
    } catch (err) {
      console.error('Failed to load notification stats:', err);
    }
  };

  const handleDeleteNotification = async (id: string) => {
    try {
      // await apiClient.deleteNotification(id);
      setNotifications(prev => prev.filter(n => n.id !== id));
      loadStats(); // Refresh stats
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Failed to delete notification:', err);
    }
  };

  const handleBulkDelete = async () => {
    if (selectedNotifications.size === 0) {return;}
    
    try {
      // await apiClient.deleteNotifications(Array.from(selectedNotifications));
      setNotifications(prev => prev.filter(n => !selectedNotifications.has(n.id)));
      setSelectedNotifications(new Set());
      loadStats(); // Refresh stats
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Failed to delete notifications:', err);
    }
  };

  const handleBulkMarkRead = async () => {
    if (selectedNotifications.size === 0) {return;}
    
    try {
      // await apiClient.markNotificationsRead(Array.from(selectedNotifications));
      setNotifications(prev => prev.map(n => 
        selectedNotifications.has(n.id) ? { ...n, read: true } : n
      ));
      setSelectedNotifications(new Set());
      loadStats(); // Refresh stats
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Failed to mark notifications as read:', err);
    }
  };

  const getPriorityColor = (priority: NotificationPriority) => {
    switch (priority) {
      case 'critical':
        return 'bg-red-500';
      case 'high':
        return 'bg-orange-500';
      case 'normal':
        return 'bg-blue-500';
      case 'low':
        return 'bg-green-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getTypeIcon = (type: NotificationType) => {
    switch (type) {
      case 'system':
        return <Settings className="h-4 w-4" />;
      case 'security':
        return <AlertTriangle className="h-4 w-4" />;
      case 'permission':
        return <CheckCircle className="h-4 w-4" />;
      case 'wallet_management':
      case 'wallet':
        return <Users className="h-4 w-4" />;
      case 'payment':
        return <Send className="h-4 w-4" />;
      default:
        return <Bell className="h-4 w-4" />;
    }
  };

  const filteredNotifications = notifications.filter((notification) => {
    if (!searchQuery) {
      return true;
    }

    const needle = searchQuery.toLowerCase();
    return (
      notification.title.toLowerCase().includes(needle) ||
      notification.message.toLowerCase().includes(needle) ||
      (notification.walletAddress?.toLowerCase().includes(needle) ?? false)
    );
  });

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Notification Management</h1>
          <p className="text-muted-foreground">
            Manage and monitor system notifications
          </p>
        </div>
        <Button onClick={() => setShowSendDialog(true)} className="flex items-center gap-2">
          <Plus className="h-4 w-4" />
          Send Notification
        </Button>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card>
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-muted-foreground">Total</p>
                  <p className="text-2xl font-bold">{stats.total}</p>
                </div>
                <Bell className="h-8 w-8 text-muted-foreground" />
              </div>
            </CardContent>
          </Card>
          
          <Card>
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-muted-foreground">Unread</p>
                  <p className="text-2xl font-bold text-orange-600">{stats.unread}</p>
                </div>
                <AlertTriangle className="h-8 w-8 text-orange-600" />
              </div>
            </CardContent>
          </Card>
          
          <Card>
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-muted-foreground">Last 24h</p>
                  <p className="text-2xl font-bold text-blue-600">{stats.last24Hours}</p>
                </div>
                <Clock className="h-8 w-8 text-blue-600" />
              </div>
            </CardContent>
          </Card>
          
          <Card>
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-muted-foreground">This Week</p>
                  <p className="text-2xl font-bold text-green-600">{stats.lastWeek}</p>
                </div>
                <BarChart3 className="h-8 w-8 text-green-600" />
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Main Content */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="notifications">Notifications</TabsTrigger>
          <TabsTrigger value="broadcast">Broadcast</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Type Distribution */}
            <Card>
              <CardHeader>
                <CardTitle>Notifications by Type</CardTitle>
              </CardHeader>
              <CardContent>
                {stats && (
                  <div className="space-y-3">
                    {Object.entries(stats.byType).map(([type, count]) => (
                      <div key={type} className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          {getTypeIcon(type)}
                          <span className="capitalize">{type}</span>
                        </div>
                        <Badge variant="secondary">{String(count)}</Badge>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Priority Distribution */}
            <Card>
              <CardHeader>
                <CardTitle>Notifications by Priority</CardTitle>
              </CardHeader>
              <CardContent>
                {stats && (
                  <div className="space-y-3">
                    {Object.entries(stats.byPriority).map(([priority, count]) => (
                      <div key={priority} className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <div className={`w-3 h-3 rounded-full ${getPriorityColor(priority)}`} />
                          <span className="capitalize">{priority}</span>
                        </div>
                        <Badge variant="secondary">{String(count)}</Badge>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="notifications" className="space-y-6">
          {/* Filters */}
          <Card>
            <CardContent className="p-4">
              <div className="flex flex-col sm:flex-row gap-4">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search notifications..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-10"
                  />
                </div>
                
                <select 
                  value={selectedType}
                  onChange={(e) => setSelectedType(e.target.value as NotificationType | 'all')}
                  className="px-3 py-2 border border-input rounded-md bg-background"
                >
                  <option value="all">All Types</option>
                  <option value="system">System</option>
                  <option value="security">Security</option>
                  <option value="permission">Permission</option>
                  <option value="wallet_management">Wallet Management</option>
                  <option value="wallet">Wallet</option>
                  <option value="payment">Payment</option>
                  <option value="general">General</option>
                </select>
                
                <select 
                  value={selectedPriority}
                  onChange={(e) => setSelectedPriority(e.target.value as NotificationPriority | 'all')}
                  className="px-3 py-2 border border-input rounded-md bg-background"
                >
                  <option value="all">All Priorities</option>
                  <option value="critical">Critical</option>
                  <option value="high">High</option>
                  <option value="normal">Normal</option>
                  <option value="low">Low</option>
                </select>
              </div>
            </CardContent>
          </Card>

          {/* Bulk Actions */}
          {selectedNotifications.size > 0 && (
            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">
                    {selectedNotifications.size} notifications selected
                  </span>
                  <div className="flex gap-2">
                    <Button variant="outline" size="sm" onClick={handleBulkMarkRead}>
                      <CheckCircle className="h-4 w-4 mr-1" />
                      Mark as Read
                    </Button>
                    <Button variant="outline" size="sm" onClick={handleBulkDelete}>
                      <Trash2 className="h-4 w-4 mr-1" />
                      Delete
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Notifications List */}
          <div className="space-y-3">
            {loading ? (
              <Card>
                <CardContent className="p-8 text-center">
                  <p className="text-muted-foreground">Loading notifications...</p>
                </CardContent>
              </Card>
            ) : error ? (
              <Card>
                <CardContent className="p-8 text-center text-destructive">
                  {error}
                </CardContent>
              </Card>
            ) : filteredNotifications.length === 0 ? (
              <Card>
                <CardContent className="p-8 text-center text-muted-foreground">
                  No notifications found
                </CardContent>
              </Card>
            ) : (
              filteredNotifications.map((notification) => (
                <Card key={notification.id} className={notification.read ? undefined : 'bg-muted/30'}>
                  <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                      <input
                        type="checkbox"
                        checked={selectedNotifications.has(notification.id)}
                        onChange={(e) => {
                          const newSelected = new Set(selectedNotifications);
                          if (e.target.checked) {
                            newSelected.add(notification.id);
                          } else {
                            newSelected.delete(notification.id);
                          }
                          setSelectedNotifications(newSelected);
                        }}
                        className="mt-1"
                      />
                      
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between gap-4">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              {getTypeIcon(notification.notificationType)}
                              <h3 className="font-medium">{notification.title}</h3>
                              <Badge 
                                variant="secondary" 
                                className={`${getPriorityColor(notification.priority)} text-white`}
                              >
                                {notification.priority}
                              </Badge>
                              {!notification.read && (
                                <Badge variant="default">Unread</Badge>
                              )}
                            </div>
                            <p className="text-sm text-muted-foreground line-clamp-2">
                              {notification.message}
                            </p>
                            <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                              <span>Type: {notification.notificationType}</span>
                              <span>•</span>
                              <span>{new Date(notification.createdAt).toLocaleString()}</span>
                              {notification.walletAddress && (
                                <>
                                  <span>•</span>
                                  <span>Wallet: {notification.walletAddress}</span>
                                </>
                              )}
                            </div>
                          </div>
                          
                          <div className="flex items-center gap-2">
                            <Button variant="ghost" size="sm">
                              <Eye className="h-4 w-4" />
                            </Button>
                            <Button variant="ghost" size="sm">
                              <Edit className="h-4 w-4" />
                            </Button>
                            <Button 
                              variant="ghost" 
                              size="sm" 
                              onClick={() => handleDeleteNotification(notification.id)}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))
            )}
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex justify-center gap-2">
              <Button
                variant="outline"
                disabled={currentPage === 1}
                onClick={() => setCurrentPage(prev => prev - 1)}
              >
                Previous
              </Button>
              <span className="px-4 py-2 text-sm">
                Page {currentPage} of {totalPages}
              </span>
              <Button
                variant="outline"
                disabled={currentPage === totalPages}
                onClick={() => setCurrentPage(prev => prev + 1)}
              >
                Next
              </Button>
            </div>
          )}
        </TabsContent>

        <TabsContent value="broadcast" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Send className="h-5 w-5" />
                Broadcast Notification
              </CardTitle>
              <CardDescription>
                Send notifications to multiple users or all users
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground">
                Broadcast functionality will be implemented here.
              </p>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Send Notification Dialog */}
      <Dialog open={showSendDialog} onOpenChange={setShowSendDialog}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Send Notification</DialogTitle>
            <DialogDescription>
              Send a notification to a specific wallet or broadcast to all users
            </DialogDescription>
          </DialogHeader>
          <SendNotificationForm
            onSuccess={() => {
              setShowSendDialog(false);
              loadNotifications();
              loadStats();
            }}
            onCancel={() => setShowSendDialog(false)}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
}