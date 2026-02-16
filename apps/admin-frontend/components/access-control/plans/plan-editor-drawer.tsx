'use client';

import { Loader2, X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import {
    deletePlanAction,
    getPlansAction,
} from '@/app/wallet-management/plan-actions';
import { Button } from '@/components/ui/button';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { logger } from '@/shared/utils/logger';

import { DeletePlanDialog } from './delete-plan-dialog';
import { PlanEditor } from './plan-editor';
import { usePlanEditForm } from './use-plans-logic';

interface Props {
    planId: string | null;
    onClose: () => void;
    onPlanUpdated: () => void;
}

// eslint-disable-next-line max-lines-per-function -- sidebar panel wrapper
export function PlanEditorDrawer({ planId, onClose, onPlanUpdated }: Props) {
    const [permissions, setPermissions] = useState<PermissionDefinition[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [deleteTarget, setDeleteTarget] = useState<
        Parameters<typeof DeletePlanDialog>[0]['planToDelete']
    >(null);

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
        if (!planId) return;
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
                        onClose();
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
    // eslint-disable-next-line react-hooks/exhaustive-deps -- load when planId changes
    }, [planId]);

    const handleDelete = async () => {
        if (!deleteTarget) return;
        try {
            await deletePlanAction(deleteTarget.id);
            toast.success('Plan deleted');
            onPlanUpdated();
            onClose();
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to delete plan');
        }
    };

    const handleSave = () => {
        void savePlan([], () => {}).then(() => {
            onPlanUpdated();
        });
    };

    return (
        <>
            <div className="h-full flex flex-col border-l border-white/5 bg-slate-950/50">
                <div className="flex items-center justify-between px-4 py-2 border-b border-white/5 shrink-0">
                    <span className="text-xs text-muted-foreground font-mono truncate">
                        {planId}
                    </span>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onClose}>
                        <X className="h-4 w-4" />
                    </Button>
                </div>
                {isLoading ? (
                    <div className="flex-1 flex items-center justify-center">
                        <Loader2 className="animate-spin" />
                    </div>
                ) : (
                    <div className="flex-1 overflow-y-auto">
                        <PlanEditor
                            selectedPlan={selectedPlan}
                            form={form}
                            setForm={setForm}
                            hasChanges={hasChanges}
                            setHasChanges={setHasChanges}
                            isSaving={isSaving}
                            onSave={handleSave}
                            onDiscard={discardChanges}
                            onDelete={() => setDeleteTarget(selectedPlan)}
                            permissions={permissions}
                        />
                    </div>
                )}
            </div>
            <DeletePlanDialog
                planToDelete={deleteTarget}
                onClose={() => setDeleteTarget(null)}
                onConfirm={() => {
                    void handleDelete();
                }}
            />
        </>
    );
}
