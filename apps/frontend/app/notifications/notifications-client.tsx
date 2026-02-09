'use client';

import { getInitialNotificationsAction, markAsReadAction, markAllAsReadAction, deleteNotificationAction, clearAllNotificationsAction } from '@/app/actions/notifications';
import type { NotificationsResponse } from '@/shared/api/notifications';
import { Bell, Check, Filter, Trash2 } from 'lucide-react';
import { useSearchParams } from 'next/navigation';
import { useEffect, useRef, useState } from 'react';

interface Notification {
  id: string
  title: string
  message: string
  type: string
  priority: string
  timestamp: string
  actionUrl?: string
  read: boolean
}

type FilterStatus = 'all' | 'unread' | 'read';
type FilterNotificationType = 'all' | 'system' | 'security' | 'permission' | 'wallet' | 'payment';
type FilterNotificationPriority = 'all' | 'low' | 'normal' | 'high' | 'critical';

interface NotificationsClientProps {
  initialData: NotificationsResponse;
  focusId?: string | null;
}

export default function NotificationsClient({ initialData, focusId }: NotificationsClientProps) {
  const searchParams = useSearchParams();
  const notificationRefs = useRef<{ [key: string]: HTMLDivElement | null }>({})

  const [notifications, setNotifications] = useState<Notification[]>(
    initialData.data.notifications.map(n => ({
      id: n.id,
      title: n.title,
      message: n.message,
      type: n.notification_type,
      priority: n.priority,
      timestamp: n.timestamp,
      actionUrl: n.action_url,
      read: Boolean(n.read_at)
    }))
  );
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState<FilterStatus>('all');
  const [typeFilter, setTypeFilter] = useState<FilterNotificationType>('all');
  const [priorityFilter, setPriorityFilter] = useState<FilterNotificationPriority>('all');
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(initialData.data.total_pages);
  const [totalCount, setTotalCount] = useState(initialData.data.total_count);
  const [unreadCount, setUnreadCount] = useState(initialData.data.unread_count);
  const [focusedId, _setFocusedId] = useState<string | null>(focusId);

  useEffect(() => {
    fetchNotifications();
  }, [filter, typeFilter, priorityFilter, page]);

  useEffect(() => {
    if (focusId && notifications.length > 0) {
      const targetNotification = notifications.find(n => n.id === focusId);

      if (targetNotification) {
        const element = notificationRefs.current[focusId];
        if (element) {
          setTimeout(() => {
            element.scrollIntoView({ behavior: 'smooth', block: 'center' });
          }, 100);
        }

        if (!targetNotification.read) {
          markAsRead(focusId);
        }
      }
    }
  }, [focusId, notifications]);

  const fetchNotifications = async () => {
    try {
      setLoading(true);
      const data = await getInitialNotificationsAction({
        page,
        type: typeFilter === 'all' ? undefined : typeFilter,
        priority: priorityFilter === 'all' ? undefined : priorityFilter,
      });

      const mappedNotifications = data.data.notifications.map(n => ({
        id: n.id,
        title: n.title,
        message: n.message,
        type: n.notification_type,
        priority: n.priority,
        timestamp: n.timestamp,
        actionUrl: n.action_url,
        read: Boolean(n.read_at)
      }));

      setNotifications(mappedNotifications);
      setTotalCount(data.data.total_count);
      setUnreadCount(data.data.unread_count);
      setTotalPages(data.data.total_pages);
    } catch (_error) {
      setNotifications([]);
    } finally {
      setLoading(false);
    }
  };

  const markAsRead = async (notificationId: string) => {
    try {
      await markAsReadAction(notificationId);

      setNotifications(prev => prev.map(n =>
        n.id === notificationId ? { ...n, read: true } : n
      ));
      setUnreadCount(prev => Math.max(0, prev - 1));
    } catch (_error) {
      // Error handling handled by parent component
    }
  };

  const markAllAsRead = async () => {
    try {
      await markAllAsReadAction();

      setNotifications(prev => prev.map(n => ({ ...n, read: true })));
      setUnreadCount(0);
    } catch (_error) {
      // Error handling handled by parent component
    }
  };

  const deleteNotification = async (notificationId: string) => {
    try {
      await deleteNotificationAction(notificationId);

      setNotifications(prev => prev.filter(n => n.id !== notificationId));
      setTotalCount(prev => prev - 1);
    } catch (_error) {
      // Error handling handled by parent component
    }
  };

  const clearAll = async () => {
    if (!confirm('Are you sure you want to clear all notifications? This cannot be undone.')) {
      return;
    }

    try {
      await clearAllNotificationsAction();

      setNotifications([]);
      setTotalCount(0);
      setUnreadCount(0);
    } catch (_error) {
      // Error handling handled by parent component
    }
  };

  const getNotificationIcon = (type: string) => {
    switch (type) {
      case 'security': return '🔒';
      case 'permission': return '🔑';
      case 'wallet': return '💼';
      case 'payment': return '💳';
      case 'system': return '⚙️';
      default: return '📬';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical':
      case 'urgent': return 'bg-red-500';
      case 'high': return 'bg-orange-500';
      case 'normal': return 'bg-blue-500';
      case 'low': return 'bg-green-500';
      default: return 'bg-gray-500';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (minutes < 1) {return 'Just now';}
    if (minutes < 60) {return `${minutes}m ago`;}
    if (hours < 24) {return `${hours}h ago`;}
    if (days < 7) {return `${days}d ago`;}
    return date.toLocaleDateString();
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-orange-50/30 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
      <div className="mx-auto max-w-6xl px-4 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-slate-900 dark:text-slate-100">
                Notifications
              </h1>
              <p className="mt-2 text-slate-600 dark:text-slate-400">
                {totalCount} total, {unreadCount} unread
              </p>
            </div>
            <div className="flex gap-3">
              {unreadCount > 0 && (
                <button
                  onClick={markAllAsRead}
                  className="flex items-center gap-2 rounded-xl bg-blue-500 px-4 py-2 text-sm font-medium text-white hover:bg-blue-600"
                >
                  <Check className="h-4 w-4" />
                  Mark All Read
                </button>
              )}
              {totalCount > 0 && (
                <button
                  onClick={clearAll}
                  className="flex items-center gap-2 rounded-xl bg-red-500 px-4 py-2 text-sm font-medium text-white hover:bg-red-600"
                >
                  <Trash2 className="h-4 w-4" />
                  Clear All
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Filters */}
        <div className="mb-6 rounded-2xl border border-orange-100 bg-white p-4 dark:border-slate-700 dark:bg-slate-800">
          <div className="flex items-center gap-2 mb-3">
            <Filter className="h-5 w-5 text-orange-500" />
            <h2 className="text-sm font-semibold text-slate-700 dark:text-slate-300">
              Filters
            </h2>
          </div>

          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            {/* Status Filter */}
            <div>
              <label className="mb-2 block text-xs font-medium text-slate-600 dark:text-slate-400">
                Status
              </label>
              <div className="flex gap-2">
                {(['all', 'unread', 'read'] as FilterStatus[]).map((f) => (
                  <button
                    key={f}
                    onClick={() => setFilter(f)}
                    className={`rounded-lg px-3 py-1.5 text-xs font-medium ${filter === f
                      ? 'bg-orange-500 text-white'
                      : 'bg-slate-100 text-slate-600 hover:bg-slate-200 dark:bg-slate-700 dark:text-slate-300 dark:hover:bg-slate-600'
                      }`}
                  >
                    {f.charAt(0).toUpperCase() + f.slice(1)}
                  </button>
                ))}
              </div>
            </div>

            {/* Type Filter */}
            <div>
              <label className="mb-2 block text-xs font-medium text-slate-600 dark:text-slate-400">
                Type
              </label>
              <select
                value={typeFilter}
                onChange={(e) => setTypeFilter(e.target.value as FilterNotificationType)}
                className="w-full rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 dark:border-slate-600 dark:bg-slate-700 dark:text-slate-300"
              >
                <option value="all">All Types</option>
                <option value="system">System</option>
                <option value="security">Security</option>
                <option value="permission">Permission</option>
                <option value="wallet">Wallet</option>
                <option value="payment">Payment</option>
              </select>
            </div>

            {/* Priority Filter */}
            <div>
              <label className="mb-2 block text-xs font-medium text-slate-600 dark:text-slate-400">
                Priority
              </label>
              <select
                value={priorityFilter}
                onChange={(e) => setPriorityFilter(e.target.value as FilterNotificationPriority)}
                className="w-full rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 dark:border-slate-600 dark:bg-slate-700 dark:text-slate-300"
              >
                <option value="all">All Priorities</option>
                <option value="low">Low</option>
                <option value="normal">Normal</option>
                <option value="high">High</option>
                <option value="urgent">Urgent</option>
              </select>
            </div>
          </div>
        </div>

        {/* Notifications List */}
        {loading ? (
          <div className="flex items-center justify-center py-12">
            <Bell className="h-12 w-12 text-slate-300 dark:text-slate-600 animate-pulse" />
          </div>
        ) : notifications.length === 0 ? (
          <div className="rounded-2xl border border-orange-100 bg-white p-12 text-center dark:border-slate-700 dark:bg-slate-800">
            <Bell className="mx-auto mb-4 h-16 w-16 text-slate-300 dark:text-slate-600" />
            <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
              No notifications
            </h3>
            <p className="mt-2 text-sm text-slate-600 dark:text-slate-400">
              {filter === 'unread'
                ? "You're all caught up! No unread notifications."
                : "You don't have any notifications yet."}
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {notifications.map((notification) => {
              const isFocused = focusedId === notification.id;
              return (
                <div
                  key={notification.id}
                  ref={(el) => { notificationRefs.current[notification.id] = el }}
                  className={`rounded-2xl border p-4 ${isFocused
                    ? 'border-blue-400 bg-blue-50 dark:border-blue-600 dark:bg-blue-950/30 ring-2 ring-blue-400/50'
                    : notification.read
                      ? 'border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-800'
                      : 'border-orange-200 bg-orange-50/50 dark:border-orange-900/30 dark:bg-orange-950/20'
                    }`}
                >

                  <div className="flex items-start gap-4">
                    <div className={`w-10 h-10 rounded-full ${getPriorityColor(notification.priority)} flex items-center justify-center flex-shrink-0`}>
                      <span className="text-lg">{getNotificationIcon(notification.type)}</span>
                    </div>

                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-3">
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                              {notification.title}
                            </h3>
                            {!notification.read && (
                              <div className="w-2 h-2 rounded-full bg-orange-500 flex-shrink-0" />
                            )}
                          </div>
                          <p className="mt-1 text-sm text-slate-600 dark:text-slate-400">
                            {notification.message}
                          </p>
                          <div className="mt-2 flex items-center gap-3 text-xs text-slate-500 dark:text-slate-500">
                            <span className="capitalize">{notification.type}</span>
                            <span>•</span>
                            <span className="capitalize">{notification.priority}</span>
                            <span>•</span>
                            <span>{formatTimestamp(notification.timestamp)}</span>
                          </div>
                        </div>

                        <div className="flex items-center gap-2">
                          {!notification.read && (
                            <button
                              onClick={() => markAsRead(notification.id)}
                              className="rounded-lg p-2 text-blue-600 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
                              title="Mark as read"
                            >
                              <Check className="h-4 w-4" />
                            </button>
                          )}
                          <button
                            onClick={() => deleteNotification(notification.id)}
                            className="rounded-lg p-2 text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                            title="Delete"
                          >
                            <Trash2 className="h-4 w-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="mt-6 flex items-center justify-center gap-2">
            <button
              onClick={() => setPage(p => Math.max(1, p - 1))}
              disabled={page === 1}
              className="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-50 disabled:cursor-not-allowed dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
            >
              Previous
            </button>
            <span className="text-sm text-slate-600 dark:text-slate-400">
              Page {page} of {totalPages}
            </span>
            <button
              onClick={() => setPage(p => Math.min(totalPages, p + 1))}
              disabled={page === totalPages}
              className="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-50 disabled:cursor-not-allowed dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
            >
              Next
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
