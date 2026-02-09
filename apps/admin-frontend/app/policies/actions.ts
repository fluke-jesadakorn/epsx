'use server';

import { logout } from '@/lib/auth/auth';
import { createAdminApiClient } from '@/shared/api';
import { redirect } from 'next/navigation';

interface PolicyStats {
    total_policies: number;
    active_policies: number;
    policies_by_type: Record<string, number>;
    evaluations_24h: number;
    avg_evaluation_time_ms: number;
    decision_breakdown: Record<string, number>;
}

export async function getPolicyStatsAction(): Promise<PolicyStats | null> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<{ stats: PolicyStats }>('/api/admin/policies/stats');
    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        return null;
    }
    if (res.data) {
        return res.data.stats;
    }
    return null;
}

export async function getPolicyTemplatesAction(): Promise<unknown[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<{ templates: unknown[] }>('/api/admin/policies/templates');
    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        return [];
    }
    if (res.data) {
        return res.data.templates ?? [];
    }
    return [];
}

export async function evaluatePolicyAction(context: Record<string, unknown>): Promise<unknown> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post<{ evaluation: unknown }>('/api/admin/policies/evaluate', context);
    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        throw new Error(res.error?.message ?? 'Failed to evaluate policy');
    }
    if (res.data) {
        return res.data.evaluation;
    }
    throw new Error('Failed to evaluate policy: No data');
}

export async function createPolicyAction(formData: Record<string, unknown>): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post('/api/admin/policies', formData);
    if (!res.success) {
        if (res.error?.code === '401' || res.error?.code === 'UNAUTHORIZED') {
            await logout();
            redirect('/auth');
        }
        throw new Error(res.error?.message ?? 'Failed to save policy');
    }
}
