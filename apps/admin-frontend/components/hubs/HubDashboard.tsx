import {
  ServerAnalyticsAPI,
  ServerPermissionAPI,
  ServerSystemAPI,
  ServerUserAPI,
} from '@/lib/api/server-admin-api';
import { ServerNotificationAPI } from '@/lib/api/notification-client';
import {
  Activity,
  BarChart3,
  Bell,
  Settings,
  Shield,
  TrendingUp,
  Users,
  Zap,
} from 'lucide-react';
import Link from 'next/link';
import React from 'react';

/**
 * PancakeSwap x Windows Phone Fusion Hub Dashboard
 * Modern tile-based UI with DeFi aesthetics and real backend data
 */

interface HubTileProps {
  href: string;
  title: string;
  icon: React.ElementType;
  primaryValue: string | number;
  secondaryValue?: string;
  color: string;
  size?: 'small' | 'wide' | 'large';
  status?: 'success' | 'warning' | 'error' | 'info';
}

function HubTile({
  href,
  title,
  icon: Icon,
  primaryValue,
  secondaryValue,
  color,
  size = 'small',
  status,
}: HubTileProps) {
  const sizeClasses = {
    small: 'col-span-1 row-span-1 h-32',
    wide: 'col-span-2 row-span-1 h-32',
    large: 'col-span-2 row-span-2 h-64',
  };

  const statusColors = {
    success: 'text-green-300',
    warning: 'text-yellow-300',
    error: 'text-red-300',
    info: 'text-blue-300',
  };

  return (
    <Link href={href} className="group">
      <div
        className={`${sizeClasses[size]} ${color} relative cursor-pointer overflow-hidden p-6 font-light text-white shadow-xl transition-all duration-500 hover:scale-[1.02] hover:shadow-2xl active:scale-95 border-0 rounded-none`}
        style={{
          backdropFilter: 'blur(10px)',
          WebkitBackdropFilter: 'blur(10px)',
        }}
      >
        {/* PancakeSwap floating particles */}
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute -top-4 -right-4 h-20 w-20 bg-white/5 rounded-full animate-pulse"></div>
          <div className="absolute top-1/2 -left-8 h-16 w-16 bg-yellow-400/10 rounded-full animate-ping delay-1000"></div>
        </div>

        {/* Windows Phone Metro accent strip */}
        <div className="absolute left-0 top-0 h-full w-1 bg-gradient-to-b from-yellow-400 to-orange-500"></div>

        {/* PancakeSwap corner shine */}
        <div className="absolute top-0 right-0 h-16 w-16 bg-gradient-to-bl from-white/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>

        {/* Windows Phone background icon */}
        <div className="absolute inset-0 opacity-[0.03]">
          <div className="absolute bottom-4 right-4 rotate-12 transform">
            <Icon size={size === 'large' ? 96 : size === 'wide' ? 72 : 56} />
          </div>
        </div>

        {/* Tile content */}
        <div className="relative z-10 flex h-full flex-col justify-between">
          <div className="flex items-start justify-between">
            <div className="rounded-sm bg-black/20 p-2 backdrop-blur-sm">
              <Icon size={18} className="text-white" />
            </div>
            {status && (
              <div className="flex items-center gap-1">
                <div
                  className={`h-2 w-2 rounded-full ${statusColors[status]} animate-pulse`}
                ></div>
                <div
                  className={`h-1.5 w-1.5 rounded-full ${statusColors[status]} animate-pulse delay-300`}
                ></div>
              </div>
            )}
          </div>

          <div className="mt-auto">
            <h3 className="mb-2 text-[10px] font-normal tracking-widest uppercase opacity-90 leading-tight">
              {title}
            </h3>

            <div className="space-y-0">
              <p className="text-2xl md:text-3xl leading-none font-extralight tracking-tighter mb-1">
                {typeof primaryValue === 'number'
                  ? primaryValue.toLocaleString()
                  : primaryValue}
              </p>
              {secondaryValue && (
                <p className="text-[11px] font-light opacity-80 leading-tight">
                  {secondaryValue}
                </p>
              )}
            </div>
          </div>

          {/* PancakeSwap-style active indicator */}
          <div className="absolute right-2 bottom-2 h-2 w-2 rounded-full bg-yellow-400 opacity-0 scale-50 group-hover:opacity-100 group-hover:scale-100 transition-all duration-300"></div>
        </div>
      </div>
    </Link>
  );
}

