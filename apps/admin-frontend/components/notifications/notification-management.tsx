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
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

import { deleteNotificationAction, getNotificationOverviewAction } from '@/app/notifications/actions';
import { StatsCard } from '@/components/admin/developer-portal/shared/stats-card';
import { Button } from '@/components/ui/button';

interface Notification {
  id: string;
  title?: string;
  message?: string;
  priority?: string;
  type?: string;
  notification_type?: string;
  timestamp?: string;
}

interface NotificationStats {
  total: number;
  unread?: number;
  sentToday: number;
  sentThisWeek: number;
  byType?: Record<string, number>;
  byPriority?: Record<string, number>;
}

function getPriorityColor(priority: string | undefined): string {
  switch (priority) {
    case 'critical': return 'bg-red-500/10 text-red-400 border-red-500/20';
    case 'high': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
    case 'normal': return 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20';
    default: return 'bg-slate-500/10 text-slate-400 border-slate-500/20';
  }
}

function getTypeIcon(notificationType: string | undefined) {
  if (notificationType === 'security') { return <Shield className="w-6 h-6" />; }
  if (notificationType === 'system') { return <RefreshCw className="w-6 h-6" />; }
  return <MessageSquare className="w-6 h-6" />;
}

function formatNotifTimestamp(ts: string | undefined): string {
  if (ts === undefined) { return ''; }
  return new Date(ts).toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function parseStats(data: Record<string, unknown>): NotificationStats {
  return {
    total: typeof data.total_notifications === 'number' ? data.total_notifications : 0,
    unread: 0,
    sentToday: typeof data.sent_today === 'number' ? data.sent_today : 0,
    sentThisWeek: typeof data.sent_this_week === 'number' ? data.sent_this_week : 0,
    byType: typeof data.by_type === 'object' && data.by_type !== null ? data.by_type as Record<string, number> : {},
    byPriority: typeof data.by_priority === 'object' && data.by_priority !== null ? data.by_priority as Record<string, number> : {},
  };
}

interface StatsGridProps {
  stats: NotificationStats;
}

function StatsGrid({ stats }: StatsGridProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <StatsCard title="Total Sent" value={stats.total} icon={Layers} iconBgColor="bg-cyan-500" iconColor="text-cyan-400" />
      <StatsCard title="Today's Pulse" value={stats.sentToday} icon={Clock} iconBgColor="bg-amber-500" iconColor="text-amber-400" />
      <StatsCard title="Weekly Volume" value={stats.sentThisWeek} icon={Calendar} iconBgColor="bg-purple-500" iconColor="text-purple-400" />
      <StatsCard title="System Health" value="Stable" icon={Shield} iconBgColor="bg-green-500" iconColor="text-green-400" />
    </div>
  );
}

interface NotificationRowProps {
  notification: Notification;
  onDelete: (id: string, title: string) => void;
}

function NotificationRow({ notification, onDelete }: NotificationRowProps) {
  return (
    <div className="group p-8 flex items-start gap-8 hover:bg-gray-50 dark:hover:bg-white/[0.02] transition-colors">
      <div className="flex-shrink-0 w-14 h-14 rounded-xl bg-muted/30 border border-border/40 flex items-center justify-center text-muted-foreground">
        {getTypeIcon(notification.notification_type)}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex flex-wrap items-center gap-3 mb-2">
          <h3 className="text-lg font-black text-foreground tracking-tight truncate">{notification.title}</h3>
          <span className={`px-3 py-1 rounded-lg text-[10px] font-black uppercase tracking-widest border ${getPriorityColor(notification.priority)}`}>
            {notification.priority}
          </span>
        </div>
        <p className="text-base font-bold text-muted-foreground mb-4 line-clamp-2 max-w-4xl">{notification.message}</p>
        <div className="flex items-center space-x-6 text-[10px] font-black uppercase tracking-widest text-muted-foreground/40">
          <span className="flex items-center"><Layers className="w-3 h-3 mr-2" />{notification.notification_type}</span>
          <span className="flex items-center"><Clock className="w-3 h-3 mr-2" />{formatNotifTimestamp(notification.timestamp)}</span>
        </div>
      </div>
      <div className="opacity-0 group-hover:opacity-100 transition-opacity">
        <Button
          onClick={() => onDelete(notification.id, notification.title ?? '')}
          variant="ghost"
          className="h-12 w-12 p-0 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-2xl border border-transparent hover:border-red-500/20 active:scale-95 transition-all"
        >
          <Trash2 className="w-6 h-6" />
        </Button>
      </div>
    </div>
  );
}

