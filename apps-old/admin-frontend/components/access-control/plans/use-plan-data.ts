import { useCallback, useState } from 'react';
import { toast } from 'sonner';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import { getPlansAction } from '@/app/wallet-management/plan-actions';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { logger } from '@/lib/logger';

export function useLoadPlansAndPermissions() {
    const [permissions, setPermissions] = useState<PermissionDefinition[]>([]);
    const [plans, setPlans] = useState<PermissionPlan[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    const load = useCallback(async () => {
        setIsLoading(true);
        try {
            const [permRes, planRes] = await Promise.all([
                getPermissionsAction(),
                getPlansAction(),
            ]);
            if (permRes.success && permRes.data !== undefined) {
                setPermissions(permRes.data);
            }
            if (Array.isArray(planRes)) {
                setPlans(planRes);
            }
        } catch (error: unknown) {
            logger.error(
                'Failed to load data:',
                error instanceof Error ? error.message : String(error)
            );
            toast.error('Failed to load access data');
        } finally {
            setIsLoading(false);
        }
    }, []);

    return { permissions, plans, isLoading, setPlans, load };
}
