'use client';

import { Activity, AlertTriangle, BarChart3, RefreshCw, Shield, Users } from 'lucide-react';
import React, { Suspense } from 'react';
import { SWRConfig } from 'swr';

import UsageAnalyticsTab from '@/components/admin/usage-analytics-tab';
import { PlanAnalyticsDashboard } from '@/components/plans/Plananalytics-dashboard';
import { PageHeader, PageLayout, PageSkeleton } from '@/components/shared';
import { AnalyticsStatsCard, AnalyticsSummaryCard } from '@/components/ui/analytics-card';
import { useAnalyticsOverview, useApiKeys, useRealTimeMetrics, type ApiKey, type DeveloperPortalStats, type PermissionAnalytics, type SystemMetrics, type UserStats } from '@/hooks/use-analytics-data';
import { Button } from '@/shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/components/ui/card';

function ErrorCard({ error, onRetry }: { error: Error | { message?: string } | null | unknown; onRetry?: () => void }): React.JSX.Element {
    return (
        <Card className="border-red-200 dark:border-red-800">
            <CardContent className="p-6">
                <div className="flex items-center space-x-3">
                    <AlertTriangle className="w-6 h-6 text-red-500" />
                    <div className="flex-1">
                        <p className="text-sm font-medium text-red-900 dark:text-red-100">
                            Failed to load data
                        </p>
                        <p className="text-xs text-red-600 dark:text-red-400">
                            {error instanceof Error ? error.message : typeof error === 'object' && error !== null && 'message' in error ? String(error.message) : 'Unknown error occurred'}
                        </p>
                    </div>
                    {onRetry && (
                        <Button variant="outline" size="sm" onClick={onRetry}>
                            <RefreshCw className="w-4 h-4" />
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}

/**
 * Analytics Page
 * Uses unified page components for consistent design
 */

interface AnalyticsDashboardProps {
    initialData: {
        'user-stats'?: UserStats;
        'permission-analytics'?: PermissionAnalytics;
        'system-metrics'?: SystemMetrics;
        'developer-portal-stats'?: DeveloperPortalStats;
        'api-keys'?: ApiKey[];
    }
}

/**
 * Analytics Dashboard (Client Component)
 * Hydrated with server-side data via SWRConfig
 */
export default function AnalyticsDashboard({ initialData }: AnalyticsDashboardProps): React.JSX.Element {
    return (
        <SWRConfig value={{ fallback: initialData }}>
            <AnalyticsContent />
        </SWRConfig>
    );
}

function AnalyticsContent(): React.JSX.Element {
    const {
        userStats,
        permissionAnalytics,
        systemMetrics,
        dashboardData,
        isLoading,
        refreshAll
    } = useAnalyticsOverview();

    const { apiKeys, isLoading: apiKeysLoading, error: apiKeysError } = useApiKeys();
    const { responseTime, memoryUsage, activeUsers } = useRealTimeMetrics();

    if (isLoading) {
        return <PageSkeleton showHeader stats={4} rows={6} />;
    }

    return (
        <PageLayout>
            {/* Header */}
            <PageHeader
                title="Analytics Dashboard"
                subtitle="Real-time system performance and user activity"
                icon="BarChart3"
                gradient="info"
                actions={
                    <Button onClick={refreshAll} variant="outline">
                        <RefreshCw className="w-4 h-4 mr-2" />
                        Refresh Data
                    </Button>
                }
            />

            <SummarySection
                userStats={userStats}
                dashboardData={dashboardData}
                permissionAnalytics={permissionAnalytics}
            />

            <StatsSection
                activeUsers={activeUsers}
                userStats={userStats}
                permissionAnalytics={permissionAnalytics}
                responseTime={responseTime}
            />

            <MetricsGrid
                userStats={userStats}
                permissionAnalytics={permissionAnalytics}
                systemMetrics={systemMetrics}
                memoryUsage={memoryUsage}
            />

            {/* API Usage Analytics */}
            <Card className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden">
                <CardHeader className="p-8 border-b border-white/5 bg-white/5">
                    <CardTitle className="text-lg font-black uppercase tracking-widest text-foreground">API Usage Analytics</CardTitle>
                </CardHeader>
                <CardContent className="p-8">
                    {apiKeysLoading ? (
                        <div className="flex items-center justify-center p-8">
                            <RefreshCw className="w-6 h-6 animate-spin mr-2 text-[#1fc7d4]" />
                            <span className="font-bold text-muted-foreground">Loading API usage data...</span>
                        </div>
                    ) : apiKeysError ? (
                        <ErrorCard error={apiKeysError} />
                    ) : (
                        <Suspense fallback={<div>Loading usage analytics...</div>}>
                            <UsageAnalyticsTab apiKeys={apiKeys} />
                        </Suspense>
                    )}
                </CardContent>
            </Card>

            {/* Plan Analytics */}
            <Suspense fallback={<div>Loading plan analytics...</div>}>
                <PlanAnalyticsDashboard />
            </Suspense>
        </PageLayout>
    );
}

function SummarySection({ userStats, dashboardData, permissionAnalytics }: { userStats?: UserStats; dashboardData?: { metrics?: { totalRequests?: number } }; permissionAnalytics?: PermissionAnalytics }) {
    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
            <AnalyticsSummaryCard
                title="Total Users"
                value={userStats?.total_users?.toLocaleString() ?? '0'}
                subtitle="Active users in system"
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] p-8"
            />
            <AnalyticsSummaryCard
                title="API Requests"
                value={dashboardData?.metrics?.totalRequests?.toLocaleString() ?? '0'}
                subtitle="Total requests processed"
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] p-8"
            />
            <AnalyticsSummaryCard
                title="Active Permissions"
                value={permissionAnalytics?.total_permissions?.toLocaleString() ?? '0'}
                subtitle="Current permission sets"
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] p-8"
            />
            <AnalyticsSummaryCard
                title="System Health"
                value={`${(permissionAnalytics?.health_score ?? 0)}%`}
                subtitle="Overall health score"
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] p-8"
            />
        </div>
    );
}

