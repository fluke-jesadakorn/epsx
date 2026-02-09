'use client';

import { useRouter, useSearchParams } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import {
    type AccessPolicy,
    type PolicyFilters as PolicyFiltersType,
    type PolicyType,
    DEFAULT_POLICY_FILTERS,
} from '@/components/access-control/types';
import { accessPolicyClient } from '@/lib/api/access-policy-client';

function filterPolicies(policies: AccessPolicy[], filters: PolicyFiltersType): AccessPolicy[] {
    let result = [...policies];

    if (filters.search) {
        const searchLower = filters.search.toLowerCase();
        result = result.filter(policy =>
            policy.name.toLowerCase().includes(searchLower) ||
            policy.description.toLowerCase().includes(searchLower) ||
            policy.permissions.some(p => p.toLowerCase().includes(searchLower))
        );
    }

    if (filters.types !== 'all') {
        result = result.filter(policy => filters.types.includes(policy.type));
    }

    if (filters.status !== 'all') {
        const isActive = filters.status === 'active';
        result = result.filter(policy => policy.isActive === isActive);
    }

    result.sort((a, b) => {
        let comparison = 0;

        switch (filters.sortBy) {
            case 'name':
                comparison = a.name.localeCompare(b.name);
                break;
            case 'members':
                comparison = a.memberCount - b.memberCount;
                break;
            case 'created_at':
                comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
                break;
            case 'revenue':
                comparison = (a.revenue ?? 0) - (b.revenue ?? 0);
                break;
            case 'type':
                comparison = a.type.localeCompare(b.type);
                break;
        }

        return filters.sortOrder === 'asc' ? comparison : -comparison;
    });

    return result;
}

export function usePolicySection(initialPolicies: AccessPolicy[]) {
    const router = useRouter();
    const searchParams = useSearchParams();

    const getInitialFilters = (): PolicyFiltersType => {
        const typeParam = searchParams.get('type');
        if (typeParam !== null && ['subscription', 'manual', 'web3_asset', 'dao', 'system'].includes(typeParam)) {
            return {
                ...DEFAULT_POLICY_FILTERS,
                types: [typeParam as PolicyType],
            };
        }
        return DEFAULT_POLICY_FILTERS;
    };

    const [policies, setPolicies] = useState<AccessPolicy[]>(initialPolicies);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [filters, setFilters] = useState<PolicyFiltersType>(getInitialFilters());

    useEffect(() => {
        const typeParam = searchParams.get('type');
        if (typeParam !== null && ['subscription', 'manual', 'web3_asset', 'dao', 'system'].includes(typeParam)) {
            setFilters(prev => ({
                ...prev,
                types: [typeParam as PolicyType],
            }));
        }
    }, [searchParams]);

    const [deleteConfirm, setDeleteConfirm] = useState<{ policy: AccessPolicy } | null>(null);
    const [isDeleting, setIsDeleting] = useState(false);
    const [selectedPolicies, setSelectedPolicies] = useState<Set<string>>(new Set());

    const filteredPolicies = useMemo(() => {
        return filterPolicies(policies, filters);
    }, [policies, filters]);

    const handleRefresh = useCallback(async () => {
        setIsRefreshing(true);
        try {
            const freshPolicies = await accessPolicyClient.getPolicies();
            setPolicies(freshPolicies);
            toast.success('Policies refreshed');
        } catch {
            toast.error('Failed to refresh policies');
        } finally {
            setIsRefreshing(false);
        }
    }, []);

    const handleSelectPolicy = (policyId: string, selected: boolean) => {
        setSelectedPolicies(prev => {
            const next = new Set(prev);
            if (selected) {
                next.add(policyId);
            } else {
                next.delete(policyId);
            }
            return next;
        });
    };

    const handleDeletePolicy = async () => {
        if (!deleteConfirm) { return; }

        setIsDeleting(true);
        try {
            await accessPolicyClient.deletePolicy(deleteConfirm.policy.id);
            toast.success(`"${deleteConfirm.policy.name}" deleted successfully`);
            setDeleteConfirm(null);
            setPolicies(prev => prev.filter(p => p.id !== deleteConfirm.policy.id));
        } catch {
            toast.error('Failed to delete policy');
        } finally {
            setIsDeleting(false);
        }
    };

    const handleCreatePlan = () => router.push('/subscriptions/plans/new');
    const handleCreateGroup = () => router.push('/wallet-management/groups/new');

    return {
        policies,
        filteredPolicies,
        isRefreshing,
        filters,
        setFilters,
        videoPolicies: selectedPolicies, // Was unused in component
        deleteConfirm,
        setDeleteConfirm,
        isDeleting,
        handleRefresh,
        handleSelectPolicy,
        handleDeletePolicy,
        handleCreatePlan,
        handleCreateGroup,
        selectedPolicies,
        setSelectedPolicies
    };
}
