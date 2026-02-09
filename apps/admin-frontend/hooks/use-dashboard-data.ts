'use client';

import { APIError, createAdminApiClient, type ApiResponse } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';

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

        const updateWalletStats = (res: { success: boolean, data: { total: number; active: number; today_connections: number } | null }) => {
            if (res.success && res.data) {
                setDashboardStats(prev => ({
                    ...prev,
                    totalWallets: res.data?.total ?? 0,
                    activeWallets: res.data?.active ?? 0,
                    todayConnections: res.data?.today_connections ?? 0
                }));
            }
        };

        const updatePermissionStats = (res: { success: boolean, data: { total: number; pending_notifications: number } | null }) => {
            if (res.success && res.data) {
                setDashboardStats(prev => ({
                    ...prev,
                    totalPermissions: res.data?.total ?? 0,
                    pendingNotifications: res.data?.pending_notifications ?? 0
                }));
            }
        };

        const updateSystemStats = (res: { success: boolean, data: { health_percentage: number; uptime: string; avg_response_time: string } | null }) => {
            if (res.success && res.data) {
                setDashboardStats(prev => ({
                    ...prev,
                    systemHealth: res.data?.health_percentage ?? 100,
                    systemUptime: res.data?.uptime ?? '99.9%',
                    avgResponseTime: res.data?.avg_response_time ?? '120ms'
                }));
            }
        };

        const loadDashboardData = async () => {
            try {
                setAccessError(null);
                const client = createAdminApiClient();

                // Helper to catch errors and format them
                const safeFetch = async <T,>(promise: Promise<ApiResponse<T>>): Promise<{
                    success: boolean;
                    data: T | null;
                    status: number;
                    error?: string;
                }> => {
                    try {
                        const res = await promise;
                        return {
                            success: res.success,
                            data: res.data,
                            status: res.success ? 200 : 400, // Fallback status
                            error: res.error?.message
                        };
                    } catch (err: unknown) {
                        if (err instanceof APIError) {
                            return { success: false, data: null, status: err.status ?? 0, error: err.message };
                        }
                        return { success: false, data: null, status: 0, error: 'Network error or unexpected issue' };
                    }
                };

                // Fetch wallet data, permissions, and system stats in parallel
                const [walletsRes, permissionsRes, systemRes] = await Promise.all([
                    safeFetch(client.get<{ total: number; active: number; today_connections: number }>('/api/admin/wallets/stats')),
                    safeFetch(client.get<{ total: number; pending_notifications: number }>('/api/admin/permissions/system/stats')),
                    safeFetch(client.get<{ health_percentage: number; uptime: string; avg_response_time: string }>('/api/admin/permissions/system/health'))
                ]);

                updateWalletStats(walletsRes);
                updatePermissionStats(permissionsRes);
                updateSystemStats(systemRes);

                // Check for permission errors
                const hasAuthError = [walletsRes, permissionsRes, systemRes].some(r => r.status === 403 ?? r.status === 401);
                if (hasAuthError) {
                    setAccessError(walletsRes.error ?? permissionsRes.error ?? systemRes.error ?? 'Access denied by backend');
                }
            } catch (err) {
                console.error('Failed to load dashboard data:', err);
            }
        };

        void loadDashboardData();
    }, [isAuthenticated]);

    return { dashboardStats, accessError };
}
