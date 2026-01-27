'use client';

import {
  Activity,
  Bell,
  Clock,
  FileText,
  Settings,
  Shield,
  Wallet,
  Zap
} from 'lucide-react';
import { useEffect, useState } from 'react';

import { RecentWalletsPanel } from '@/components/admin/RecentWalletsPanel';
import { PageAuthRequired, PageHeader, PageLayout, PageSkeleton } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { APIError, createAdminApiClient } from '@/shared/utils/api-client';

/**
 * Admin Dashboard Client Component
 */
interface DashboardClientProps {
  initialRecentWallets?: any;
}

export default function DashboardClient({ initialRecentWallets }: DashboardClientProps) {
  const { user, isAuthenticated, isLoading } = useSharedAuth();
  const [dashboardStats, setDashboardStats] = useState({
    totalWallets: 0,
    activeWallets: 0,
    totalPermissions: 0,
    systemHealth: 100,
    todayConnections: 0,
    pendingNotifications: 0,
    systemUptime: '99.9%',
    avgResponseTime: '120ms'
  });
  const [accessError, setAccessError] = useState<string | null>(null);

  // Load dashboard data from real backend APIs
  useEffect(() => {
    if (isAuthenticated) {
      const loadDashboardData = async () => {
        try {
          setAccessError(null);
          const client = createAdminApiClient();

          // Helper to catch errors and format them
          const safeFetch = async (promise: Promise<any>) => {
            try {
              return await promise;
            } catch (err: any) {
              if (err instanceof APIError) {
                return { success: false, data: null, status: err.status, error: err.message };
              }
              return { success: false, data: null, status: 0, error: 'Network error or unexpected issue' };
            }
          };

          // Fetch wallet data, permissions, and system stats in parallel
          const [walletsRes, permissionsRes, systemRes] = await Promise.all([
            safeFetch(client.get('/api/admin/wallets/stats')),
            safeFetch(client.get('/api/admin/permissions/system/stats')),
            safeFetch(client.get('/api/admin/permissions/system/health'))
          ]);

          // Update stats from real API responses
          if (walletsRes.success && walletsRes.data) {
            setDashboardStats(prev => ({
              ...prev,
              totalWallets: walletsRes.data.total || 0,
              activeWallets: walletsRes.data.active || 0,
              todayConnections: walletsRes.data.today_connections || 0
            }));
          }

          if (permissionsRes.success && permissionsRes.data) {
            setDashboardStats(prev => ({
              ...prev,
              totalPermissions: permissionsRes.data.total || 0,
              pendingNotifications: permissionsRes.data.pending_notifications || 0
            }));
          }

          if (systemRes.success && systemRes.data) {
            setDashboardStats(prev => ({
              ...prev,
              systemHealth: systemRes.data.health_percentage || 100,
              systemUptime: systemRes.data.uptime || '99.9%',
              avgResponseTime: systemRes.data.avg_response_time || '120ms'
            }));
          }

          // Check for permission errors from any of the requests
          if (walletsRes.status === 403 || walletsRes.status === 401 ||
            permissionsRes.status === 403 || permissionsRes.status === 401 ||
            systemRes.status === 403 || systemRes.status === 401) {
            setAccessError(walletsRes.error || permissionsRes.error || systemRes.error || 'Access denied by backend');
          }
        } catch (err) {
          console.error('Failed to load dashboard data:', err);
        }
      };

      loadDashboardData();
    }
  }, [isAuthenticated]);

  // Show loading state
  if (isLoading) {
    return <PageSkeleton showHeader stats={4} rows={6} />;
  }

  // Show authentication required if not authenticated
  if (!isAuthenticated) {
    return <PageAuthRequired />;
  }

  // Show access error if backend rejected the request
  if (accessError) {
    return (
      <PageLayout>
        <div className="text-center max-w-md mx-auto py-16">
          <div className="w-20 h-20 bg-destructive/10 rounded-3xl flex items-center justify-center mx-auto mb-6">
            <Shield className="w-10 h-10 text-destructive" />
          </div>
          <h1 className="text-2xl sm:text-3xl font-bold text-foreground mb-4">
            Access Denied
          </h1>
          <p className="text-muted-foreground mb-8">
            {accessError}
          </p>
          <a
            href="/auth"
            className="inline-flex items-center gap-2 px-6 py-3 bg-destructive text-destructive-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity"
          >
            Try Again
          </a>
        </div>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      {/* Page Header */}
      <PageHeader
        title="EPSX Admin Center"
        subtitle={`Welcome back, ${user?.wallet_address ? `${user.wallet_address.slice(0, 6)}...${user.wallet_address.slice(-4)}` : 'Admin'}`}
        icon="Home"
        gradient="primary"
        centered
      />

      {/* System Status */}
      <div className="text-center text-sm text-muted-foreground flex items-center justify-center gap-2 -mt-4 mb-6">
        <span>{new Date().toLocaleDateString()}</span>
        <span>•</span>
        <span className="flex items-center gap-1">
          System Status: <span className="text-success font-medium">Operational</span>
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-success/40 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-success"></span>
          </span>
        </span>
      </div>

      {/* Stats Dashboard */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
        {/* Total Wallets */}
        <div className="bg-slate-900/40 backdrop-blur-2xl rounded-[32px] p-6 shadow-xl border border-white/5 relative overflow-hidden group">
          <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" />
          <div className="relative z-10">
            <div className="flex items-center justify-between mb-4">
              <div className="p-3 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[18px] text-[#1fc7d4] border border-[#1fc7d4]/20">
                <Wallet className="w-6 h-6" />
              </div>
              <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Total</span>
            </div>
            <div className="space-y-1">
              <div className="text-3xl sm:text-4xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent">{dashboardStats.totalWallets}</div>
              <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">Wallets</div>
              <div className="text-xs text-muted-foreground font-medium">{dashboardStats.activeWallets} active today</div>
            </div>
          </div>
        </div>

        {/* System Health */}
        <div className="bg-slate-900/40 backdrop-blur-2xl rounded-[32px] p-6 shadow-xl border border-white/5 relative overflow-hidden group">
          <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#31d0aa]/10 rounded-full blur-2xl group-hover:bg-[#31d0aa]/20 transition-colors" />
          <div className="relative z-10">
            <div className="flex items-center justify-between mb-4">
              <div className="p-3 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-[18px] text-[#31d0aa] border border-[#31d0aa]/20">
                <Activity className="w-6 h-6" />
              </div>
              <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Health</span>
            </div>
            <div className="space-y-1">
              <div className="text-3xl sm:text-4xl font-bold text-[#31d0aa]">{dashboardStats.systemHealth}%</div>
              <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">System Health</div>
              <div className="text-xs text-muted-foreground font-medium">{dashboardStats.systemUptime} uptime</div>
            </div>
          </div>
        </div>

        {/* Today's Activity */}
        <div className="bg-slate-900/40 backdrop-blur-2xl rounded-[32px] p-6 shadow-xl border border-white/5 relative overflow-hidden group">
          <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#ed4b9e]/10 rounded-full blur-2xl group-hover:bg-[#ed4b9e]/20 transition-colors" />
          <div className="relative z-10">
            <div className="flex items-center justify-between mb-4">
              <div className="p-3 bg-gradient-to-br from-[#ed4b9e]/10 to-[#7645d9]/10 rounded-[18px] text-[#ed4b9e] border border-[#ed4b9e]/20">
                <Zap className="w-6 h-6" />
              </div>
              <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Today</span>
            </div>
            <div className="space-y-1">
              <div className="text-3xl sm:text-4xl font-bold text-[#ed4b9e]">{dashboardStats.todayConnections}</div>
              <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">Connections</div>
              <div className="text-xs text-muted-foreground font-medium">{dashboardStats.totalPermissions} permissions</div>
            </div>
          </div>
        </div>

        {/* Performance */}
        <div className="bg-slate-900/40 backdrop-blur-2xl rounded-[32px] p-6 shadow-xl border border-white/5 relative overflow-hidden group">
          <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#ffb237]/10 rounded-full blur-2xl group-hover:bg-[#ffb237]/20 transition-colors" />
          <div className="relative z-10">
            <div className="flex items-center justify-between mb-4">
              <div className="p-3 bg-gradient-to-br from-[#ffb237]/10 to-[#ffb237]/10 rounded-[18px] text-[#ffb237] border border-[#ffb237]/20">
                <Clock className="w-6 h-6" />
              </div>
              <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Speed</span>
            </div>
            <div className="space-y-1">
              <div className="text-3xl sm:text-4xl font-bold text-[#ffb237]">{dashboardStats.avgResponseTime}</div>
              <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">Avg Response</div>
              <div className="text-xs text-muted-foreground font-medium">{dashboardStats.pendingNotifications} pending</div>
            </div>
          </div>
        </div>
      </div>

      {/* Recent Wallets Panel */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <RecentWalletsPanel initialData={initialRecentWallets} />
        </div>
        <div className="space-y-4">
          {/* Quick Stats */}
          <div className="bg-slate-900/40 backdrop-blur-2xl rounded-[32px] p-8 shadow-xl border border-white/5 relative overflow-hidden group">
            <div className="absolute -right-6 -top-6 w-20 h-20 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" />
            <h3 className="text-xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent mb-6 flex items-center gap-3">
              <Activity className="w-6 h-6 text-[#1fc7d4]" /> Quick Stats
            </h3>
            <div className="space-y-4">
              <div className="flex justify-between items-center p-3 rounded-2xl bg-white/5 border border-white/5 transition-colors hover:bg-white/10">
                <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">Active Wallets</span>
                <span className="font-bold text-foreground text-lg">{dashboardStats.activeWallets}</span>
              </div>
              <div className="flex justify-between items-center p-3 rounded-2xl bg-white/5 border border-white/5 transition-colors hover:bg-white/10">
                <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">System Health</span>
                <span className="font-bold text-[#31d0aa] text-lg">{dashboardStats.systemHealth}%</span>
              </div>
              <div className="flex justify-between items-center p-3 rounded-2xl bg-white/5 border border-white/5 transition-colors hover:bg-white/10">
                <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">Response Time</span>
                <span className="font-bold text-[#ffb237] text-lg">{dashboardStats.avgResponseTime}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
        {[
          {
            href: "/wallet-management",
            title: "Wallet Management",
            icon: <Wallet className="w-6 h-6" />,
            description: "Manage wallets and permissions",
            gradient: "from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]",
            textGradient: "text-[#1fc7d4]",
            bgGradient: "bg-[#1fc7d4]/5",
            stats: `${dashboardStats.totalWallets} wallets`
          },
          {
            href: "/group-and-permission",
            title: "Permissions",
            icon: <Shield className="w-6 h-6" />,
            description: "Grant and manage access permissions",
            gradient: "from-[#31d0aa] via-[#1fc7d4] to-[#31d0aa]",
            textGradient: "text-[#31d0aa]",
            bgGradient: "bg-[#31d0aa]/5",
            stats: `${dashboardStats.totalPermissions} permissions`
          },
          {
            href: "/audit-log",
            title: "Audit Log",
            icon: <FileText className="w-6 h-6" />,
            description: "Track admin actions and changes",
            gradient: "from-[#ed4b9e] via-[#7645d9] to-[#ed4b9e]",
            textGradient: "text-[#ed4b9e]",
            bgGradient: "bg-[#ed4b9e]/5",
            stats: "View history"
          },
          {
            href: "/notifications",
            title: "Notifications",
            icon: <Bell className="w-6 h-6" />,
            description: "Send notifications and manage alerts",
            gradient: "from-[#7645d9] via-[#ed4b9e] to-[#7645d9]",
            textGradient: "text-[#7645d9]",
            bgGradient: "bg-[#7645d9]/5",
            stats: `${dashboardStats.pendingNotifications} pending`
          },
          {
            href: "/settings",
            title: "Settings",
            icon: <Settings className="w-6 h-6" />,
            description: "Configure system and user settings",
            gradient: "from-slate-400 via-slate-500 to-slate-400",
            textGradient: "text-slate-400",
            bgGradient: "bg-slate-500/5",
            stats: "System config"
          },
          {
            href: "/developer-portal",
            title: "Developer",
            icon: <Zap className="w-6 h-6" />,
            description: "API documentation and developer tools",
            gradient: "from-[#ffb237] via-[#1fc7d4] to-[#ffb237]",
            textGradient: "text-[#ffb237]",
            bgGradient: "bg-[#ffb237]/5",
            stats: "API & Tools"
          }
        ].map((action) => (
          <a
            key={action.href}
            href={action.href}
            className="block group active:scale-[0.98] transition-all"
          >
            <div className={`relative overflow-hidden rounded-[32px] ${action.bgGradient} p-0.5 border border-white/5 hover:border-[#1fc7d4]/30 transition-colors bg-slate-900/40 backdrop-blur-xl`}>
              <div className="relative p-6 sm:p-8">
                {/* Floating decoration */}
                <div className={`absolute -top-4 -right-4 w-16 h-16 bg-gradient-to-r ${action.gradient} rounded-full blur-2xl opacity-10 group-hover:opacity-20 transition-opacity`}></div>

                <div className="relative z-10">
                  <div className="flex items-center gap-4 mb-4">
                    <div className={`p-3 rounded-2xl bg-white/5 border border-white/5 ${action.textGradient}`}>
                      {action.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className={`text-xl sm:text-2xl font-bold bg-gradient-to-r ${action.gradient} bg-clip-text text-transparent truncate`}>
                        {action.title}
                      </h3>
                      <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest mt-0.5">
                        {action.stats}
                      </div>
                    </div>
                  </div>

                  <p className="text-sm text-muted-foreground mb-8 line-clamp-2 font-medium">
                    {action.description}
                  </p>

                  <div className="flex items-center justify-between">
                    <div className={`px-5 py-2 bg-[#1fc7d4] text-white rounded-2xl text-xs font-bold shadow-lg shadow-cyan-500/10 group-hover:shadow-cyan-500/30 transition-all`}>
                      Open Tool
                    </div>
                    <div className="w-8 h-8 rounded-full bg-white/5 border border-white/5 flex items-center justify-center text-muted-foreground group-hover:text-[#1fc7d4] transition-colors">
                      →
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </a>
        ))}
      </div>
    </PageLayout>
  );
}
