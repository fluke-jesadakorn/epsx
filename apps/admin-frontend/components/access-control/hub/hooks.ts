import { useSearchParams } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import {
    type AccessPolicy,
    DEFAULT_POLICY_FILTERS,
    DEFAULT_POLICY_STATS,
    POLICY_TYPE_CONFIG,
    type PolicyFilters as PolicyFiltersType,
    type PolicyStats,
    type PolicyType,
} from '@/components/access-control/types';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import { logger } from '@/shared/utils/logger';

export function useAccessControlHub() {
    const searchParams = useSearchParams();

    // State
    const [policies, setPolicies] = useState<AccessPolicy[]>([]);
    const [stats, setStats] = useState<PolicyStats>(DEFAULT_POLICY_STATS);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Filters - initialized from URL params
    const [filters, setFilters] = useState<PolicyFiltersType>(() => {
        const typeParam = searchParams.get('type');
        const types: PolicyType[] | 'all' = typeParam
            ? (typeParam
                .split(',')
                .filter((t) => t in POLICY_TYPE_CONFIG) as PolicyType[])
            : 'all';

        return {
            ...DEFAULT_POLICY_FILTERS,
            types: types.length === 0 ? 'all' : types,
        };
    });

    // Delete confirmation
    const [deleteConfirm, setDeleteConfirm] = useState<{
        policy: AccessPolicy;
    } | null>(null);
    const [isDeleting, setIsDeleting] = useState(false);

    // Selection for bulk actions
    const [selectedPolicies, setSelectedPolicies] = useState<Set<string>>(
        new Set()
    );

    // Load data
    const loadData = useCallback(async () => {
        setIsLoading(true);
        setError(null);

        try {
            const [policiesData, statsData] = await Promise.all([
                accessPolicyClient.getPolicies() as Promise<AccessPolicy[]>,
                accessPolicyClient.getStats() as Promise<PolicyStats>,
            ]);

            setPolicies(policiesData);
            setStats(statsData);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            logger.error('Failed to load access control data:', message);
            setError(message);
            toast.error('Failed to load access control data');
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        void loadData();
    }, [loadData]);

    // Filter and sort policies
    const filteredPolicies = useMemo(() => {
        return accessPolicyClient.filterPolicies(policies, filters);
    }, [policies, filters]);

    // Selection handlers
    const handleSelectPolicy = (policyId: string, selected: boolean) => {
        setSelectedPolicies((prev) => {
            const next = new Set(prev);
            if (selected) {
                next.add(policyId);
            } else {
                next.delete(policyId);
            }
            return next;
        });
    };

    // Delete handler
    const handleDeletePolicy = async () => {
        if (!deleteConfirm) {
            return;
        }

        setIsDeleting(true);
        try {
            await accessPolicyClient.deletePolicy(deleteConfirm.policy.id);
            toast.success(`"${deleteConfirm.policy.name}" deleted successfully`);
            setDeleteConfirm(null);
            await loadData();
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            logger.error('Failed to delete policy:', message);
            toast.error('Failed to delete policy');
        } finally {
            setIsDeleting(false);
        }
    };

    return {
        policies,
        stats,
        isLoading,
        error,
        filters,
        setFilters,
        deleteConfirm,
        setDeleteConfirm,
        isDeleting,
        selectedPolicies,
        handleSelectPolicy,
        handleDeletePolicy,
        loadData,
        filteredPolicies,
    };
}
