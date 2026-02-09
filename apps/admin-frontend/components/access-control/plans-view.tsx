'use client';

import { Loader2 } from 'lucide-react';
import { useEffect } from 'react';

import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';

import { DeletePlanDialog } from './plans/delete-plan-dialog';
import { PlanEditor } from './plans/plan-editor';
import { PlanListSidebar } from './plans/plan-list-sidebar';
import { type PlansViewProps } from './plans/types';
import {
    useLoadPlansAndPermissions,
    usePlanDeletion,
    usePlanDragAndDrop,
    usePlanEditForm,
    useQuickTogglePlan,
} from './plans/use-plans-logic';

export function PlansView({ className }: PlansViewProps) {
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
    const {
        permissions,
        plans,
        isLoading: isLoadingData,
        setPlans,
        load: loadAllData,
    } = useLoadPlansAndPermissions();

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

    const {
        deleteConfirm,
        setDeleteConfirm,
        deletePlan,
    } = usePlanDeletion({
        plans,
        setPlans,
        selectedPlan,
        setSelectedPlan: selectPlan,
    });

    const {
        activeId,
        sensors,
        snapToCursor,
        handleDragStart,
        handleDragEnd,
    } = usePlanDragAndDrop({ plans, setPlans, selectedPlan, setForm });

    const handleQuickToggle = useQuickTogglePlan({ plans, setPlans });

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
        <div
            className={cn(
                'grid grid-cols-1 lg:grid-cols-12 gap-6 h-[calc(100vh-250px)] min-h-[500px]',
                className
            )}
        >
            <div className="lg:col-span-4 flex flex-col gap-4 h-full">
                <PlanListSidebar
                    plans={plans}
                    selectedPlanId={selectedPlan?.id}
                    onSelect={selectPlan}
                    onQuickToggle={handleQuickToggle}
                    onRefresh={loadAllData}
                    sensors={sensors}
                    activeId={activeId}
                    onDragStart={handleDragStart}
                    onDragEnd={handleDragEnd}
                    modifiers={[snapToCursor]}
                />
            </div>

            <div className="lg:col-span-8 h-full">
                <PlanEditor
                    selectedPlan={selectedPlan}
                    form={form}
                    setForm={setForm}
                    hasChanges={hasChanges}
                    setHasChanges={setHasChanges}
                    isSaving={isSaving}
                    onSave={() => {
                        void savePlan(plans, setPlans);
                    }}
                    onDiscard={discardChanges}
                    onDelete={() => setDeleteConfirm(selectedPlan)}
                    permissions={permissions}
                />
            </div>

            <DeletePlanDialog
                planToDelete={deleteConfirm}
                onClose={() => setDeleteConfirm(null)}
                onConfirm={() => {
                    void deletePlan();
                }}
            />
        </div>
    );
}