interface ActionButtonsProps {
  onRefresh: () => void;
  onAnalytics: () => void;
}

function ActionButtons({ onRefresh, onAnalytics }: ActionButtonsProps) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
      <button onClick={onRefresh} className="group relative overflow-hidden rounded-xl bg-card border border-border/20 transition-all hover:border-[#1fc7d4]/30 shadow-xl">
        <div className="relative p-8 flex items-center justify-between transition-colors group-hover:bg-muted/20">
          <div className="flex items-center space-x-6">
            <div className="w-16 h-16 flex items-center justify-center bg-cyan-500/10 rounded-2xl border border-cyan-500/20 text-[#1fc7d4] transition-transform group-hover:rotate-180 duration-500">
              <RefreshCw className="w-8 h-8" />
            </div>
            <div className="text-left">
              <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-1">Synchronize</h3>
              <p className="text-sm font-bold text-muted-foreground">Refresh real-time telemetry</p>
            </div>
          </div>
          <ChevronRight className="w-6 h-6 text-muted-foreground/30 group-hover:text-[#1fc7d4] group-hover:translate-x-1 transition-all" />
        </div>
      </button>
      <button onClick={onAnalytics} className="group relative overflow-hidden rounded-xl bg-card border border-border/20 transition-all hover:border-purple-500/30 shadow-xl">
        <div className="relative p-8 flex items-center justify-between transition-colors group-hover:bg-muted/20">
          <div className="flex items-center space-x-6">
            <div className="w-16 h-16 flex items-center justify-center bg-purple-500/10 rounded-2xl border border-purple-500/20 text-purple-400 transition-transform group-hover:scale-110">
              <BarChart2 className="w-8 h-8" />
            </div>
            <div className="text-left">
              <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-1">Analytics</h3>
              <p className="text-sm font-bold text-muted-foreground">Deep dive performance metrics</p>
            </div>
          </div>
          <ChevronRight className="w-6 h-6 text-muted-foreground/30 group-hover:text-purple-400 group-hover:translate-x-1 transition-all" />
        </div>
      </button>
    </div>
  );
}

interface NotifTableProps {
  notifications: Notification[];
  onDelete: (id: string, title: string) => void;
}

function NotifTable({ notifications, onDelete }: NotifTableProps) {
  return (
    <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
      <div className="h-[3px] bg-gradient-to-r from-[#ffb237] to-[#7645d9]" />
      <div className="flex items-center justify-between px-4 py-3 border-b border-border/20">
        <h2 className="text-xs font-bold text-[#ffb237] uppercase tracking-[0.2em]">RECENT BROADCASTS</h2>
        <div className="relative hidden sm:block">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground/50" />
          <input type="text" placeholder="Filter events..." className="bg-muted/30 border border-border/40 rounded-xl pl-10 pr-4 py-2 text-xs font-bold focus:outline-none focus:border-[#1fc7d4]/50 transition-colors" />
        </div>
      </div>
      <div className="divide-y divide-white/5">
        {notifications.length === 0 ? (
          <div className="py-24 text-center">
            <div className="inline-flex p-6 bg-muted/30 rounded-xl mb-6"><Bell className="w-12 h-12 text-muted-foreground/20" /></div>
            <h3 className="text-xl font-black text-muted-foreground uppercase tracking-tight">Silence is Golden</h3>
            <p className="text-sm font-bold text-muted-foreground/50 mt-2">No active notifications detected in the grid</p>
          </div>
        ) : (
          notifications.map(n => (
            <NotificationRow key={n.id} notification={n} onDelete={onDelete} />
          ))
        )}
      </div>
    </div>
  );
}

