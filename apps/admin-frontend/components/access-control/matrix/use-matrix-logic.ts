import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { type AccessPolicy } from '@/components/access-control/types';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import {
    type PermissionDefinitionDto,
    planMgmt,
} from '@/lib/api/plan-management-client';

export function useAccessControlMatrix() {
    const [policies, setPolicies] = useState<AccessPolicy[]>([]);
    const [permissions, setPermissions] = useState<PermissionDefinitionDto[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [search, setSearch] = useState('');
    const [isUpdating, setIsUpdating] = useState<Record<string, boolean>>({});

    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            // Load both datasets in parallel
            const [policiesData, permissionsData] = await Promise.all([
                accessPolicyClient.getPolicies(),
                planMgmt.getPermissionDefinitions() as Promise<
                    PermissionDefinitionDto[]
                >,
            ]);

            setPolicies(policiesData);
            setPermissions(permissionsData);
        } catch {
            toast.error('Failed to load access control data');
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        void loadData();
    }, [loadData]);

    // Group permissions by platform
    const groupedPermissions = useMemo(() => {
        const filtered = permissions.filter(
            (p) =>
                (p.permission ?? '').toLowerCase().includes(search.toLowerCase()) ||
                (p.description ?? '').toLowerCase().includes(search.toLowerCase())
        );

        const groups: Record<string, PermissionDefinitionDto[]> = {};

        filtered.forEach((p) => {
            const platform = p.platform ?? 'Other';
            if (!groups[platform]) {
                groups[platform] = [];
            }
            groups[platform]?.push(p);
        });

        // specific sort order for platforms
        const sortOrder = ['epsx', 'admin', 'pay', 'token', 'other'];
        return Object.entries(groups).sort((a, b) => {
            const indexA = sortOrder.indexOf(a[0].toLowerCase());
            const indexB = sortOrder.indexOf(b[0].toLowerCase());
            if (indexA !== -1 && indexB !== -1) {
                return indexA - indexB;
            }
            if (indexA !== -1) {
                return -1;
            }
            if (indexB !== -1) {
                return 1;
            }
            return a[0].localeCompare(b[0]);
        });
    }, [permissions, search]);

    // Handle Permission Toggle
    const togglePermission = async (
        policy: AccessPolicy,
        permissionKey: string
    ) => {
        // Optimistic update ID
        const updateKey = `${policy.id}-${permissionKey}`;
        if (isUpdating[updateKey] === true) {
            return;
        }

        setIsUpdating((prev) => ({ ...prev, [updateKey]: true }));

        const hasPermission = policy.permissions.includes(permissionKey);
        const newPermissions = hasPermission
            ? policy.permissions.filter((p) => p !== permissionKey)
            : [...policy.permissions, permissionKey];

        // Optimistically update local state
        setPolicies((current) =>
            current.map((p) => {
                if (p.id === policy.id) {
                    return { ...p, permissions: newPermissions };
                }
                return p;
            })
        );

        try {
            await accessPolicyClient.updatePolicy(policy.id, {
                permissions: newPermissions,
            });
        } catch {
            toast.error('Could not update permission assignment');
            // Revert on failure
            void loadData();
        } finally {
            setIsUpdating((prev) => ({ ...prev, [updateKey]: false }));
        }
    };

    return {
        policies,
        permissions,
        isLoading,
        search,
        setSearch,
        isUpdating,
        groupedPermissions,
        togglePermission,
        loadData,
    };
}
