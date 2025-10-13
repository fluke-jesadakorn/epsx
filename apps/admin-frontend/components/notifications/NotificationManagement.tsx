'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { Bell, BarChart3, Clock, AlertTriangle } from 'lucide-react';
import { createNotificationsClient } from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { SendNotificationForm } from './SendNotificationForm';

type Notification = any;
type NotificationStats = any;

interface NotificationManagementProps {
  currentUser: any;
}

export function NotificationManagement({ currentUser }: NotificationManagementProps) {
  const router = useRouter();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [stats, setStats] = useState<NotificationStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const client = createNotificationsClient(createAdminApiClient());

      const [notificationsResponse, statsResponse] = await Promise.all([
        client.getAllNotifications({ page: 1, limit: 20 }),
        client.getNotificationStats()
      ]);

      setNotifications(notificationsResponse.data.notifications);

      const backendStats = statsResponse.data;
      setStats({
        total: backendStats.total_notifications,
        unread: 0,
        last24Hours: backendStats.sent_today,
        lastWeek: backendStats.sent_this_week,
        byType: backendStats.by_type || {},
        byPriority: backendStats.by_priority || {},
      });
    } catch (err) {
      console.error('Failed to load notifications:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto space-y-8">
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-blue-400 to-purple-500 rounded-2xl w-96 mx-auto mb-6"></div>
          <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto"></div>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-gray-300 rounded-3xl h-32"></div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="space-y-6 sm:space-y-8">
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-purple-500/20 rounded-full blur-xl"></div>
          <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-orange-500/20 rounded-full blur-lg"></div>
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-purple-400/15 to-blue-500/15 rounded-full blur-xl"></div>
        </div>

        <div className="relative max-w-7xl mx-auto">
          <div className="text-center mb-8 sm:mb-12">
            <div className="relative inline-block">
              <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-blue-600 via-purple-600 via-pink-600 to-orange-600 bg-clip-text text-transparent mb-4">
                🔔 Notification Management
              </h1>
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full"></div>
            </div>
            <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
              Send and manage system notifications for users
            </p>
          </div>

          {stats && (
            <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">📬</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-blue-600 dark:text-blue-400">{stats.total}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Notifications</div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">All time</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">⚠️</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Unread</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-orange-600 dark:text-orange-400">{stats.unread}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Pending</div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">Action needed</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">📅</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Today</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-purple-600 dark:text-purple-400">{stats.last24Hours}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Last 24h</div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">Recent</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">📊</div>
                  <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Week</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-green-600 dark:text-green-400">{stats.lastWeek}</div>
                  <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Last 7 days</div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">Weekly</div>
                </div>
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-500/20 to-pink-500/20 p-0.5 cursor-pointer"
              onClick={loadData}
            >
              <div className="relative bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 text-white rounded-2xl sm:rounded-3xl">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">🔄</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                  <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload notification data and statistics</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    Refresh
                  </div>
                </div>
              </div>
            </div>

            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-orange-400/20 via-pink-500/20 to-red-500/20 p-0.5 cursor-pointer"
              onClick={() => router.push('/notifications/analytics')}
            >
              <div className="relative bg-gradient-to-br from-orange-400 via-pink-500 to-red-500 text-white rounded-2xl sm:rounded-3xl">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">📈</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">View Analytics</h3>
                  <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Detailed notification performance metrics</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    Analytics
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-pink-400/20 via-orange-400/20 to-purple-400/20 p-0.5">
            <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
              <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-pink-600 via-orange-600 to-purple-600 bg-clip-text text-transparent mb-6">
                Recent Notifications
              </h2>

              {notifications.length === 0 ? (
                <div className="text-center py-12 sm:py-16">
                  <div className="h-20 w-20 bg-gradient-to-br from-blue-200 to-purple-200 dark:from-blue-800 dark:to-purple-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-4xl">📭</span>
                  </div>
                  <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                    No notifications yet
                  </h3>
                  <p className="text-gray-500 dark:text-gray-500">
                    Start by sending your first notification
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  {notifications.slice(0, 5).map(notification => (
                    <div
                      key={notification.id}
                      className="p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl"
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="flex-1">
                          <h3 className="font-medium text-gray-900 dark:text-white mb-1">
                            {notification.title}
                          </h3>
                          <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2 mb-2">
                            {notification.message}
                          </p>
                          <div className="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-500">
                            <span>{notification.notification_type}</span>
                            <span>•</span>
                            <span>{new Date(notification.timestamp).toLocaleString()}</span>
                          </div>
                        </div>
                        <div className={`px-3 py-1 rounded-full text-xs font-semibold ${
                          notification.priority === 'critical' ? 'bg-red-100 text-red-600' :
                          notification.priority === 'high' ? 'bg-orange-100 text-orange-600' :
                          notification.priority === 'normal' ? 'bg-blue-100 text-blue-600' :
                          'bg-green-100 text-green-600'
                        }`}>
                          {notification.priority}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
