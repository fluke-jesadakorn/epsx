'use client';

import { Loader2 } from 'lucide-react';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { useCallback, useEffect, useRef } from 'react';
import { toast } from 'sonner';

import { updatePlanAction } from '@/app/wallet-management/plan-actions';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';

import { PlanEditorDrawer } from './plans/plan-editor-drawer';
import { PlanListSidebar } from './plans/plan-list-sidebar';
import { FREE_PLAN_ID, type PlanEditFormState, type PlansViewProps } from './plans/types';
import {
    useLoadPlansAndPermissions,
    usePlanDragAndDrop,
} from './plans/use-plans-logic';

export function PlansView({ className }: PlansViewProps) {
    const router = useRouter();
    const pathname = usePathname();
    const searchParams = useSearchParams();
    const selectedPlanId = searchParams.get('planId');
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
    const duplicateRef = useRef<((plan: PermissionPlan) => void) | null>(null);
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
            const params = new URLSearchParams(searchParams.toString());
            params.set('planId', plan.id);
            router.replace(`${pathname}?${params.toString()}`);
        },
        [router, pathname, searchParams]
    );

    const handleClose = useCallback(() => {
        const params = new URLSearchParams(searchParams.toString());
        params.delete('planId');
        const qs = params.toString();
        router.replace(qs ? `${pathname}?${qs}` : pathname);
    }, [router, pathname, searchParams]);

    const handlePlanUpdated = useCallback(() => {
        void loadAllData();
    }, [loadAllData]);

    const handleDuplicateFromEditor = useCallback((plan: PermissionPlan) => {
        duplicateRef.current?.(plan);
    }, []);

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

    const isOpen = selectedPlanId !== null;

    return (
        <div className={cn('h-[calc(100vh-250px)] min-h-[500px] flex gap-4', className)}>
            <div className={cn(
                'transition-all duration-300 h-full',
                isOpen ? 'w-[380px] shrink-0' : 'w-full'
            )}>
                <PlanListSidebar
                    plans={plans}
                    selectedPlanId={selectedPlanId ?? undefined}
                    onSelect={handleSelect}
                    onQuickToggle={handleQuickToggle}
                    onRefresh={loadAllData}
                    duplicateRef={duplicateRef}
                    sensors={sensors}
                    activeId={activeId}
                    onDragStart={handleDragStart}
                    onDragEnd={handleDragEnd}
                    modifiers={[snapToCursor]}
                />
            </div>
            {isOpen && (
                <div className="flex-1 min-w-0 h-full animate-in slide-in-from-right-4 duration-300">
                    <PlanEditorDrawer
                        planId={selectedPlanId}
                        onClose={handleClose}
                        onPlanUpdated={handlePlanUpdated}
                        onDuplicate={handleDuplicateFromEditor}
                    />
                </div>
            )}
        </div>
    );
}
