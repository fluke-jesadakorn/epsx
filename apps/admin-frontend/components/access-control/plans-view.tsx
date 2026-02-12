'use client';

import { Loader2 } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect } from 'react';
import { toast } from 'sonner';

import { updatePlanAction } from '@/app/wallet-management/plan-actions';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';

import { PlanListSidebar } from './plans/plan-list-sidebar';
import { FREE_PLAN_ID, type PlanEditFormState, type PlansViewProps } from './plans/types';
import {
    useLoadPlansAndPermissions,
    usePlanDragAndDrop,
} from './plans/use-plans-logic';

export function PlansView({ className }: PlansViewProps) {
    const router = useRouter();
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
    const {
        plans,
        isLoading: isLoadingData,
        setPlans,
        load: loadAllData,
    } = useLoadPlansAndPermissions();

    const noopSetForm = useCallback(
        (_: React.SetStateAction<PlanEditFormState>) => {},
        []
    ) as React.Dispatch<React.SetStateAction<PlanEditFormState>>;

    const {
        activeId,
        sensors,
        snapToCursor,
        handleDragStart,
        handleDragEnd,
    } = usePlanDragAndDrop({
        plans,
        setPlans,
        selectedPlan: null,
        setForm: noopSetForm,
    });

    const handleSelect = useCallback(
        (plan: PermissionPlan) => {
            router.push(`/wallet-management/access/plans/${plan.id}`);
        },
        [router]
    );

    const handleQuickToggle = useCallback(
        (e: React.MouseEvent, plan: PermissionPlan) => {
            e.stopPropagation();
            if (plan.id === FREE_PLAN_ID) {
                toast.error('Free Plan status cannot be changed');
                return;
            }
            const newState = !(plan.is_active === true);
            setPlans(
                plans.map((p) =>
                    p.id === plan.id ? { ...p, is_active: newState } : p
                )
            );
            void updatePlanAction(plan.id, { is_active: newState })
                .then(() =>
                    toast.success(
                        `Plan ${newState ? 'activated' : 'deactivated'}`
                    )
                )
                .catch(() => {
                    setPlans(
                        plans.map((p) =>
                            p.id === plan.id
                                ? { ...p, is_active: !newState }
                                : p
                        )
                    );
                    toast.error('Failed to update status');
                });
        },
        [plans, setPlans]
    );

    useEffect(() => {
        if (isAuthenticated) {
            void loadAllData();
        }
    }, [isAuthenticated, loadAllData]);

    if (authLoading || (isLoadingData && plans.length === 0)) {
        return (
            <div className="p-8 flex justify-center">
                <Loader2 className="animate-spin" />
            </div>
        );
    }

    return (
        <div className={cn('h-[calc(100vh-250px)] min-h-[500px]', className)}>
            <PlanListSidebar
                plans={plans}
                onSelect={handleSelect}
                onQuickToggle={handleQuickToggle}
                onRefresh={loadAllData}
                sensors={sensors}
                activeId={activeId}
                onDragStart={handleDragStart}
                onDragEnd={handleDragEnd}
                modifiers={[snapToCursor]}
            />
        </div>
    );
}
