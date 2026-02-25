'use client';

import { Copy, Loader2, Package, RotateCcw, Shield, Trash2, X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import {
    deletePlanAction,
    getPlansAction,
} from '@/app/wallet-management/plan-actions';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { logger } from '@/shared/utils/logger';

import { categoryBadgeClass, FREE_PLAN_ID, isSystemPlan } from './types';

import { DeletePlanDialog } from './delete-plan-dialog';
import { PlanEditor } from './plan-editor';
import { usePlanEditForm } from './use-plans-logic';

interface Props {
    planId: string | null;
    onClose: () => void;
    onPlanUpdated: () => void;
    onDuplicate?: (plan: import('@/lib/api/plan-management-client').PermissionPlan) => void;
}

// eslint-disable-next-line max-lines-per-function -- sidebar panel wrapper
export function PlanEditorDrawer({ planId, onClose, onPlanUpdated, onDuplicate }: Props) {
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
        if (!planId) {return;}
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
        if (!deleteTarget) {return;}
        try {
            await deletePlanAction(deleteTarget.id);
            toast.success('Plan deleted');
            onPlanUpdated();
            onClose();
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to delete plan');
        }
    };

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

    const handleSave = () => {
        void savePlan([], () => {}).then(() => {
            onPlanUpdated();
        });
    };

    return (
        <>
            {/* Backdrop */}
            <div
                className="fixed inset-0 z-40 bg-black/50 animate-in fade-in duration-200"
                onClick={onClose}
            />
            {/* Drawer */}
            <div className="fixed inset-y-0 right-0 z-50 w-full max-w-full sm:max-w-xl flex flex-col bg-card border-l border-border/20 shadow-2xl animate-in slide-in-from-right duration-300">
                <div className="flex items-center justify-between px-3 sm:px-5 py-3 border-b border-border/20 shrink-0 gap-2">
                    <div className="flex items-center gap-2 sm:gap-3 min-w-0">
                        {selectedPlan != null ? (
                            <>
                                <div className="h-8 w-8 rounded-lg bg-[#1fc7d4]/20 flex items-center justify-center shrink-0">
                                    <Package className="w-4 h-4 text-[#1fc7d4]" />
                                </div>
                                <div className="min-w-0">
                                    <div className="flex items-center gap-1.5 sm:gap-2 flex-wrap">
                                        <span className="text-sm font-medium truncate max-w-[120px] sm:max-w-none">{selectedPlan.name}</span>
                                        <Badge variant="outline" className={`text-[10px] px-1.5 py-0 shrink-0 ${categoryBadgeClass(form.plan_category)}`}>
                                            {form.plan_category}
                                        </Badge>
                                        {isSystemPlan(selectedPlan) && (
                                            <Badge variant="outline" className="text-[10px] px-1.5 py-0 shrink-0 bg-purple-500/15 text-purple-400 border-purple-500/30">
                                                <Shield className="w-2.5 h-2.5 mr-1" />
                                                System
                                            </Badge>
                                        )}
                                        {!form.is_active && (
                                            <Badge variant="outline" className="text-[10px] px-1.5 py-0 shrink-0 bg-red-500/15 text-red-400 border-red-500/30">
                                                inactive
                                            </Badge>
                                        )}
                                    </div>
                                    <p className="text-[11px] text-muted-foreground font-mono truncate">
                                        {selectedPlan.slug ?? selectedPlan.id}
                                    </p>
                                </div>
                            </>
                        ) : (
                            <span className="text-xs text-muted-foreground font-mono truncate">
                                {planId}
                            </span>
                        )}
                    </div>
                    <div className="flex items-center gap-1 sm:gap-2 shrink-0">
                        {selectedPlan != null && (
                            <>
                                {isSystemPlan(selectedPlan) ? (
                                    <Button variant="ghost" className="text-purple-400 hover:text-purple-300 hover:bg-purple-500/10" size="sm" onClick={() => onDuplicate?.(selectedPlan)}>
                                        <Copy className="w-3.5 h-3.5 sm:mr-1.5" />
                                        <span className="hidden sm:inline">Template</span>
                                    </Button>
                                ) : selectedPlan.id !== FREE_PLAN_ID && (
                                    <Button variant="ghost" className="text-red-400 hover:text-red-300 hover:bg-red-500/10" size="sm" onClick={() => setDeleteTarget(selectedPlan)}>
                                        <Trash2 className="w-3.5 h-3.5 sm:mr-1.5" />
                                        <span className="hidden sm:inline">Delete</span>
                                    </Button>
                                )}
                                <Button variant="ghost" size="sm" onClick={discardChanges} disabled={!hasChanges || isSaving} className="text-muted-foreground hover:text-foreground">
                                    <RotateCcw className="w-3.5 h-3.5 sm:mr-1.5" />
                                    <span className="hidden sm:inline">Discard</span>
                                </Button>
                                <Button size="sm" onClick={handleSave} disabled={!hasChanges || isSaving} className="bg-[#1fc7d4] text-white hover:bg-[#1fc7d4]/90">
                                    {isSaving && <Loader2 className="w-3 h-3 animate-spin mr-1.5" />}
                                    Save
                                </Button>
                            </>
                        )}
                        <Button variant="ghost" size="icon" className="h-7 w-7 shrink-0" onClick={onClose}>
                            <X className="h-4 w-4" />
                        </Button>
                    </div>
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
                            setHasChanges={setHasChanges}
                            permissions={permissions}
                            onPermissionsChanged={() => void reloadPermissions()}
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
