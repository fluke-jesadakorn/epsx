'use client';

import { ArrowLeft, Loader2 } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import {
    deletePlanAction,
    getPlansAction,
} from '@/app/wallet-management/plan-actions';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { useSharedAuth } from '@/shared/components/auth';
import { logger } from '@/shared/utils/logger';

import { DeletePlanDialog } from './delete-plan-dialog';
import { PlanEditor } from './plan-editor';
import { usePlanEditForm } from './use-plans-logic';

interface Props {
    planId: string;
}

export function PlanEditorPage({ planId }: Props) {
    const router = useRouter();
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
    const [permissions, setPermissions] = useState<PermissionDefinition[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [deleteTarget, setDeleteTarget] = useState<PermissionPlan | null>(null);

    const {
        selectedPlan,
        form,
        setForm,
        hasChanges,
        setHasChanges,
        isSaving,
        selectPlan,
        savePlan,
        discardChanges,
    } = usePlanEditForm();

    useEffect(() => {
        if (!isAuthenticated) return;
        void (async () => {
            setIsLoading(true);
            try {
                const [permRes, planRes] = await Promise.all([
                    getPermissionsAction(),
                    getPlansAction(),
                ]);
                if (permRes.success && permRes.data) {
                    setPermissions(permRes.data);
                }
                if (Array.isArray(planRes)) {
                    const plan = planRes.find((p) => p.id === planId);
                    if (plan) {
                        selectPlan(plan);
                    } else {
                        toast.error('Plan not found');
                        router.push('/wallet-management/access');
                    }
                }
            } catch (error: unknown) {
                logger.error(
                    'Failed to load plan:',
                    error instanceof Error ? error.message : String(error)
                );
                toast.error('Failed to load plan data');
            } finally {
                setIsLoading(false);
            }
        })();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- load once on mount
    }, [isAuthenticated, planId]);

    const reloadPermissions = async () => {
        try {
            const res = await getPermissionsAction();
            if (res.success && res.data) {
                setPermissions(res.data);
            }
        } catch (_) {
            // ignore
        }
    };

    const handleDelete = async () => {
        if (!deleteTarget) return;
        try {
            await deletePlanAction(deleteTarget.id);
            toast.success('Plan deleted');
            router.push('/wallet-management/access');
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to delete plan');
        }
    };

    if (authLoading || isLoading) {
        return (
            <div className="p-8 flex justify-center">
                <Loader2 className="animate-spin" />
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4 h-full">
            <Link
                href="/wallet-management/access"
                className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors w-fit"
            >
                <ArrowLeft className="w-4 h-4" />
                Back to Plans
            </Link>
            <div className="flex-1 min-h-0">
                <PlanEditor
                    selectedPlan={selectedPlan}
                    form={form}
                    setForm={setForm}
                    hasChanges={hasChanges}
                    setHasChanges={setHasChanges}
                    isSaving={isSaving}
                    onSave={() => {
                        void savePlan([], () => {});
                    }}
                    onDiscard={discardChanges}
                    onDelete={() => setDeleteTarget(selectedPlan)}
                    permissions={permissions}
                    onPermissionsChanged={() => void reloadPermissions()}
                />
            </div>
            <DeletePlanDialog
                planToDelete={deleteTarget}
                onClose={() => setDeleteTarget(null)}
                onConfirm={() => {
                    void handleDelete();
                }}
            />
        </div>
    );
}
