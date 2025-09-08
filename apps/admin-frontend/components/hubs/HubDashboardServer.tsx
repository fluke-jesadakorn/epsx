/**
 * Server-rendered Hub Dashboard - Analytics Style
 * Uses server component data fetching with analytics theming
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
    <div className="min-h-screen bg-[hsl(var(--background))] p-6">
      {/* Welcome Message */}
      <div className="mb-8">
        <h1 className="text-4xl font-bold bg-gradient-to-r from-blue-500 via-purple-400 to-blue-600 bg-clip-text text-transparent mb-2">
          Welcome back, {session.user?.name || session.user?.email}
        </h1>
        <p className="text-blue-700 dark:text-blue-300">
          Admin Dashboard • {new Date().toLocaleDateString()}
        </p>
      </div>

      {/* Analytics Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 lg:gap-6 mb-8">
        
        <Link href="/users" className="block">
          <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden touch-manipulation transition-all duration-300 hover:shadow-3xl hover:scale-[1.02] p-6">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-full flex items-center justify-center bg-green-400 text-purple-800">
                  <span className="font-bold text-lg">1</span>
                </div>
                <h3 className="font-bold text-3xl text-white">USERS</h3>
              </div>
            </div>
            <div className="mb-6 flex justify-center">
              <button className="px-8 py-3 rounded-full font-bold text-lg transition-colors bg-green-400 text-purple-800 hover:bg-green-300">
                ● ACTIVE
              </button>
            </div>
            <div className="mb-6">
              <div className="flex items-center justify-between mb-3">
                <span className="text-white font-medium">Status</span>
                <span className="text-white font-medium">30d left</span>
              </div>
              <div className="w-full bg-purple-500/50 rounded-full h-3">
                <div className="bg-green-400 h-3 rounded-full transition-all duration-1000" style={{width: '70%'}}></div>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="rounded-2xl p-4 text-center bg-green-400 text-purple-800">
                <div className="font-bold text-sm mb-1">Value</div>
                <div className="font-bold text-xl">{(stats.total_users || 5).toLocaleString()}</div>
              </div>
              <div className="text-center flex flex-col justify-center">
                <div className="text-white/80 font-medium text-sm mb-1">Info</div>
                <div className="text-white font-bold text-lg">+{stats.recent_users_30_days || 0} this month</div>
              </div>
            </div>
          </div>
        </Link>

        <Link href="/permissions" className="block">
          <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden touch-manipulation transition-all duration-300 hover:shadow-3xl hover:scale-[1.02] p-6">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-full flex items-center justify-center bg-green-400 text-purple-800">
                  <span className="font-bold text-lg">2</span>
                </div>
                <h3 className="font-bold text-3xl text-white">PERMISSIONS</h3>
              </div>
            </div>
            <div className="mb-6 flex justify-center">
              <button className="px-8 py-3 rounded-full font-bold text-lg transition-colors bg-green-400 text-purple-800 hover:bg-green-300">
                ● ACTIVE
              </button>
            </div>
            <div className="mb-6">
              <div className="flex items-center justify-between mb-3">
                <span className="text-white font-medium">Status</span>
                <span className="text-white font-medium">45d left</span>
              </div>
              <div className="w-full bg-purple-500/50 rounded-full h-3">
                <div className="bg-green-400 h-3 rounded-full transition-all duration-1000" style={{width: '85%'}}></div>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="rounded-2xl p-4 text-center bg-green-400 text-purple-800">
                <div className="font-bold text-sm mb-1">Value</div>
                <div className="font-bold text-xl">{(permissionAnalytics.total_permissions || 3842).toLocaleString()}</div>
              </div>
              <div className="text-center flex flex-col justify-center">
                <div className="text-white/80 font-medium text-sm mb-1">Info</div>
                <div className="text-white font-bold text-lg">{permissionAnalytics.expiring_soon || 0} expiring</div>
              </div>
            </div>
          </div>
        </Link>

        <Link href="/analytics" className="block">
          <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden touch-manipulation transition-all duration-300 hover:shadow-3xl hover:scale-[1.02] p-6">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-full flex items-center justify-center bg-green-400 text-purple-800">
                  <span className="font-bold text-lg">3</span>
                </div>
                <h3 className="font-bold text-3xl text-white">ANALYTICS</h3>
              </div>
            </div>
            <div className="mb-6 flex justify-center">
              <button className="px-8 py-3 rounded-full font-bold text-lg transition-colors bg-green-400 text-purple-800 hover:bg-green-300">
                ● ACTIVE
              </button>
            </div>
            <div className="mb-6">
              <div className="flex items-center justify-between mb-3">
                <span className="text-white font-medium">Status</span>
                <span className="text-white font-medium">60d left</span>
              </div>
              <div className="w-full bg-purple-500/50 rounded-full h-3">
                <div className="bg-green-400 h-3 rounded-full transition-all duration-1000" style={{width: '95%'}}></div>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="rounded-2xl p-4 text-center bg-green-400 text-purple-800">
                <div className="font-bold text-sm mb-1">Value</div>
                <div className="font-bold text-xl">{Math.round(permissionAnalytics.health_score || 95)}%</div>
              </div>
              <div className="text-center flex flex-col justify-center">
                <div className="text-white/80 font-medium text-sm mb-1">Info</div>
                <div className="text-white font-bold text-lg">System Health</div>
              </div>
            </div>
          </div>
        </Link>

        <Link href="/system" className="block">
          <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden touch-manipulation transition-all duration-300 hover:shadow-3xl hover:scale-[1.02] p-6">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-full flex items-center justify-center bg-green-400 text-purple-800">
                  <span className="font-bold text-lg">4</span>
                </div>
                <h3 className="font-bold text-3xl text-white">SYSTEM</h3>
              </div>
            </div>
            <div className="mb-6 flex justify-center">
              <button className="px-8 py-3 rounded-full font-bold text-lg transition-colors bg-green-400 text-purple-800 hover:bg-green-300">
                ● ACTIVE
              </button>
            </div>
            <div className="mb-6">
              <div className="flex items-center justify-between mb-3">
                <span className="text-white font-medium">Status</span>
                <span className="text-white font-medium">15d left</span>
              </div>
              <div className="w-full bg-purple-500/50 rounded-full h-3">
                <div className="bg-green-400 h-3 rounded-full transition-all duration-1000" style={{width: '65%'}}></div>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="rounded-2xl p-4 text-center bg-green-400 text-purple-800">
                <div className="font-bold text-sm mb-1">Value</div>
                <div className="font-bold text-xl">{systemMetrics.memory_usage || 65}%</div>
              </div>
              <div className="text-center flex flex-col justify-center">
                <div className="text-white/80 font-medium text-sm mb-1">Info</div>
                <div className="text-white font-bold text-lg">Memory Usage</div>
              </div>
            </div>
          </div>
        </Link>
      </div>

      {/* Analytics Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 lg:gap-6">
        <div className="bg-gradient-to-br from-white/80 via-blue-50/20 to-purple-50/30 dark:from-gray-900 dark:via-gray-800 dark:to-blue-900/20 border-2 border-blue-200/50 dark:border-blue-700/50 shadow-xl shadow-blue-200/30 dark:shadow-blue-900/50 hover:shadow-2xl hover:shadow-blue-300/40 dark:hover:shadow-blue-700/60 hover:scale-105 transition-all duration-300 rounded-2xl p-6 text-center backdrop-blur-md">
          <div className="text-7xl font-black text-gray-900 dark:text-white mb-3 drop-shadow-sm">{(stats.active_users || 5).toLocaleString()}</div>
          <div className="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-2">Active Users</div>
          <div className="text-base text-gray-600 dark:text-gray-400 font-medium">Total registered users in the system</div>
        </div>
        
        <div className="bg-gradient-to-br from-white/80 via-blue-50/20 to-purple-50/30 dark:from-gray-900 dark:via-gray-800 dark:to-blue-900/20 border-2 border-blue-200/50 dark:border-blue-700/50 shadow-xl shadow-blue-200/30 dark:shadow-blue-900/50 hover:shadow-2xl hover:shadow-blue-300/40 dark:hover:shadow-blue-700/60 hover:scale-105 transition-all duration-300 rounded-2xl p-6 text-center backdrop-blur-md">
          <div className="text-7xl font-black text-gray-900 dark:text-white mb-3 drop-shadow-sm">{Math.round(permissionAnalytics.health_score || 95)}%</div>
          <div className="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-2">System Health</div>
          <div className="text-base text-gray-600 dark:text-gray-400 font-medium">Overall system performance and reliability</div>
        </div>
        
        <div className="bg-gradient-to-br from-white/80 via-blue-50/20 to-purple-50/30 dark:from-gray-900 dark:via-gray-800 dark:to-blue-900/20 border-2 border-blue-200/50 dark:border-blue-700/50 shadow-xl shadow-blue-200/30 dark:shadow-blue-900/50 hover:shadow-2xl hover:shadow-blue-300/40 dark:hover:shadow-blue-700/60 hover:scale-105 transition-all duration-300 rounded-2xl p-6 text-center backdrop-blur-md">
          <div className="text-6xl font-black text-gray-900 dark:text-white mb-3 drop-shadow-sm">{systemMetrics.api_response_time ? systemMetrics.api_response_time.toFixed(1) : '<100'}ms</div>
          <div className="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-2">Avg Response Time</div>
          <div className="text-base text-gray-600 dark:text-gray-400 font-medium">Average API response time across all endpoints</div>
        </div>
      </div>
    </div>
  );
}