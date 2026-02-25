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
import { Building2, ChevronDown, Code2, Search, User, Wrench } from 'lucide-react';
import React, { useCallback, useMemo, useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { type PermissionPlan, type PlanGroup } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';

import { CreatePlanSheet } from './create-plan-sheet';
import { PlanItem, SortablePlanItem } from './plan-item';
import { isSystemPlan } from './types';

const GROUP_CONFIG: Record<PlanGroup, { label: string; icon: React.ReactNode }> = {
    personal: { label: 'Personal Plans', icon: <User className="h-3.5 w-3.5" /> },
    enterprise: { label: 'Enterprise Plans', icon: <Building2 className="h-3.5 w-3.5" /> },
    api: { label: 'API Plans', icon: <Code2 className="h-3.5 w-3.5" /> },
    custom: { label: 'Custom Plans', icon: <Wrench className="h-3.5 w-3.5" /> },
};

const GROUP_ORDER: PlanGroup[] = ['personal', 'enterprise', 'api', 'custom'];

export interface PlanListSidebarProps {
    plans: PermissionPlan[];
    selectedPlanId?: string;
    onSelect: (plan: PermissionPlan) => void;
    onRefresh: () => void;
    // External duplicate trigger (from editor drawer)
    duplicateRef?: React.MutableRefObject<((plan: PermissionPlan) => void) | null>;
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
    onRefresh,
    duplicateRef,
    sensors,
    activeId,
    onDragStart,
    onDragEnd,
    modifiers,
}: PlanListSidebarProps) {
    const [planSearch, setPlanSearch] = useState('');
    const [isCreatePlanOpen, setIsCreatePlanOpen] = useState(false);
    const [sourcePlan, setSourcePlan] = useState<PermissionPlan | null>(null);
    const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

    const handleDuplicate = useCallback((plan: PermissionPlan) => {
        setSourcePlan(plan);
        setIsCreatePlanOpen(true);
    }, []);

    // Expose duplicate trigger to parent via ref
    React.useEffect(() => {
        if (duplicateRef) {
            duplicateRef.current = handleDuplicate;
        }
    }, [duplicateRef, handleDuplicate]);

    const clearSource = useCallback(() => {
        setSourcePlan(null);
    }, []);

    const filteredPlans = useMemo(
        () =>
            plans.filter((p) =>
                (p.name ?? '').toLowerCase().includes(planSearch.toLowerCase())
            ),
        [plans, planSearch]
    );

    const grouped = useMemo(() => {
        const map: Record<PlanGroup, PermissionPlan[]> = { personal: [], enterprise: [], api: [], custom: [] };
        for (const p of filteredPlans) {
            const g = p.plan_group ?? 'personal';
            (map[g] ??= []).push(p);
        }
        return map;
    }, [filteredPlans]);

    const toggleCollapse = useCallback((g: string) => {
        setCollapsed((prev) => ({ ...prev, [g]: !prev[g] }));
    }, []);

    const isSearchActive = planSearch.length > 0;

    const renderPlans = (groupPlans: PermissionPlan[], group: PlanGroup, disabled: boolean) => (
        <div className="divide-y divide-white/5">
            {groupPlans.map((plan, index) => (
                <SortablePlanItem
                    key={plan.id}
                    plan={plan}
                    index={index}
                    group={group}
                    selectedPlanId={selectedPlanId}
                    onSelect={onSelect}
                    onDuplicate={handleDuplicate}
                    disabled={disabled || isSystemPlan(plan)}
                />
            ))}
        </div>
    );

    const renderGroupHeader = (group: PlanGroup, count: number) => {
        const cfg = GROUP_CONFIG[group];
        const isCollapsed = collapsed[group] === true;
        return (
            <button
                type="button"
                onClick={() => toggleCollapse(group)}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground hover:bg-muted/30 transition-colors"
            >
                {cfg.icon}
                <span className="flex-1 text-left">{cfg.label}</span>
                <Badge variant="secondary" className="bg-muted/30 text-muted-foreground text-[10px] px-1.5 py-0">
                    {count}
                </Badge>
                <ChevronDown className={cn('h-3 w-3 transition-transform', isCollapsed && '-rotate-90')} />
            </button>
        );
    };

    return (
        <div>
            <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold flex items-center gap-2 uppercase tracking-wider text-muted-foreground">
                    Active Plans
                    <Badge variant="secondary" className="bg-cyan-500/10 text-[#1fc7d4]">
                        {plans.length}
                    </Badge>
                </h2>
                <div className="flex items-center gap-2">
                    <div className="relative">
                        <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground/50" />
                        <Input
                            placeholder="Search plans..."
                            value={planSearch}
                            onChange={(e) => setPlanSearch(e.target.value)}
                            className="h-9 w-56 pl-9 text-sm bg-muted/30 border-border/20 rounded-lg"
                        />
                    </div>
                    <CreatePlanSheet
                        open={isCreatePlanOpen}
                        onOpenChange={setIsCreatePlanOpen}
                        onSuccess={onRefresh}
                        sourcePlan={sourcePlan}
                        onSourceClear={clearSource}
                    />
                </div>
            </div>
            <div className="divide-y divide-white/5 border border-border/20 rounded-xl overflow-hidden">
                {isSearchActive ? (
                    <>
                        {GROUP_ORDER.map((g) => {
                            const gPlans = grouped[g];
                            if (gPlans.length === 0) {return null;}
                            return (
                                <div key={g}>
                                    {renderGroupHeader(g, gPlans.length)}
                                    {collapsed[g] !== true && renderPlans(gPlans, g, true)}
                                </div>
                            );
                        })}
                    </>
                ) : (
                    <DndContext
                        sensors={sensors}
                        collisionDetection={closestCenter}
                        onDragStart={onDragStart}
                        onDragEnd={onDragEnd}
                        modifiers={modifiers}
                    >
                        {GROUP_ORDER.map((g) => {
                            const gPlans = grouped[g];
                            if (gPlans.length === 0) {return null;}
                            return (
                                <div key={g}>
                                    {renderGroupHeader(g, gPlans.length)}
                                    {collapsed[g] !== true && (
                                        <SortableContext
                                            items={gPlans.map((p) => p.id)}
                                            strategy={verticalListSortingStrategy}
                                        >
                                            {renderPlans(gPlans, g, false)}
                                        </SortableContext>
                                    )}
                                </div>
                            );
                        })}
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
                                                disabled={false}
                                                dragHandleProps={{}}
                                            />
                                        );
                                    })()}
                                </div>
                            ) : null}
                        </DragOverlay>
                    </DndContext>
                )}
            </div>
        </div>
    );
}