interface DeleteModalProps {
  title: string;
  deleting: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

function DeleteModal({ title, deleting, onConfirm, onCancel }: DeleteModalProps) {
  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-background/80 animate-in fade-in duration-300">
      <div className="rounded-2xl border border-red-500/20 bg-card max-w-md w-full shadow-2xl overflow-hidden">
        <div className="p-10 text-center">
          <div className="w-20 h-20 bg-red-500/10 rounded-[28px] flex items-center justify-center mx-auto mb-8 border border-red-500/20">
            <AlertTriangle className="w-10 h-10 text-red-500" />
          </div>
          <h3 className="text-3xl font-black text-foreground uppercase tracking-tight mb-4">Confirm Deletion</h3>
          <p className="text-base font-bold text-muted-foreground mb-8">
            This broadcast will be permanently purged from the system grid.
          </p>
          <div className="bg-muted/30 border border-border/40 rounded-xl p-6 mb-10">
            <p className="text-sm font-black text-foreground line-clamp-2">{title}</p>
          </div>
          <div className="flex gap-4">
            <Button onClick={onCancel} disabled={deleting} className="flex-1 py-7 rounded-xl bg-muted/30 border border-border/40 text-foreground font-black uppercase tracking-widest hover:bg-muted/50 transition-all">
              Abort
            </Button>
            <Button onClick={onConfirm} disabled={deleting} className="flex-1 py-7 rounded-2xl bg-red-500 hover:bg-red-600 text-white font-black uppercase tracking-widest shadow-lg shadow-red-500/20 active:scale-95 transition-all">
              {deleting ? 'Purging...' : 'Purge'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

export function NotificationManagement() {
  const router = useRouter();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [stats, setStats] = useState<NotificationStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [deleteModal, setDeleteModal] = useState<{ show: boolean; id: string; title: string }>({ show: false, id: '', title: '' });
  const [deleting, setDeleting] = useState(false);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      const res = await getNotificationOverviewAction(20);
      if (res.success === true) {
        const { notifications: notifs, stats: rawStats } = res.data;
        setNotifications(notifs as Notification[]);
        if (rawStats !== null && rawStats !== undefined) {
          setStats(parseStats(rawStats as Record<string, unknown>));
        }
      } else {
        toast.error('Failed to load notification data');
      }
    } catch {
      toast.error('Failed to load notification data');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void loadData(); }, [loadData]);

  const confirmDelete = useCallback(async () => {
    try {
      setDeleting(true);
      const result = await deleteNotificationAction(deleteModal.id);
      if (result.success === true) {
        setNotifications(prev => prev.filter(n => n.id !== deleteModal.id));
        if (stats !== null) { setStats({ ...stats, total: stats.total - 1 }); }
        setDeleteModal({ show: false, id: '', title: '' });
        toast.success('Notification deleted successfully');
      } else {
        toast.error('Failed to delete notification');
      }
    } catch {
      toast.error('Failed to delete notification');
    } finally {
      setDeleting(false);
    }
  }, [deleteModal.id, stats]);

  if (loading) {
    return (
      <div className="space-y-8 animate-in fade-in duration-500">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {Array.from({ length: 4 }, (_, i) => `notif-skel-${i}`).map((key) => (
            <div key={key} className="h-32 rounded-xl bg-card border border-border/20 animate-pulse" />
          ))}
        </div>
        <div className="h-96 rounded-2xl bg-card border border-border/20 animate-pulse" />
      </div>
    );
  }

  return (
    <div className="space-y-10">
      {stats !== null && <StatsGrid stats={stats} />}
      <ActionButtons onRefresh={() => void loadData()} onAnalytics={() => router.push('/notifications/analytics')} />
      <NotifTable notifications={notifications} onDelete={(id, title) => setDeleteModal({ show: true, id, title })} />

      {deleteModal.show && (
        <DeleteModal
          title={deleteModal.title}
          deleting={deleting}
          onConfirm={() => { void confirmDelete(); }}
          onCancel={() => setDeleteModal({ show: false, id: '', title: '' })}
        />
      )}
    </div>
  );
}
