import {
    closestCenter,
    DndContext,
    DragOverlay,
    type SensorDescriptor,
    type SensorOptions,
} from '@dnd-kit/core';
import {
    SortableContext,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { Search } from 'lucide-react';
import React, { useMemo, useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { type PermissionPlan } from '@/lib/api/plan-management-client';

import { CreatePlanSheet } from './create-plan-sheet';
import { PlanItem, SortablePlanItem } from './plan-item';
import { FREE_PLAN_ID } from './types';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export interface PlanListSidebarProps {
    plans: PermissionPlan[];
    selectedPlanId?: string;
    onSelect: (plan: PermissionPlan) => void;
    onQuickToggle: (e: React.MouseEvent, plan: PermissionPlan) => void;
    onRefresh: () => void;
    // Dnd props
    sensors: SensorDescriptor<SensorOptions>[];
    activeId: string | null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onDragStart: (event: any) => void;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onDragEnd: (event: any) => void;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    modifiers: any[];
}

export function PlanListSidebar({
    plans,
    selectedPlanId,
    onSelect,
    onQuickToggle,
    onRefresh,
    sensors,
    activeId,
    onDragStart,
    onDragEnd,
    modifiers,
}: PlanListSidebarProps) {
    const [planSearch, setPlanSearch] = useState('');
    const [isCreatePlanOpen, setIsCreatePlanOpen] = useState(false);

    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions -- plan.name check
    const filteredPlans = useMemo(
        () =>
            plans.filter((p) =>
                (p.name ?? '').toLowerCase().includes(planSearch.toLowerCase())
            ),
        [plans, planSearch]
    );

    const isSearchActive = planSearch.length > 0;

    return (
        <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[24px] overflow-hidden flex flex-col h-full">
            <CardHeader className="p-4 bg-white/5 border-b border-white/5 space-y-4 shrink-0">
                <div className="flex items-center justify-between">
                    <h2 className="text-sm font-bold flex items-center gap-2 uppercase tracking-wider text-muted-foreground">
                        Active Plans
                    </h2>
                    <Badge variant="secondary" className="bg-cyan-500/10 text-[#1fc7d4]">
                        {plans.length}
                    </Badge>
                </div>
                <div className="flex gap-2">
                    <div className="relative flex-1">
                        <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground/50" />
                        <Input
                            placeholder="Search plans..."
                            value={planSearch}
                            onChange={(e) => setPlanSearch(e.target.value)}
                            className="h-9 pl-9 text-sm bg-white/5 border-white/5"
                        />
                    </div>
                    <CreatePlanSheet
                        open={isCreatePlanOpen}
                        onOpenChange={setIsCreatePlanOpen}
                        onSuccess={onRefresh}
                    />
                </div>
            </CardHeader>
            <CardContent className="p-0 overflow-y-auto flex-1 scrollbar-thin scrollbar-thumb-white/10">
                {isSearchActive ? (
                    <div className="divide-y divide-white/5">
                        {filteredPlans.map((plan, index) => (
                            <SortablePlanItem
                                key={plan.id}
                                plan={plan}
                                index={index}
                                selectedPlanId={selectedPlanId}
                                onSelect={onSelect}
                                isFreePlan={plan.id === FREE_PLAN_ID}
                                onQuickToggle={onQuickToggle}
                                disabled={true} // Disable drag when searching
                            />
                        ))}
                    </div>
                ) : (
                    <DndContext
                        sensors={sensors}
                        collisionDetection={closestCenter}
                        onDragStart={onDragStart}
                        onDragEnd={onDragEnd}
                        modifiers={modifiers}
                    >
                        <SortableContext
                            items={filteredPlans.map((p) => p.id)}
                            strategy={verticalListSortingStrategy}
                        >
                            <div className="divide-y divide-white/5">
                                {filteredPlans.map((plan, index) => (
                                    <SortablePlanItem
                                        key={plan.id}
                                        plan={plan}
                                        index={index}
                                        selectedPlanId={selectedPlanId}
                                        onSelect={onSelect}
                                        isFreePlan={plan.id === FREE_PLAN_ID}
                                        onQuickToggle={onQuickToggle}
                                    />
                                ))}
                            </div>
                        </SortableContext>
                        <DragOverlay>
                            {activeId ? (
                                <div className="bg-slate-800/90 backdrop-blur border border-cyan-500/30 shadow-2xl rounded-lg overflow-hidden cursor-grabbing">
                                    {(() => {
                                        const planIndex = plans.findIndex((p) => p.id === activeId);
                                        const plan = plans[planIndex];
                                        if (!plan) {
                                            return null;
                                        }
                                        return (
                                            <PlanItem
                                                plan={plan}
                                                index={planIndex}
                                                isFreePlan={plan.id === FREE_PLAN_ID}
                                                disabled={false} // Always show handle
                                                dragHandleProps={{}}
                                            />
                                        );
                                    })()}
                                </div>
                            ) : null}
                        </DragOverlay>
                    </DndContext>
                )}
            </CardContent>
        </Card>
    );
}
