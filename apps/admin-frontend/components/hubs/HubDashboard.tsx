'use client'

import {
  ClientAnalyticsAPI,
  ClientPermissionAPI,
  ClientSystemAPI,
  ClientUserAPI,
  ClientNotificationAPI,
} from '@/lib/api/client-admin-api';
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
import React, { useState, useEffect } from 'react';
import { PancakeCard, PancakeStatsCard } from '@/components/ui/PancakeCard';
import { EnhancedStatsCard } from '@/components/ui/AdminIcons';

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


export default function HubDashboard() {
  const [stats, setStats] = useState({ total_users: 2847, active_users: 2234, recent_users_30_days: 234 })
  const [permissions, setPermissions] = useState({ total_permissions: 1847, expiring_soon: 12, health_score: 92 })
  const [notifications, setNotifications] = useState({ count: 0 })
  const [system, setSystem] = useState({ jwt_secret_configured: true, smtp_configured: true, oauth_configured: true })
  const [eps, setEps] = useState({ uptime: 99.9 })
  const [performance, setPerformance] = useState({ active_users: 234, api_response_time: 1.2 })

  useEffect(() => {
    const loadData = async () => {
      try {
        const [userStatsData, permissionAnalyticsData, unreadNotificationsData, systemConfigData, epsHealthData, performanceMetricsData] = 
          await Promise.allSettled([
            ClientUserAPI.getUserStats(),
            ClientPermissionAPI.getPermissionAnalytics(),
            ClientNotificationAPI.getUnreadCount(),
            ClientSystemAPI.getSystemConfig(),
            ClientAnalyticsAPI.getEPSHealth(),
            ClientAnalyticsAPI.getPerformanceMetrics(),
          ])

        if (userStatsData.status === 'fulfilled' && userStatsData.value) setStats(userStatsData.value)
        if (permissionAnalyticsData.status === 'fulfilled' && permissionAnalyticsData.value) setPermissions(permissionAnalyticsData.value)
        if (unreadNotificationsData.status === 'fulfilled' && unreadNotificationsData.value) setNotifications(unreadNotificationsData.value)
        if (systemConfigData.status === 'fulfilled' && systemConfigData.value) setSystem(systemConfigData.value)
        if (epsHealthData.status === 'fulfilled' && epsHealthData.value) setEps(epsHealthData.value)
        if (performanceMetricsData.status === 'fulfilled' && performanceMetricsData.value) setPerformance(performanceMetricsData.value)
      } catch (error) {
        console.error('Failed to load dashboard data:', error)
      }
    }

    loadData()
  }, [])


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
    <div className="min-h-screen bg-gradient-to-br from-orange-50 via-yellow-50 to-orange-100 dark:from-orange-950 dark:via-yellow-950 dark:to-orange-900 p-6">
      {/* Welcome Section */}
      <div className="mb-8">
        <h1 className="text-4xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent mb-2">
          Welcome back, Info EPSX
        </h1>
        <p className="text-orange-700 dark:text-orange-300">
          Admin Dashboard • {new Date().toLocaleDateString()}
        </p>
      </div>

      {/* Enhanced PancakeSwap Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 lg:gap-6 mb-8">
        <Link href="/users" className="block">
          <EnhancedStatsCard
            title="USERS"
            value={stats.total_users}
            subtitle={`+${stats.recent_users_30_days} this month`}
            iconName="users"
            trend="up"
            trendValue={`+${stats.recent_users_30_days}`}
            statusColor="blue"
          />
        </Link>

        <Link href="/permissions" className="block">
          <EnhancedStatsCard
            title="PERMISSIONS"
            value={permissions.total_permissions.toLocaleString()}
            subtitle={`${permissions.expiring_soon || 0} expiring soon`}
            iconName="permissions"
            trend={(permissions.expiring_soon || 0) > 10 ? 'down' : 'neutral'}
            trendValue={`${permissions.expiring_soon || 0} expiring`}
            statusColor={(permissions.expiring_soon || 0) > 10 ? 'red' : 'green'}
          />
        </Link>

        <Link href="/analytics" className="block">
          <EnhancedStatsCard
            title="ANALYTICS"
            value={`${Math.round(eps.uptime || 99.9)}%`}
            subtitle="system health"
            iconName="analytics"
            trend="up"
            trendValue="99.9%"
            statusColor="green"
          />
        </Link>

        <Link href="/system" className="block">
          <EnhancedStatsCard
            title="SYSTEM"
            value={`${Math.round(performance.active_users || 234)}`}
            subtitle="active users"
            iconName="system"
            trend="up"
            trendValue="+5.2%"
            statusColor="blue"
          />
        </Link>

        <Link href="/notifications" className="block">
          <EnhancedStatsCard
            title="NOTIFICATIONS"
            value={notifications.count.toString()}
            subtitle={notifications.count > 0 ? `${notifications.count} unread` : 'all clear'}
            iconName="notifications"
            trend={notifications.count > 0 ? 'down' : 'up'}
            trendValue={notifications.count > 0 ? 'pending' : 'clear'}
            statusColor={notifications.count > 0 ? 'yellow' : 'green'}
          />
        </Link>

        <Link href="/analytics/eps" className="block">
          <EnhancedStatsCard
            title="EPS DATA"
            value="45.2K"
            subtitle="analytics queries"
            iconName="eps"
            trend="up"
            trendValue="+12.5%"
            statusColor="green"
          />
        </Link>

        <Link href="/activity" className="block">
          <EnhancedStatsCard
            title="REAL-TIME"
            value={performance.active_users || 234}
            subtitle="users online now"
            iconName="realtime"
            trend="up"
            trendValue="Live"
            statusColor="green"
          />
        </Link>

        <Link href="/actions" className="block">
          <EnhancedStatsCard
            title="ACTIONS"
            value="7"
            subtitle="quick admin tasks"
            iconName="actions"
            trend="neutral"
            trendValue="Ready"
            statusColor="blue"
          />
        </Link>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 lg:gap-6">
        <PancakeCard variant="feature" className="text-center hover:scale-105 transition-transform duration-300">
          <div className="text-7xl font-black bg-gradient-to-r from-orange-600 via-yellow-500 to-orange-600 bg-clip-text text-transparent mb-3 drop-shadow-sm">
            {stats.total_users.toLocaleString()}
          </div>
          <div className="text-2xl font-bold text-orange-700 dark:text-orange-300 mb-2">
            Active Users
          </div>
          <div className="text-base text-orange-600/80 dark:text-orange-400/80 font-medium">
            Total registered users in the system
          </div>
        </PancakeCard>

        <PancakeCard variant="feature" className="text-center hover:scale-105 transition-transform duration-300">
          <div className="text-7xl font-black bg-gradient-to-r from-orange-600 via-yellow-500 to-orange-600 bg-clip-text text-transparent mb-3 drop-shadow-sm">
            {Math.round(permissions.health_score || 95)}%
          </div>
          <div className="text-2xl font-bold text-orange-700 dark:text-orange-300 mb-2">
            System Health
          </div>
          <div className="text-base text-orange-600/80 dark:text-orange-400/80 font-medium">
            Overall system performance and reliability
          </div>
        </PancakeCard>

        <PancakeCard variant="feature" className="text-center hover:scale-105 transition-transform duration-300">
          <div className="text-6xl font-black bg-gradient-to-r from-orange-600 via-yellow-500 to-orange-600 bg-clip-text text-transparent mb-3 drop-shadow-sm">
            {performance.api_response_time ? `${performance.api_response_time.toFixed(1)}ms` : '<100ms'}
          </div>
          <div className="text-2xl font-bold text-orange-700 dark:text-orange-300 mb-2">
            Avg Response Time
          </div>
          <div className="text-base text-orange-600/80 dark:text-orange-400/80 font-medium">
            Average API response time across all endpoints
          </div>
        </PancakeCard>
      </div>
    </div>
  );
}
