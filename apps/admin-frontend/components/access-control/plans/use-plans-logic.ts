import {
    type DragEndEvent,
    type DragStartEvent,
    type Modifier,
    PointerSensor,
    useSensor,
    useSensors,
} from '@dnd-kit/core';
import { arrayMove } from '@dnd-kit/sortable';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import {
    createPlanAction,
    deletePlanAction,
    getPlansAction,
    updatePlanAction,
} from '@/app/wallet-management/plan-actions';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { logger } from '@/shared/utils/logger';

import {
    type DragDropContext,
    FREE_PLAN_ID,
    getFeatureValue,
    type PlanDeletionContext,
    type PlanEditFormState,
} from './types';

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
            if (permRes.success && permRes.data) {
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

/** datetime-local input value → RFC3339 UTC string for backend */
function toIso(dt: string): string {
    if (!dt) { return ''; }
    if (dt.includes('Z') || /[+-]\d{2}:\d{2}$/.test(dt)) { return dt; }
    // YYYY-MM-DDTHH:mm:ss → append Z; YYYY-MM-DDTHH:mm → append :00Z
    return dt.length === 16 ? `${dt}:00Z` : `${dt}Z`;
}

/** RFC3339 UTC string → datetime-local input value (YYYY-MM-DDTHH:mm) */
function toLocal(iso: string): string {
    if (!iso) { return ''; }
    return iso.slice(0, 16);
}

export function usePlanEditForm() {
    const [selectedPlan, setSelectedPlan] = useState<PermissionPlan | null>(null);
    const [form, setForm] = useState<PlanEditFormState>({
        name: '',
        description: '',
        plan_category: 'base',
        plan_group: 'personal',
        priority: 0,
        price: 0,
        expiryDays: 30,
        gracePeriodHours: 0,
        permissions: [],
        is_public: true,
        is_active: true,
        features: [],
        promoEnabled: false,
        promoType: 'percentage',
        promoValue: 0,
        promoPrice: 0,
        promoStart: '',
        promoEnd: '',
    });
    const [hasChanges, setHasChanges] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    const selectPlanLogic = useCallback((plan: PermissionPlan | null) => {
        if (!plan) {
            setSelectedPlan(null);
            setForm({
                name: '',
                description: '',
                plan_category: 'base',
                plan_group: 'personal',
                priority: 0,
                price: 0,
                expiryDays: 30,
                gracePeriodHours: 0,
                permissions: [],
                is_public: true,
                is_active: true,
                features: [],
                promoEnabled: false,
                promoType: 'percentage',
                promoValue: 0,
                promoPrice: 0,
                promoStart: '',
                promoEnd: '',
            });
        } else {
            setSelectedPlan(plan);
            const features = Array.isArray(plan.plan_metadata?.features)
                ? (plan.plan_metadata?.features as string[])
                : [];
            const promo = plan.plan_metadata?.promotion as Record<string, unknown> | undefined;
            setForm({
                name: plan.name,
                description: plan.description,
                plan_category: plan.plan_category ?? 'base',
                plan_group: plan.plan_group ?? 'personal',
                priority: plan.tier_level ?? 0,
                price: Number(plan.price) || 0,
                expiryDays: plan.default_expiry_days ?? 30,
                gracePeriodHours: plan.grace_period_hours ?? 0,
                permissions: plan.permissions ?? [],
                is_public: plan.is_public !== false,
                is_active: plan.is_active !== false,
                features,
                promoEnabled: promo?.enabled === true,
                promoType: (promo?.type as 'percentage' | 'fixed') ?? 'percentage',
                promoValue: Number(promo?.value) || 0,
                promoPrice: Number(promo?.price) || 0,
                promoStart: toLocal((promo?.start_date as string) ?? ''),
                promoEnd: toLocal((promo?.end_date as string) ?? ''),
            });
        }
        setHasChanges(false);
    }, []);

    const savePlan = async (
        plans: PermissionPlan[],
        setPlans: (p: PermissionPlan[]) => void
    ) => {
        if (!selectedPlan) {
            return;
        }
        setIsSaving(true);
        try {
            const offsetStr = getFeatureValue(form.permissions, 'epsx:rankings:offset');
            const limitStr = getFeatureValue(form.permissions, 'epsx:rankings:limit');
            const rankingOffset = offsetStr !== null ? parseInt(offsetStr, 10) : undefined;
            const rankingsLimit = limitStr !== null ? parseInt(limitStr, 10) : undefined;

            const updated = await updatePlanAction(selectedPlan.id, {
                name: form.name,
                description: form.description,
                plan_category: form.plan_category,
                plan_group: form.plan_group,
                tier_level: form.priority,
                price: selectedPlan.id === FREE_PLAN_ID ? undefined : form.price,
                default_expiry_days: form.expiryDays,
                grace_period_hours: form.gracePeriodHours,
                permissions: form.permissions,
                is_public: form.is_public,
                is_active: form.is_active,
                plan_metadata: {
                    ...selectedPlan.plan_metadata,
                    ...(rankingOffset !== undefined && !isNaN(rankingOffset) ? { ranking_offset: rankingOffset } : {}),
                    ...(rankingsLimit !== undefined && !isNaN(rankingsLimit) ? { rankings_limit: rankingsLimit } : {}),
                    features: form.features,
                    promotion: {
                        enabled: form.promoEnabled,
                        type: form.promoType,
                        value: form.promoValue,
                        price: form.promoPrice,
                        start_date: toIso(form.promoStart),
                        end_date: toIso(form.promoEnd),
                    },
                },
            });
            toast.success('Plan updated');
            setPlans(plans.map((p) => (p.id === updated.id ? updated : p)));
            setSelectedPlan(updated);
            setHasChanges(false);
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to update plan');
        } finally {
            setIsSaving(false);
        }
    };

    const discardChanges = () => {
        if (selectedPlan) {
            selectPlanLogic(selectedPlan);
            toast.info('Changes discarded');
        }
    };

    return {
        selectedPlan,
        form,
        setForm,
        hasChanges,
        setHasChanges,
        isSaving,
        selectPlan: selectPlanLogic,
        savePlan,
        discardChanges,
    };
}

export function usePlanDeletion(ctx: PlanDeletionContext) {
    const { plans, setPlans, selectedPlan, setSelectedPlan } = ctx;
    const [deleteConfirm, setDeleteConfirm] = useState<PermissionPlan | null>(
        null
    );
    const [confirmInput, setConfirmInput] = useState('');

    useEffect(() => {
        if (!deleteConfirm) {
            setConfirmInput('');
        }
    }, [deleteConfirm]);

    const deletePlan = async () => {
        if (!deleteConfirm) {
            return;
        }
        try {
            await deletePlanAction(deleteConfirm.id);
            toast.success('Plan deleted');
            setPlans(plans.filter((p) => p.id !== deleteConfirm.id));
            if (selectedPlan?.id === deleteConfirm.id) {
                setSelectedPlan(null);
            }
            setDeleteConfirm(null);
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to delete plan');
        }
    };

    return {
        deleteConfirm,
        setDeleteConfirm,
        confirmInput,
        setConfirmInput,
        deletePlan,
    };
}

export function usePlanDragAndDrop(ctx: DragDropContext) {
    const { plans, setPlans, selectedPlan, setForm } = ctx;
    const [activeId, setActiveId] = useState<string | null>(null);

    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: { distance: 1 },
        })
    );

    const snapToCursor = useCallback<Modifier>(
        ({ transform, activatorEvent, draggingNodeRect }) => {
            if (activatorEvent && draggingNodeRect) {
                const event = activatorEvent as unknown as PointerEvent;

                const offsetY = event.clientY - draggingNodeRect.top;
                return {
                    ...transform,
                    y: transform.y + offsetY - 10,
                };
            }
            return transform;
        },
        []
    );

    const handleDragStart = (event: DragStartEvent) => {
        setActiveId(event.active.id as string);
    };

    const handleDragEnd = async (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveId(null);

        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions -- over?.id check
        if (!over?.id || active.id === over.id) {
            return;
        }

        const oldIndex = plans.findIndex((item) => item.id === active.id);
        const newIndex = plans.findIndex((item) => item.id === over.id);

        if (oldIndex === -1 || newIndex === -1) {
            return;
        }

        const newItems = arrayMove(plans, oldIndex, newIndex);
        setPlans(newItems);

        const updates = newItems.map((plan, index) => ({
            id: plan.id,
            tier_level: index,
        }));

        if (selectedPlan) {
            const updatedSelected = updates.find((u) => u.id === selectedPlan.id);
            if (updatedSelected) {
                setForm((prev) => ({
                    ...prev,
                    priority: updatedSelected.tier_level,
                }));
            }
        }

        await Promise.all(
            updates.map((u) =>
                updatePlanAction(u.id, { tier_level: u.tier_level })
            )
        )
            .then(() => {
                toast.success('Plans reordered');
            })
            .catch(() => {
                toast.error('Failed to reorder plans');
            });
    };

    return { activeId, sensors, snapToCursor, handleDragStart, handleDragEnd };
}