const getTrendValue = (userStats?: UserStats) => {
    if (userStats?.new_users_30_days && userStats.total_users) {
        return `+${((userStats.new_users_30_days / userStats.total_users) * 100).toFixed(1)}%`;
    }
    return 'N/A';
};

const getPermissionTrend = (permissionAnalytics?: PermissionAnalytics) => {
    return permissionAnalytics?.expired && permissionAnalytics.expired > 0 ? "down" : "neutral";
};

const getPermissionTrendValue = (permissionAnalytics?: PermissionAnalytics) => {
    return permissionAnalytics?.expired ? `${permissionAnalytics.expired} expired` : "stable";
};

const getResponseTimeStatus = (responseTime: number) => {
    if (responseTime > 500) {
        return "red";
    }
    if (responseTime > 200) {
        return "yellow";
    }
    return "green";
};

function StatsSection({ activeUsers, userStats, permissionAnalytics, responseTime }: { activeUsers: number; userStats?: UserStats; permissionAnalytics?: PermissionAnalytics; responseTime: number }) {
    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
            <AnalyticsStatsCard
                title="Active Users"
                value={activeUsers}
                subtitle="Currently online"
                iconName="users"
                trend="up"
                trendValue={getTrendValue(userStats)}
                statusColor="green"
                rank={1}
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden"
            />
            <AnalyticsStatsCard
                title="Expiring Permissions"
                value={permissionAnalytics?.expiring_soon ?? 0}
                subtitle="Need attention"
                iconName="permissions"
                trend={getPermissionTrend(permissionAnalytics)}
                trendValue={getPermissionTrendValue(permissionAnalytics)}
                statusColor={permissionAnalytics?.expiring_soon && permissionAnalytics.expiring_soon > 10 ? "yellow" : "green"}
                rank={2}
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden"
            />
            <AnalyticsStatsCard
                title="Response Time"
                value={`${responseTime}ms`}
                subtitle="API latency"
                iconName="analytics"
                trend="neutral"
                trendValue="real-time"
                statusColor={getResponseTimeStatus(responseTime)}
                rank={3}
                className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden"
            />
        </div>
    );
}

function MetricsGrid({ userStats, permissionAnalytics, systemMetrics, memoryUsage }: { userStats?: UserStats; permissionAnalytics?: PermissionAnalytics; systemMetrics?: SystemMetrics; memoryUsage: number }) {
    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
            <Card className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden group">
                <CardContent className="p-8">
                    <div className="flex items-center">
                        <div className="p-3 bg-[#1fc7d4]/10 rounded-2xl border border-[#1fc7d4]/10 text-[#1fc7d4]">
                            <Users className="w-6 h-6" />
                        </div>
                        <div className="ml-6">
                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em]">
                                Active Users
                            </p>
                            <p className="text-3xl font-black text-foreground tracking-tighter mt-1">
                                {userStats?.active_users?.toLocaleString() ?? '0'}
                            </p>
                            <p className="text-xs font-bold text-[#31d0aa] mt-0.5">
                                {userStats?.new_users_30_days ? `+${userStats.new_users_30_days} this month` : 'No recent data'}
                            </p>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden group">
                <CardContent className="p-8">
                    <div className="flex items-center">
                        <div className="p-3 bg-[#31d0aa]/10 rounded-2xl border border-[#31d0aa]/10 text-[#31d0aa]">
                            <Shield className="w-6 h-6" />
                        </div>
                        <div className="ml-6">
                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em]">
                                Status Score
                            </p>
                            <p className="text-3xl font-black text-foreground tracking-tighter mt-1">
                                {permissionAnalytics?.health_score ? `${permissionAnalytics.health_score}%` : 'N/A'}
                            </p>
                            <p className="text-xs font-bold text-muted-foreground/60 mt-0.5">
                                Permission health
                            </p>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden group">
                <CardContent className="p-8">
                    <div className="flex items-center">
                        <div className="p-3 bg-[#7645d9]/10 rounded-2xl border border-[#7645d9]/10 text-[#7645d9]">
                            <Activity className="w-6 h-6" />
                        </div>
                        <div className="ml-6">
                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em]">
                                Query Time
                            </p>
                            <p className="text-3xl font-black text-foreground tracking-tighter mt-1">
                                {systemMetrics?.database_query_time ?? 0}ms
                            </p>
                            <p className="text-xs font-bold text-muted-foreground/60 mt-0.5">
                                Database latency
                            </p>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card className="bg-slate-900/40 backdrop-blur-2xl border-white/5 shadow-xl rounded-[32px] overflow-hidden group">
                <CardContent className="p-8">
                    <div className="flex items-center">
                        <div className="p-3 bg-[#ed4b9e]/10 rounded-2xl border border-[#ed4b9e]/10 text-[#ed4b9e]">
                            <BarChart3 className="w-6 h-6" />
                        </div>
                        <div className="ml-6">
                            <p className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em]">
                                Memory
                            </p>
                            <p className="text-3xl font-black text-foreground tracking-tighter mt-1">
                                {memoryUsage}%
                            </p>
                            <p className="text-xs font-bold text-muted-foreground/60 mt-0.5">
                                System usage
                            </p>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
