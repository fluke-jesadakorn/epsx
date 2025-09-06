/**
 * Server-rendered Hub Dashboard
 * Uses server component data fetching with hybrid client interactions
 */

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
import type { DashboardData } from '@/lib/server/unified-data-fetchers';
import type { AdminSession } from '@/lib/server/auth-helpers';

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

interface HubDashboardServerProps {
  session: AdminSession;
  dashboardData: DashboardData;
}

export default function HubDashboardServer({ session, dashboardData }: HubDashboardServerProps) {
  const { stats, permissionAnalytics, systemMetrics } = dashboardData;

  const getSystemStatus = () => {
    // Simple system health based on metrics
    if (systemMetrics.api_response_time < 2 && systemMetrics.memory_usage < 80) return 'success';
    if (systemMetrics.api_response_time < 5 && systemMetrics.memory_usage < 90) return 'warning';
    return 'error';
  };

  const getNotificationStatus = () => {
    // For now, we'll show success - this can be enhanced with actual notification count
    return 'success';
  };

  const getPermissionStatus = () => {
    if (permissionAnalytics.expiring_soon === 0) return 'success';
    if (permissionAnalytics.expiring_soon <= 10) return 'warning';
    return 'error';
  };

  return (
    <div className="wp-pancake-page-bg overflow-x-hidden px-4 py-6 text-white lg:px-6">
      {/* Welcome Message */}
      <div className="mb-6 text-center">
        <h1 className="text-2xl font-light text-white/90">
          Welcome back, {session.user?.name || session.user?.email}
        </h1>
        <p className="text-sm text-white/60">
          Admin Dashboard • {new Date().toLocaleDateString()}
        </p>
      </div>

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
          primaryValue={permissionAnalytics.total_permissions}
          secondaryValue={`${permissionAnalytics.expiring_soon} expiring soon`}
          color="bg-gradient-to-br from-[#0078D4] to-[#106EBE]"
          status={getPermissionStatus()}
        />

        {/* Analytics Hub with PancakeSwap mint green */}
        <HubTile
          href="/analytics"
          title="ANALYTICS"
          icon={BarChart3}
          primaryValue={`${Math.round(permissionAnalytics.health_score)}%`}
          secondaryValue="system health"
          color="bg-gradient-to-br from-[#31D0AA] to-[#00B3A6]"
          status="success"
        />

        {/* System Hub with Windows Phone purple */}
        <HubTile
          href="/system"
          title="SYSTEM"
          icon={Settings}
          primaryValue={`${systemMetrics.memory_usage}%`}
          secondaryValue="memory usage"
          color="bg-gradient-to-br from-[#8764B8] to-[#744DA9]"
          status={getSystemStatus()}
        />

        {/* Notifications Hub with vibrant red */}
        <HubTile
          href="/notifications"
          title="NOTIFICATIONS"
          icon={Bell}
          primaryValue="0"
          secondaryValue="all clear"
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
          primaryValue={systemMetrics.active_users}
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

      {/* Server-rendered Stats Summary */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-4 max-w-4xl mx-auto">
        <div className="bg-black/20 backdrop-blur-sm rounded-lg p-4 text-center">
          <div className="text-2xl font-light text-yellow-400">{stats.active_users}</div>
          <div className="text-sm text-white/60">Active Users</div>
        </div>
        <div className="bg-black/20 backdrop-blur-sm rounded-lg p-4 text-center">
          <div className="text-2xl font-light text-green-400">{permissionAnalytics.users_with_permissions}</div>
          <div className="text-sm text-white/60">Users with Permissions</div>
        </div>
        <div className="bg-black/20 backdrop-blur-sm rounded-lg p-4 text-center">
          <div className="text-2xl font-light text-blue-400">{systemMetrics.api_response_time}s</div>
          <div className="text-sm text-white/60">Avg Response Time</div>
        </div>
      </div>
    </div>
  );
}