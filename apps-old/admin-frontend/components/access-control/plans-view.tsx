'use client';

import { Loader2 } from 'lucide-react';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { useCallback, useEffect, useRef } from 'react';

import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';

import { PlanEditorDrawer } from './plans/plan-editor-drawer';
import { PlanListSidebar } from './plans/plan-list-sidebar';
import { type PlanEditFormState, type PlansViewProps } from './plans/types';
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
        <div className={cn('relative', className)}>
            <PlanListSidebar
                plans={plans}
                selectedPlanId={selectedPlanId ?? undefined}
                onSelect={handleSelect}
                onRefresh={() => { void loadAllData(); }}
                duplicateRef={duplicateRef}
                sensors={sensors}
                activeId={activeId}
                onDragStart={handleDragStart}
                onDragEnd={(e) => { void handleDragEnd(e); }}
                modifiers={[snapToCursor]}
            />
            {isOpen && (
                <PlanEditorDrawer
                    planId={selectedPlanId}
                    onClose={handleClose}
                    onPlanUpdated={handlePlanUpdated}
                    onDuplicate={handleDuplicateFromEditor}
                />
            )}
        </div>
    );
}
