'use client';

import {
    assignUserToPlanAction,
    grantPermissionAction,
    removeUserFromPlanAction,
    revokePermissionAction,
} from '@/app/wallet-management/plan-actions';
import { useCallback } from 'react';

export function useWalletAccessActions(walletAddress: string | null, onSuccess: () => Promise<void>) {
    // Assign permission
    const assignPermission = useCallback(async (permissionId: string, expiresAt?: string) => {
        if (walletAddress === null) { return; }
        try {
            await grantPermissionAction(walletAddress, permissionId, expiresAt);
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to assign permission', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Revoke permission
    const revokePermission = useCallback(async (permissionId: string) => {
        if (walletAddress === null) { return; }
        try {
            await revokePermissionAction(walletAddress, permissionId);
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to revoke permission', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Assign plan
    const assignPlan = useCallback(async (planId: string, expiresAt?: string) => {
        if (walletAddress === null) { return; }
        try {
            await assignUserToPlanAction(walletAddress, planId, expiresAt ?? null);
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to assign plan', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Remove plan
    const removePlan = useCallback(async (planId: string) => {
        if (walletAddress === null) { return; }
        try {
            await removeUserFromPlanAction(walletAddress, planId);
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to remove plan', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Batch assign permissions
    const batchAssignPermissions = useCallback(async (permissionIds: string[], expiresAt?: string) => {
        if (walletAddress === null || permissionIds.length === 0) { return; }
        try {
            await Promise.all(
                permissionIds.map(permissionId =>
                    grantPermissionAction(walletAddress, permissionId, expiresAt)
                )
            );
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch assign permissions', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Batch revoke permissions
    const batchRevokePermissions = useCallback(async (permissionIds: string[]) => {
        if (walletAddress === null || permissionIds.length === 0) { return; }
        try {
            await Promise.all(
                permissionIds.map(permissionId =>
                    revokePermissionAction(walletAddress, permissionId)
                )
            );
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch revoke permissions', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Batch assign plans
    const batchAssignPlans = useCallback(async (planIds: string[], expiresAt?: string) => {
        if (walletAddress === null || planIds.length === 0) { return; }
        try {
            await Promise.all(
                planIds.map(planId =>
                    assignUserToPlanAction(walletAddress, planId, expiresAt ?? null)
                )
            );
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch assign plans', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    // Batch remove plans
    const batchRemovePlans = useCallback(async (planIds: string[]) => {
        if (walletAddress === null || planIds.length === 0) { return; }
        try {
            await Promise.all(
                planIds.map(planId => removeUserFromPlanAction(walletAddress, planId))
            );
            await onSuccess();
        } catch (err) {
            throw new Error(err instanceof Error ? err.message : 'Failed to batch remove plans', { cause: err });
        }
    }, [walletAddress, onSuccess]);

    return {
        assignPermission,
        revokePermission,
        assignPlan,
        removePlan,
        batchAssignPermissions,
        batchRevokePermissions,
        batchAssignPlans,
        batchRemovePlans,
    };
}