export function useQuickTogglePlan(ctx: {
    plans: PermissionPlan[];
    setPlans: (p: PermissionPlan[]) => void;
}) {
    return useCallback(
        async (params: {
            e: React.MouseEvent;
            plan: PermissionPlan;
            selectedPlan: PermissionPlan | null;
            setSelectedPlan: (p: PermissionPlan | null) => void;
            setForm: (f: (prev: PlanEditFormState) => PlanEditFormState) => void;
        }) => {
            const { e, plan, selectedPlan, setSelectedPlan, setForm } = params;
            e.stopPropagation();
            if (plan.id === FREE_PLAN_ID) {
                toast.error('Constant Free Plan status cannot be changed');
                return;
            }

            const newState = !(plan.is_active === true);
            try {
                ctx.setPlans(
                    ctx.plans.map((p) =>
                        p.id === plan.id ? { ...p, is_active: newState } : p
                    )
                );
                if (selectedPlan?.id === plan.id) {
                    setSelectedPlan({ ...selectedPlan, is_active: newState });
                    setForm((prev) => ({ ...prev, is_active: newState }));
                }
                await updatePlanAction(plan.id, { is_active: newState });
                toast.success(`Plan ${newState ? 'activated' : 'deactivated'}`);
            } catch {
                ctx.setPlans(
                    ctx.plans.map((p) =>
                        p.id === plan.id ? { ...p, is_active: !newState } : p
                    )
                );
                if (selectedPlan?.id === plan.id) {
                    setSelectedPlan({ ...selectedPlan, is_active: !newState });
                    setForm((prev) => ({ ...prev, is_active: !newState }));
                }
                toast.error('Failed to update status');
            }
        },
        [ctx]
    );
}

export async function submitCreatePlan(
    formData: {
        name: string;
        description: string;
        priority: number;
        price: number;
        default_expiry_days: number;
        permissions?: string[];
        plan_group?: string;
    },
    onSuccess: () => void
) {
    try {
        await createPlanAction({
            name: formData.name,
            description: formData.description,
            permissions: formData.permissions ?? [],
            tier_level: formData.priority,
            price: formData.price,
            default_expiry_days: formData.default_expiry_days,
            plan_group: (formData.plan_group as 'personal' | 'enterprise' | 'api') ?? 'personal',
        });
        toast.success('Created');
        onSuccess();
    } catch (err: unknown) {
        toast.error(err instanceof Error ? err.message : 'Failed to create plan');
        throw err;
    }
}
