'use client';

import { createAdminApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';

import { logger } from '@/lib/logger';

export interface DashboardStats {
    totalWallets: number;
    activeWallets: number;
    totalPermissions: number;
    systemHealth: number;
    todayConnections: number;
    pendingNotifications: number;
    systemUptime: string;
    avgResponseTime: string;
}

interface DashboardSummary {
    wallet_stats: { total: number; active: number; today_connections: number } | null;
    permission_stats: { total: number; pending_notifications: number } | null;
    system_health: { health_percentage: number; uptime: string; avg_response_time: string } | null;
}

/**
 * Hook to manage dashboard data loading
 */
export function useDashboardData(isAuthenticated: boolean) {
    const [dashboardStats, setDashboardStats] = useState<DashboardStats>({
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

    useEffect(() => {
        if (!isAuthenticated) { return; }

        const loadDashboardData = async () => {
            try {
                setAccessError(null);
                const client = createAdminApiClient();

                const result = await client.get<DashboardSummary>('/api/admin/dashboard/summary');

                if (result.success && result.data) {
                    const { wallet_stats, permission_stats, system_health } = result.data;
                    if (wallet_stats) {
                        setDashboardStats(prev => ({
                            ...prev,
                            totalWallets: wallet_stats.total,
                            activeWallets: wallet_stats.active,
                            todayConnections: wallet_stats.today_connections,
                        }));
                    }
                    if (permission_stats) {
                        setDashboardStats(prev => ({
                            ...prev,
                            totalPermissions: permission_stats.total,
                            pendingNotifications: permission_stats.pending_notifications,
                        }));
                    }
                    if (system_health) {
                        setDashboardStats(prev => ({
                            ...prev,
                            systemHealth: system_health.health_percentage,
                            systemUptime: system_health.uptime,
                            avgResponseTime: system_health.avg_response_time,
                        }));
                    }
                }

                if (!result.success) {
                    setAccessError(result.error?.message ?? 'Access denied');
                }
            } catch (err) {
                logger.error('Failed to load dashboard data:', { err });
            }
        };

        void loadDashboardData();
    }, [isAuthenticated]);

    return { dashboardStats, accessError };
}
