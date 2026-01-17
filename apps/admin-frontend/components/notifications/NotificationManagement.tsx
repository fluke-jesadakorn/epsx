'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { createNotificationsClient } from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';

type Notification = any;
type NotificationStats = any;

interface NotificationManagementProps {
  currentUser: any;
}

/**
 *
 * @param root0
 * @param root0.currentUser
 */
export function NotificationManagement({ currentUser }: NotificationManagementProps) {
  const router = useRouter();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [stats, setStats] = useState<NotificationStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [deleteModal, setDeleteModal] = useState<{ show: boolean; id: string; title: string }>({
    show: false,
    id: '',
    title: ''
  });
  const [deleting, setDeleting] = useState(false);

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

      setNotifications(notificationsResponse?.data?.notifications || []);

      const backendStats = statsResponse?.data;
      if (backendStats) {
        setStats({
          total: backendStats.total_notifications,
          unread: 0,
          last24Hours: backendStats.sent_today,
          lastWeek: backendStats.sent_this_week,
          byType: backendStats.by_type || {},
          byPriority: backendStats.by_priority || {},
        });
      }
    } catch (err) {
      console.error('Failed to load notifications:', err);
    } finally {
      setLoading(false);
    }
  };

  const showDeleteModal = (id: string, title: string) => {
    setDeleteModal({ show: true, id, title });
  };

  const hideDeleteModal = () => {
    setDeleteModal({ show: false, id: '', title: '' });
  };

  const confirmDelete = async () => {
    try {
      setDeleting(true);
      const client = createNotificationsClient(createAdminApiClient());
      await client.deleteAdminNotification(deleteModal.id);

      // Update local state immediately
      setNotifications(prev => prev.filter(n => n.id !== deleteModal.id));

      // Update stats
      if (stats) {
        setStats({
          ...stats,
          total: stats.total - 1
        });
      }

      hideDeleteModal();
      toast.success('Notification deleted');
    } catch (err) {
      console.error('Failed to delete notification:', err);
      toast.error('Failed to delete notification');
    } finally {
      setDeleting(false);
    }
  };

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto space-y-8">
        <div className="text-center mb-12">
          <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6 animate-pulse"></div>
          <div className="h-6 bg-muted rounded-full w-64 mx-auto animate-pulse"></div>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-card border border-border rounded-3xl h-32 animate-pulse"></div>
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
              <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent mb-4">
                🔔 Notification Management
              </h1>
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary rounded-full"></div>
            </div>
            <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
              Send and manage system notifications for users
            </p>
          </div>

          {stats && (
            <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
              <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-primary/20">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">📬</div>
                  <span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-primary">{stats.total}</div>
                  <div className="text-xs sm:text-sm text-foreground/80">Notifications</div>
                  <div className="text-xs text-muted-foreground">All time</div>
                </div>
              </div>

              <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-warning/20">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">⚠️</div>
                  <span className="text-xs sm:text-sm font-medium text-muted-foreground">Unread</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-warning">{stats.unread}</div>
                  <div className="text-xs sm:text-sm text-foreground/80">Pending</div>
                  <div className="text-xs text-muted-foreground">Action needed</div>
                </div>
              </div>

              <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-secondary/20">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">📅</div>
                  <span className="text-xs sm:text-sm font-medium text-muted-foreground">Today</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-secondary">{stats.last24Hours}</div>
                  <div className="text-xs sm:text-sm text-foreground/80">Last 24h</div>
                  <div className="text-xs text-muted-foreground">Recent</div>
                </div>
              </div>

              <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-success/20">
                <div className="flex items-center justify-between mb-3 sm:mb-4">
                  <div className="text-xl sm:text-2xl">📊</div>
                  <span className="text-xs sm:text-sm font-medium text-muted-foreground">Week</span>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl sm:text-3xl font-bold text-success">{stats.lastWeek}</div>
                  <div className="text-xs sm:text-sm text-foreground/80">Last 7 days</div>
                  <div className="text-xs text-muted-foreground">Weekly</div>
                </div>
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5 cursor-pointer"
              onClick={loadData}
            >
              <div className="relative bg-primary text-primary-foreground rounded-2xl sm:rounded-3xl transition-opacity hover:opacity-90">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">🔄</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                  <p className="text-primary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload notification data and statistics</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    Refresh
                  </div>
                </div>
              </div>
            </div>

            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5 cursor-pointer"
              onClick={() => router.push('/notifications/analytics')}
            >
              <div className="relative bg-secondary text-secondary-foreground rounded-2xl sm:rounded-3xl transition-opacity hover:opacity-90">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">📈</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">View Analytics</h3>
                  <p className="text-secondary-foreground/80 mb-4 sm:mb-6 text-sm sm:text-base">Detailed notification performance metrics</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    Analytics
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-border/20 p-0.5">
            <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
              <h2 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent mb-6">
                Recent Notifications
              </h2>

              {notifications.length === 0 ? (
                <div className="text-center py-12 sm:py-16">
                  <div className="h-20 w-20 bg-muted rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-4xl">📭</span>
                  </div>
                  <h3 className="text-xl font-semibold text-muted-foreground mb-2">
                    No notifications yet
                  </h3>
                  <p className="text-muted-foreground/60">
                    Start by sending your first notification
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  {notifications.slice(0, 5).map(notification => (
                    <div
                      key={notification.id}
                      className="p-4 bg-muted/30 rounded-2xl border border-border/50"
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="flex-1">
                          <h3 className="font-medium text-foreground mb-1">
                            {notification.title}
                          </h3>
                          <p className="text-sm text-muted-foreground line-clamp-2 mb-2">
                            {notification.message}
                          </p>
                          <div className="flex items-center gap-4 text-xs text-muted-foreground/70">
                            <span>{notification.notification_type}</span>
                            <span>•</span>
                            <span>{new Date(notification.timestamp).toLocaleString()}</span>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <div className={`px-3 py-1 rounded-full text-xs font-semibold ${notification.priority === 'critical' ? 'bg-destructive/10 text-destructive' :
                            notification.priority === 'high' ? 'bg-warning/10 text-warning' :
                              notification.priority === 'normal' ? 'bg-primary/10 text-primary' :
                                'bg-success/10 text-success'
                            }`}>
                            {notification.priority}
                          </div>
                          <button
                            onClick={() => showDeleteModal(notification.id, notification.title)}
                            className="px-3 py-1 bg-destructive hover:bg-destructive/90 text-destructive-foreground text-xs font-semibold rounded-full transition-colors"
                          >
                            Delete
                          </button>
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

      {/* Delete Confirmation Modal */}
      {deleteModal.show && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
          <div className="relative bg-card rounded-3xl p-8 max-w-md w-full shadow-2xl border border-destructive/20">
            <div className="text-center mb-6">
              <div className="w-16 h-16 bg-destructive/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                <span className="text-3xl">🗑️</span>
              </div>
              <h3 className="text-2xl font-bold text-foreground mb-2">
                Delete Notification
              </h3>
              <p className="text-muted-foreground mb-4">
                Are you sure you want to delete this notification?
              </p>
              <div className="bg-muted rounded-xl p-4 mb-6">
                <p className="text-sm font-medium text-foreground line-clamp-2">
                  {deleteModal.title}
                </p>
              </div>
              <p className="text-sm text-destructive font-semibold">
                This action cannot be undone.
              </p>
            </div>

            <div className="flex gap-3">
              <button
                onClick={hideDeleteModal}
                disabled={deleting}
                className="flex-1 px-6 py-3 bg-muted hover:bg-muted/80 text-foreground font-semibold rounded-xl disabled:opacity-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={confirmDelete}
                disabled={deleting}
                className="flex-1 px-6 py-3 bg-destructive hover:bg-destructive/90 text-destructive-foreground font-semibold rounded-xl disabled:opacity-50 transition-colors"
              >
                {deleting ? 'Deleting...' : 'Delete'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
