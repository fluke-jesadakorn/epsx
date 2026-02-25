/**
 * Plan Analytics Dashboard Component
 * Provides basic analytics for permission plans
 */

'use client'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { usePlanAnalytics } from '@/hooks/use-plan-permissions';
import { useEffect } from 'react';

interface PlanAnalyticsDashboardProps {
    className?: string
}

/**
 *
 * @param root0
 * @param root0.className
 */
export function PlanAnalyticsDashboard({ className }: PlanAnalyticsDashboardProps) {
    const { stats, loading, refreshStats } = usePlanAnalytics();

    useEffect(() => {
        refreshStats();
    }, [refreshStats]);

    if (loading) {
        return (
            <div className={`space-y-6 ${className ?? ''}`}>
                <div>
                    <Skeleton className="h-8 w-48 mb-2" />
                    <Skeleton className="h-4 w-64" />
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <Card key={i}>
                            <CardHeader><Skeleton className="h-6 w-32" /></CardHeader>
                            <CardContent><Skeleton className="h-8 w-16" /></CardContent>
                        </Card>
                    ))}
                </div>
            </div>
        );
    }

    return (
        <div className={`space-y-6 ${className ?? ''}`}>
            <div>
                <h2 className="text-2xl font-bold">Plan Analytics</h2>
                <p className="text-muted-foreground mt-1">
                    Analytics dashboard for permission plans
                </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <Card>
                    <CardHeader>
                        <CardTitle>Total Plans</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-2xl font-bold">{stats.totalPlans}</p>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>Active Plans</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-2xl font-bold">{stats.activePlans}</p>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>System Plans</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-2xl font-bold">{stats.systemPlans}</p>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>Avg Permissions</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p className="text-2xl font-bold">{stats.avgPermissionsPerPlan}</p>
                    </CardContent>
                </Card>
            </div>

            {stats.totalPlans === 0 && (
                <Card>
                    <CardContent className="p-6 text-center text-muted-foreground">
                        No plans found. Create a plan to see analytics.
                    </CardContent>
                </Card>
            )}
        </div>
    )
}

export default PlanAnalyticsDashboard
