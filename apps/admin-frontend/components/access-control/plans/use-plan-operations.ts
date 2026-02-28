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

import { createPlanAction, deletePlanAction, updatePlanAction } from '@/app/wallet-management/plan-actions';
import { type PermissionPlan } from '@/lib/api/plan-management-client';

import {
    type DragDropContext,
    FREE_PLAN_ID,
    type PlanDeletionContext,
    type PlanEditFormState,
} from './types';

export function usePlanDeletion(ctx: PlanDeletionContext) {
    const { plans, setPlans, selectedPlan, setSelectedPlan } = ctx;
    const [deleteConfirm, setDeleteConfirm] = useState<PermissionPlan | null>(null);
    const [confirmInput, setConfirmInput] = useState('');

    useEffect(() => {
        if (deleteConfirm === null) { setConfirmInput(''); }
    }, [deleteConfirm]);

    const deletePlan = async () => {
        if (deleteConfirm === null) { return; }
        try {
            await deletePlanAction(deleteConfirm.id);
            toast.success('Plan deleted');
            setPlans(plans.filter((p) => p.id !== deleteConfirm.id));
            if (selectedPlan?.id === deleteConfirm.id) { setSelectedPlan(null); }
            setDeleteConfirm(null);
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to delete plan');
        }
    };

    return { deleteConfirm, setDeleteConfirm, confirmInput, setConfirmInput, deletePlan };
}

export function usePlanDragAndDrop(ctx: DragDropContext) {
    const { plans, setPlans, selectedPlan, setForm } = ctx;
    const [activeId, setActiveId] = useState<string | null>(null);

    const sensors = useSensors(
        useSensor(PointerSensor, { activationConstraint: { distance: 1 } })
    );

    const snapToCursor = useCallback<Modifier>(
        ({ transform, activatorEvent, draggingNodeRect }) => {
            if (activatorEvent && draggingNodeRect) {
                const event = activatorEvent as unknown as PointerEvent;
                const offsetY = event.clientY - draggingNodeRect.top;
                return { ...transform, y: transform.y + offsetY - 10 };
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
        if (over?.id === undefined || active.id === over.id) { return; }

        const oldIndex = plans.findIndex((item) => item.id === active.id);
        const newIndex = plans.findIndex((item) => item.id === over.id);
        if (oldIndex === -1 || newIndex === -1) { return; }

        const newItems = arrayMove(plans, oldIndex, newIndex);
        setPlans(newItems);

        const updates = newItems.map((plan, index) => ({ id: plan.id, tier_level: index }));
        if (selectedPlan) {
            const updatedSelected = updates.find((u) => u.id === selectedPlan.id);
            if (updatedSelected) {
                setForm((prev) => ({ ...prev, priority: updatedSelected.tier_level }));
            }
        }

        await Promise.all(updates.map((u) => updatePlanAction(u.id, { tier_level: u.tier_level })))
            .then(() => { toast.success('Plans reordered'); })
            .catch(() => { toast.error('Failed to reorder plans'); });
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
                ctx.setPlans(ctx.plans.map((p) => p.id === plan.id ? { ...p, is_active: newState } : p));
                if (selectedPlan?.id === plan.id) {
                    setSelectedPlan({ ...selectedPlan, is_active: newState });
                    setForm((prev) => ({ ...prev, is_active: newState }));
                }
                await updatePlanAction(plan.id, { is_active: newState });
                toast.success(`Plan ${newState ? 'activated' : 'deactivated'}`);
            } catch {
                ctx.setPlans(ctx.plans.map((p) => p.id === plan.id ? { ...p, is_active: !newState } : p));
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
            plan_group: (formData.plan_group as 'personal' | 'enterprise' | 'api' | undefined) ?? 'personal',
        });
        toast.success('Created');
        onSuccess();
    } catch (err: unknown) {
        toast.error(err instanceof Error ? err.message : 'Failed to create plan');
        throw err;
    }
}
