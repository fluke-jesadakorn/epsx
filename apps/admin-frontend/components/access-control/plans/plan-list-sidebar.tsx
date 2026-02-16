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
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { type PermissionPlan, type PlanGroup } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';

import { CreatePlanSheet } from './create-plan-sheet';
import { PlanItem, SortablePlanItem } from './plan-item';
import { FREE_PLAN_ID } from './types';

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
    const [sourcePlan, setSourcePlan] = useState<PermissionPlan | null>(null);
    const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

    const handleDuplicate = useCallback((plan: PermissionPlan) => {
        setSourcePlan(plan);
        setIsCreatePlanOpen(true);
    }, []);

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

    const renderPlans = (groupPlans: PermissionPlan[], disabled: boolean) => (
        <div className="divide-y divide-white/5">
            {groupPlans.map((plan, index) => (
                <SortablePlanItem
                    key={plan.id}
                    plan={plan}
                    index={index}
                    selectedPlanId={selectedPlanId}
                    onSelect={onSelect}
                    onDuplicate={handleDuplicate}
                    isFreePlan={plan.id === FREE_PLAN_ID}
                    onQuickToggle={onQuickToggle}
                    disabled={disabled}
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
                className="w-full flex items-center gap-2 px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground hover:bg-white/5 transition-colors"
            >
                {cfg.icon}
                <span className="flex-1 text-left">{cfg.label}</span>
                <Badge variant="secondary" className="bg-white/5 text-muted-foreground text-[10px] px-1.5 py-0">
                    {count}
                </Badge>
                <ChevronDown className={cn('h-3 w-3 transition-transform', isCollapsed && '-rotate-90')} />
            </button>
        );
    };

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
                        sourcePlan={sourcePlan}
                        onSourceClear={clearSource}
                    />
                </div>
            </CardHeader>
            <CardContent className="p-0 overflow-y-auto flex-1 scrollbar-thin scrollbar-thumb-white/10">
                {isSearchActive ? (
                    <>
                        {GROUP_ORDER.map((g) => {
                            const gPlans = grouped[g];
                            if (gPlans.length === 0) return null;
                            return (
                                <div key={g}>
                                    {renderGroupHeader(g, gPlans.length)}
                                    {collapsed[g] !== true && renderPlans(gPlans, true)}
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
                            if (gPlans.length === 0) return null;
                            return (
                                <div key={g}>
                                    {renderGroupHeader(g, gPlans.length)}
                                    {collapsed[g] !== true && (
                                        <SortableContext
                                            items={gPlans.map((p) => p.id)}
                                            strategy={verticalListSortingStrategy}
                                        >
                                            {renderPlans(gPlans, false)}
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
                                                isFreePlan={plan.id === FREE_PLAN_ID}
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
            </CardContent>
        </Card>
    );
}