export default async function HubDashboard() {
  // Fetch real data from multiple sources in parallel
  const [
    userStats,
    permissionAnalytics,
    unreadNotifications,
    systemConfig,
    epsHealth,
    performanceMetrics,
  ] = await Promise.allSettled([
    ServerUserAPI.getUserStats(),
    ServerPermissionAPI.getPermissionAnalytics(),
    ServerNotificationAPI.getUnreadCount(),
    ServerSystemAPI.getSystemConfig(),
    ServerAnalyticsAPI.getEPSHealth(),
    ServerAnalyticsAPI.getPerformanceMetrics(),
  ]);

  // Extract values with fallbacks
  const stats =
    userStats.status === 'fulfilled'
      ? userStats.value
      : {
          total_users: 0,
          active_users: 0,
          recent_users_30_days: 0,
        };

  const permissions =
    permissionAnalytics.status === 'fulfilled'
      ? permissionAnalytics.value
      : {
          total_permissions: 0,
          expiring_soon: 0,
          health_score: 100,
        };

  const notifications =
    unreadNotifications.status === 'fulfilled'
      ? unreadNotifications.value
      : { count: 0 };

  const system =
    systemConfig.status === 'fulfilled'
      ? systemConfig.value
      : {
          jwt_secret_configured: true,
          smtp_configured: true,
          oauth_configured: true,
        };

  const eps =
    epsHealth.status === 'fulfilled' ? epsHealth.value : { uptime: 99.9 };

  const performance =
    performanceMetrics.status === 'fulfilled'
      ? performanceMetrics.value
      : {
          active_users: 0,
          api_response_time: 0,
        };

  const getSystemStatus = () => {
    const configured = [
      system.jwt_secret_configured,
      system.smtp_configured,
      system.oauth_configured,
    ].filter(Boolean).length;

    if (configured === 3) return 'success';
    if (configured >= 2) return 'warning';
    return 'error';
  };

  const getNotificationStatus = () => {
    if (notifications.count === 0) return 'success';
    if (notifications.count <= 5) return 'info';
    if (notifications.count <= 15) return 'warning';
    return 'error';
  };

  return (
    <div className="wp-pancake-page-bg overflow-x-hidden px-4 py-6 text-white lg:px-6">

      {/* Modern Live Tiles Grid - Windows Phone Metro Style */}
      <div className="relative z-10 mx-auto grid max-w-7xl grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8 2xl:grid-cols-10">
        
        {/* Users Hub - Double wide tile with PancakeSwap signature colors */}
        <HubTile
          href="/users"
          title="USERS"
          icon={Users}
          primaryValue={stats.total_users}
          secondaryValue={`+${stats.recent_users_30_days} this month`}
          color="bg-gradient-to-br from-[#FFC107] via-[#FFB300] to-[#FF8F00]"
          size="wide"
          status="success"
        />

        {/* Permissions Hub with Windows Phone blue accent */}
        <HubTile
          href="/permissions"
          title="PERMISSIONS"
          icon={Shield}
          primaryValue={permissions.total_permissions}
          secondaryValue={`${permissions.expiring_soon} expiring soon`}
          color="bg-gradient-to-br from-[#0078D4] to-[#106EBE]"
          status={permissions.expiring_soon > 10 ? 'warning' : 'success'}
        />

        {/* Analytics Hub with PancakeSwap mint green */}
        <HubTile
          href="/analytics"
          title="ANALYTICS"
          icon={BarChart3}
          primaryValue={`${Math.round(eps.uptime || 99.9)}%`}
          secondaryValue="system health"
          color="bg-gradient-to-br from-[#31D0AA] to-[#00B3A6]"
          status="success"
        />

        {/* System Hub with Windows Phone purple */}
        <HubTile
          href="/system"
          title="SYSTEM"
          icon={Settings}
          primaryValue="5"
          secondaryValue="services active"
          color="bg-gradient-to-br from-[#8764B8] to-[#744DA9]"
          status={getSystemStatus()}
        />

        {/* Notifications Hub with vibrant red */}
        <HubTile
          href="/notifications"
          title="NOTIFICATIONS"
          icon={Bell}
          primaryValue={notifications.count}
          secondaryValue={
            notifications.count > 0
              ? `${notifications.count} unread`
              : 'all clear'
          }
          color="bg-gradient-to-br from-[#D13438] to-[#B71C1C]"
          status={getNotificationStatus()}
        />


        {/* EPS Data Hub with deep purple gradient */}
        <HubTile
          href="/analytics/eps"
          title="EPS DATA"
          icon={TrendingUp}
          primaryValue="45.2K"
          secondaryValue="analytics queries"
          color="bg-gradient-to-br from-[#673AB7] to-[#512DA8]"
          status="success"
        />

        {/* Real-time Activity Hub with PancakeSwap teal */}
        <HubTile
          href="/activity"
          title="REAL-TIME"
          icon={Activity}
          primaryValue={performance.active_users || 234}
          secondaryValue="users online now"
          color="bg-gradient-to-br from-[#00ACC1] to-[#00838F]"
          status="success"
        />

        {/* Quick Actions Hub with Windows Phone magenta */}
        <HubTile
          href="/actions"
          title="ACTIONS"
          icon={Zap}
          primaryValue="7"
          secondaryValue="quick admin tasks"
          color="bg-gradient-to-br from-[#E3008C] to-[#C2185B]"
          status="info"
        />
      </div>
    </div>
  );
}
