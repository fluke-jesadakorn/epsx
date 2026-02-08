'use client';

import {
  AlertTriangle,
  BarChart2,
  Bell,
  Calendar,
  ChevronRight,
  Clock,
  Layers,
  MessageSquare,
  RefreshCw,
  Search,
  Shield,
  Trash2
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { deleteNotificationAction, getNotificationsAction, getNotificationStatsAction } from '@/app/notifications/actions';
import { StatsCard } from '@/components/admin/developer-portal/shared/stats-card';
import { Button } from '@/components/ui/button';

type Notification = any;
type NotificationStats = any;

interface NotificationManagementProps {
  currentUser: any;
}

/**
 * Modernized Notification Management with PancakeSwap aesthetic
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
      const [notificationsResponse, statsResponse] = await Promise.all([
        getNotificationsAction(1, 20),
        getNotificationStatsAction()
      ]);

      if (notificationsResponse.success) {
        setNotifications(notificationsResponse.data?.notifications || []);
      } else {
        console.error('Failed to load notifications:', notificationsResponse.error);
        toast.error('Failed to load notifications');
      }

      if (statsResponse.success && statsResponse.data) {
        const backendStats = statsResponse.data;
        setStats({
          total: backendStats.total_notifications,
          unread: 0,
          sentToday: backendStats.sent_today,
          sentThisWeek: backendStats.sent_this_week,
          byType: backendStats.by_type || {},
          byPriority: backendStats.by_priority || {},
        });
      }
    } catch (err) {
      console.error('Failed to load notifications:', err);
      toast.error('Failed to load notification data');
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
      const result = await deleteNotificationAction(deleteModal.id);

      if (result.success) {
        setNotifications(prev => prev.filter(n => n.id !== deleteModal.id));

        if (stats) {
          setStats({
            ...stats,
            total: stats.total - 1
          });
        }

        hideDeleteModal();
        toast.success('Notification deleted successfully');
      } else {
        console.error('Failed to delete notification:', result.error);
        toast.error('Failed to delete notification');
      }
    } catch (err) {
      console.error('Failed to delete notification:', err);
      toast.error('Failed to delete notification');
    } finally {
      setDeleting(false);
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical': return 'bg-red-500/10 text-red-400 border-red-500/20';
      case 'high': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
      case 'normal': return 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20';
      default: return 'bg-slate-500/10 text-slate-400 border-slate-500/20';
    }
  };

  if (loading) {
    return (
      <div className="space-y-8 animate-in fade-in duration-500">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="h-32 rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 animate-pulse" />
          ))}
        </div>
        <div className="h-96 rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 animate-pulse" />
      </div>
    );
  }

  return (
    <div className="space-y-10">
      {/* Stats Section */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <StatsCard
            title="Total Sent"
            value={stats.total}
            icon={Layers}
            iconBgColor="bg-cyan-500"
            iconColor="text-cyan-400"
          />
          <StatsCard
            title="Today's Pulse"
            value={stats.sentToday}
            icon={Clock}
            iconBgColor="bg-amber-500"
            iconColor="text-amber-400"
          />
          <StatsCard
            title="Weekly Volume"
            value={stats.sentThisWeek}
            icon={Calendar}
            iconBgColor="bg-purple-500"
            iconColor="text-purple-400"
          />
          <StatsCard
            title="System Health"
            value="Stable"
            icon={Shield}
            iconBgColor="bg-green-500"
            iconColor="text-green-400"
          />
        </div>
      )}

      {/* Action Buttons */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <button
          onClick={loadData}
          className="group relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-1 transition-all hover:border-[#1fc7d4]/30 shadow-xl"
        >
          <div className="relative bg-card rounded-[28px] p-8 flex items-center justify-between transition-colors group-hover:bg-white/[0.02]">
            <div className="flex items-center space-x-6">
              <div className="w-16 h-16 flex items-center justify-center bg-cyan-500/10 rounded-2xl border border-cyan-500/20 text-[#1fc7d4] transition-transform group-hover:rotate-180 duration-500">
                <RefreshCw className="w-8 h-8" />
              </div>
              <div className="text-left">
                <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-1">
                  Synchronize
                </h3>
                <p className="text-sm font-bold text-muted-foreground">Refresh real-time telemetry</p>
              </div>
            </div>
            <ChevronRight className="w-6 h-6 text-muted-foreground/30 group-hover:text-[#1fc7d4] group-hover:translate-x-1 transition-all" />
          </div>
        </button>

        <button
          onClick={() => router.push('/notifications/analytics')}
          className="group relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-1 transition-all hover:border-purple-500/30 shadow-xl"
        >
          <div className="relative bg-card rounded-[28px] p-8 flex items-center justify-between transition-colors group-hover:bg-white/[0.02]">
            <div className="flex items-center space-x-6">
              <div className="w-16 h-16 flex items-center justify-center bg-purple-500/10 rounded-2xl border border-purple-500/20 text-purple-400 transition-transform group-hover:scale-110">
                <BarChart2 className="w-8 h-8" />
              </div>
              <div className="text-left">
                <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-1">
                  Analytics
                </h3>
                <p className="text-sm font-bold text-muted-foreground">Deep dive performance metrics</p>
              </div>
            </div>
            <ChevronRight className="w-6 h-6 text-muted-foreground/30 group-hover:text-purple-400 group-hover:translate-x-1 transition-all" />
          </div>
        </button>
      </div>

      {/* Table Section */}
      <div className="relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-xl">
        <div className="p-8 border-b border-white/5 flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-black text-foreground uppercase tracking-tight mb-2">
              Recent Broadcasts
            </h2>
            <p className="text-sm font-bold text-muted-foreground">Monitoring the latest system communications</p>
          </div>
          <div className="relative hidden sm:block">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground/50" />
            <input
              type="text"
              placeholder="Filter events..."
              className="bg-white/5 border border-white/10 rounded-xl pl-10 pr-4 py-2 text-xs font-bold focus:outline-none focus:border-[#1fc7d4]/50 transition-colors"
            />
          </div>
        </div>

        <div className="divide-y divide-white/5">
          {notifications.length === 0 ? (
            <div className="py-24 text-center">
              <div className="inline-flex p-6 bg-white/5 rounded-[32px] mb-6">
                <Bell className="w-12 h-12 text-muted-foreground/20" />
              </div>
              <h3 className="text-xl font-black text-muted-foreground uppercase tracking-tight">
                Silence is Golden
              </h3>
              <p className="text-sm font-bold text-muted-foreground/50 mt-2">No active notifications detected in the grid</p>
            </div>
          ) : (
            notifications.map(notification => (
              <div
                key={notification.id}
                className="group p-8 flex items-start gap-8 hover:bg-white/[0.02] transition-colors"
              >
                <div className="flex-shrink-0 w-14 h-14 rounded-2xl bg-white/5 border border-white/5 flex items-center justify-center text-muted-foreground shadow-inner">
                  {notification.notification_type === 'security' ? <Shield className="w-6 h-6" /> :
                    notification.notification_type === 'system' ? <RefreshCw className="w-6 h-6" /> :
                      <MessageSquare className="w-6 h-6" />}
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex flex-wrap items-center gap-3 mb-2">
                    <h3 className="text-lg font-black text-foreground tracking-tight truncate">
                      {notification.title}
                    </h3>
                    <span className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase tracking-widest border ${getPriorityColor(notification.priority)}`}>
                      {notification.priority}
                    </span>
                  </div>
                  <p className="text-base font-bold text-muted-foreground mb-4 line-clamp-2 max-w-4xl">
                    {notification.message}
                  </p>
                  <div className="flex items-center space-x-6 text-[10px] font-black uppercase tracking-widest text-muted-foreground/40">
                    <span className="flex items-center">
                      <Layers className="w-3 h-3 mr-2" />
                      {notification.notification_type}
                    </span>
                    <span className="flex items-center">
                      <Clock className="w-3 h-3 mr-2" />
                      {new Date(notification.timestamp).toLocaleString(undefined, {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit'
                      })}
                    </span>
                  </div>
                </div>

                <div className="opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button
                    onClick={() => showDeleteModal(notification.id, notification.title)}
                    variant="ghost"
                    className="h-12 w-12 p-0 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-2xl border border-transparent hover:border-red-500/20 active:scale-95 transition-all"
                  >
                    <Trash2 className="w-6 h-6" />
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Delete Confirmation Modal */}
      {deleteModal.show && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-xl animate-in fade-in duration-300">
          <div className="relative overflow-hidden rounded-[40px] bg-slate-900/60 border border-red-500/20 p-1 max-w-md w-full shadow-2xl">
            <div className="bg-card rounded-[38px] p-10 text-center">
              <div className="w-20 h-20 bg-red-500/10 rounded-[28px] flex items-center justify-center mx-auto mb-8 border border-red-500/20">
                <AlertTriangle className="w-10 h-10 text-red-500" />
              </div>
              <h3 className="text-3xl font-black text-foreground uppercase tracking-tight mb-4">
                Confirm Deletion
              </h3>
              <p className="text-base font-bold text-muted-foreground mb-8">
                This broadcast will be permanently purged from the system grid.
              </p>

              <div className="bg-white/5 border border-white/5 rounded-2xl p-6 mb-10">
                <p className="text-sm font-black text-foreground line-clamp-2">
                  {deleteModal.title}
                </p>
              </div>

              <div className="flex gap-4">
                <Button
                  onClick={hideDeleteModal}
                  disabled={deleting}
                  className="flex-1 py-7 rounded-2xl bg-white/5 border border-white/10 text-foreground font-black uppercase tracking-widest hover:bg-white/10 transition-all"
                >
                  Abort
                </Button>
                <Button
                  onClick={confirmDelete}
                  disabled={deleting}
                  className="flex-1 py-7 rounded-2xl bg-red-500 hover:bg-red-600 text-white font-black uppercase tracking-widest shadow-lg shadow-red-500/20 active:scale-95 transition-all"
                >
                  {deleting ? 'Purging...' : 'Purge'}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
