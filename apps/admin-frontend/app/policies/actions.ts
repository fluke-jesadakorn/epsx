'use server';

import { createAdminApiClient } from '@/shared/api';

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
    if (res.success && res.data) {
        return res.data.stats;
    }
    return null;
}

export async function getPolicyTemplatesAction(): Promise<any[]> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.get<any>('/api/admin/policies/templates');
    if (res.success && res.data) {
        return res.data.templates || [];
    }
    return [];
}

export async function evaluatePolicyAction(context: any): Promise<any> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post<any>('/api/admin/policies/evaluate', context);
    if (res.success && res.data) {
        return res.data.evaluation;
    }
    throw new Error(res.error?.message || 'Failed to evaluate policy');
}

export async function createPolicyAction(formData: any): Promise<void> {
    const apiClient = createAdminApiClient({ serverSide: true });
    const res = await apiClient.post<any>('/api/admin/policies', formData);
    if (!res.success) {
        throw new Error(res.error?.message || 'Failed to save policy');
    }
}
