'use client';

import { Activity, AlertTriangle, BarChart3, RefreshCw, Shield, Users } from 'lucide-react';
import React, { Suspense } from 'react';
import { SWRConfig } from 'swr';

import UsageAnalyticsTab from '@/components/admin/UsageAnalyticsTab';
import { PlanAnalyticsDashboard } from '@/components/plans/PlanAnalyticsDashboard';
import { PageHeader, PageLayout, PageSkeleton } from '@/components/shared';
import { AnalyticsStatsCard, AnalyticsSummaryCard } from '@/components/ui/AnalyticsCard';
import { useAnalyticsOverview, useApiKeys, useRealTimeMetrics } from '@/hooks/useAnalyticsData';
import { Button } from '@/shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/components/ui/card';

function LoadingCard(): React.JSX.Element {
    return (
        <Card>
            <CardContent className="p-6">
                <div className="animate-pulse space-y-3">
                    <div className="w-10 h-10 bg-muted rounded-lg"></div>
                    <div className="w-20 h-4 bg-muted rounded"></div>
                    <div className="w-24 h-8 bg-muted rounded"></div>
                    <div className="w-32 h-3 bg-muted rounded"></div>
                </div>
            </CardContent>
        </Card>
    );
}

function ErrorCard({ error, onRetry }: { error: any; onRetry?: () => void }): React.JSX.Element {
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
                            {error?.message || 'Unknown error occurred'}
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
        'user-stats': any;
        'permission-analytics': any;
        'system-metrics': any;
        'developer-portal-stats': any;
        'api-keys': any;
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
        hasError,
        errors,
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
                    <Button onClick={refreshAll} variant="outline" disabled={isLoading}>
                        <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
                        Refresh Data
                    </Button>
                }
            />

            {/* Summary Cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
                {isLoading ? (
                    Array.from({ length: 4 }).map((_, i) => <LoadingCard key={i} />)
                ) : hasError ? (
                    <div className="col-span-full">
                        <ErrorCard error={errors.userStats || errors.dashboard} onRetry={refreshAll} />
                    </div>
                ) : (
                    <>
                        <AnalyticsSummaryCard
                            title="Total Users"
                            value={userStats?.total_users?.toLocaleString() || '0'}
                            subtitle="Active users in system"
                        />
                        <AnalyticsSummaryCard
                            title="API Requests"
                            value={dashboardData?.metrics?.totalRequests?.toLocaleString() || '0'}
                            subtitle="Total requests processed"
                        />
                        <AnalyticsSummaryCard
                            title="Active Permissions"
                            value={permissionAnalytics?.total_permissions?.toLocaleString() || '0'}
                            subtitle="Current permission sets"
                        />
                        <AnalyticsSummaryCard
                            title="System Health"
                            value={`${(permissionAnalytics?.health_score || 0)}%`}
                            subtitle="Overall health score"
                        />
                    </>
                )}
            </div>

            {/* Real-time Stats Cards Row */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
                {isLoading ? (
                    Array.from({ length: 3 }).map((_, i) => <LoadingCard key={i} />)
                ) : (
                    <>
                        <AnalyticsStatsCard
                            title="Active Users"
                            value={activeUsers || systemMetrics?.active_users || 0}
                            subtitle="Currently online"
                            iconName="users"
                            trend="up"
                            trendValue={userStats?.new_users_30_days && userStats.total_users ? `+${((userStats.new_users_30_days / userStats.total_users) * 100).toFixed(1)}%` : 'N/A'}
                            statusColor="green"
                            rank={1}
                        />
                        <AnalyticsStatsCard
                            title="Expiring Permissions"
                            value={permissionAnalytics?.expiring_soon || 0}
                            subtitle="Need attention"
                            iconName="permissions"
                            trend={permissionAnalytics?.expired && permissionAnalytics.expired > 0 ? "down" : "neutral"}
                            trendValue={permissionAnalytics?.expired ? `${permissionAnalytics.expired} expired` : "stable"}
                            statusColor={permissionAnalytics?.expiring_soon && permissionAnalytics.expiring_soon > 10 ? "yellow" : "green"}
                            rank={2}
                        />
                        <AnalyticsStatsCard
                            title="Response Time"
                            value={`${responseTime || systemMetrics?.api_response_time || 0}ms`}
                            subtitle="API latency"
                            iconName="analytics"
                            trend="neutral"
                            trendValue="real-time"
                            statusColor={responseTime && responseTime > 500 ? "red" : responseTime && responseTime > 200 ? "yellow" : "green"}
                            rank={3}
                        />
                    </>
                )}
            </div>

            {/* Key Metrics Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
                <Card>
                    <CardContent className="p-6">
                        <div className="flex items-center">
                            <div className="p-2 bg-blue-100 dark:bg-blue-900/50 rounded-lg">
                                <Users className="w-6 h-6 text-blue-600" />
                            </div>
                            <div className="ml-4">
                                <p className="text-sm font-medium text-muted-foreground">
                                    Active Users
                                </p>
                                <p className="text-2xl font-bold text-foreground">
                                    {userStats?.active_users?.toLocaleString() || '0'}
                                </p>
                                <p className="text-xs text-muted-foreground">
                                    {userStats?.new_users_30_days ? `+${userStats.new_users_30_days} this month` : 'No recent data'}
                                </p>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                <Card>
                    <CardContent className="p-6">
                        <div className="flex items-center">
                            <div className="p-2 bg-green-100 dark:bg-green-900/50 rounded-lg">
                                <Shield className="w-6 h-6 text-green-600" />
                            </div>
                            <div className="ml-4">
                                <p className="text-sm font-medium text-muted-foreground">
                                    Security Score
                                </p>
                                <p className="text-2xl font-bold text-foreground">
                                    {permissionAnalytics?.health_score ? `${permissionAnalytics.health_score}%` : 'N/A'}
                                </p>
                                <p className="text-xs text-muted-foreground">
                                    Permission health
                                </p>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                <Card>
                    <CardContent className="p-6">
                        <div className="flex items-center">
                            <div className="p-2 bg-purple-100 dark:bg-purple-900/50 rounded-lg">
                                <Activity className="w-6 h-6 text-purple-600" />
                            </div>
                            <div className="ml-4">
                                <p className="text-sm font-medium text-muted-foreground">
                                    DB Query Time
                                </p>
                                <p className="text-2xl font-bold text-foreground">
                                    {systemMetrics?.database_query_time || 0}ms
                                </p>
                                <p className="text-xs text-muted-foreground">
                                    Database latency
                                </p>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                <Card>
                    <CardContent className="p-6">
                        <div className="flex items-center">
                            <div className="p-2 bg-orange-100 dark:bg-orange-900/50 rounded-lg">
                                <BarChart3 className="w-6 h-6 text-orange-600" />
                            </div>
                            <div className="ml-4">
                                <p className="text-sm font-medium text-muted-foreground">
                                    Memory Usage
                                </p>
                                <p className="text-2xl font-bold text-foreground">
                                    {memoryUsage || 0}%
                                </p>
                                <p className="text-xs text-muted-foreground">
                                    System memory
                                </p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* API Usage Analytics */}
            <Card>
                <CardHeader>
                    <CardTitle>API Usage Analytics</CardTitle>
                </CardHeader>
                <CardContent>
                    {apiKeysLoading ? (
                        <div className="flex items-center justify-center p-8">
                            <RefreshCw className="w-6 h-6 animate-spin mr-2" />
                            <span>Loading API usage data...</span>
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
