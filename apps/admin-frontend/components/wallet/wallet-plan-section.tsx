'use client';

import { useDroppable } from '@dnd-kit/core';
import { Package, Plus, Trash2, Users } from 'lucide-react';
import { useState } from 'react';

import { AddResourceModal } from './add-resource-modal';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { AccessItem } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';

interface WalletPlanSectionProps {
    plans: AccessItem[];
    availablePlans: AccessItem[];
    onAddPlan: (plan: AccessItem) => Promise<void>;
    onRemovePlan: (planId: string) => Promise<void>;
    isLoading?: boolean;
    pendingDrops?: Record<string, AccessItem[]>; // Map planID -> List of pending permissions
}

function PlanCard({
    plan,
    onRemove,
    pendingPermissions = []
}: {
    plan: AccessItem;
    onRemove: () => void;
    pendingPermissions?: AccessItem[];
}) {
    const { setNodeRef, isOver } = useDroppable({
        id: plan.id,
        data: { type: 'plan', name: plan.name }
    });

    return (
        <div
            ref={setNodeRef}
            className={cn(
                "group relative flex flex-col gap-2 p-4 rounded-xl border transition-all",
                isOver
                    ? "bg-blue-50 border-blue-400 dark:bg-blue-900/20 dark:border-blue-500 shadow-md ring-1 ring-blue-400 scale-[1.01]"
                    : "bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
            )}
        >
            <div className="flex items-start justify-between gap-3">
                <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center flex-shrink-0">
                        <Users className="h-5 w-5 text-purple-600 dark:text-purple-400" />
                    </div>
                    <div>
                        <h4 className="font-semibold text-gray-900 dark:text-white">
                            {plan.name}
                        </h4>
                        <p className="text-sm text-gray-500 dark:text-gray-400">
                            {plan.description ?? 'No description'}
                        </p>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={onRemove}
                        className="h-8 w-8 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                        title="Remove Plan"
                    >
                        <Trash2 className="h-4 w-4" />
                    </Button>
                </div>
            </div>

            {/* Dropped/Pending Permissions */}
            {pendingPermissions.length > 0 && (
                <div className="mt-2 pl-13 animate-in fade-in slide-in-from-top-1">
                    <div className="flex flex-wrap gap-2">
                        {pendingPermissions.map(p => (
                            <Badge
                                key={p.id}
                                variant="outline"
                                className="bg-blue-50 text-blue-700 border-blue-200 gap-1 pl-1 pr-2"
                            >
                                <span className="bg-blue-200 text-blue-800 text-[10px] px-1 rounded-sm">+</span>
                                {p.name}
                            </Badge>
                        ))}
                    </div>
                </div>
            )}

            {/* Drop Zone Check Indicator */}
            {isOver && (
                <div className="absolute inset-0 bg-blue-500/5 pointer-events-none rounded-xl flex items-center justify-center">
                    <Badge className="bg-blue-600 text-white shadow-lg scale-110">
                        Drop to Assign
                    </Badge>
                </div>
            )}
        </div>
    );
}

export function WalletPlanSection({
    plans,
    availablePlans,
    onAddPlan,
    onRemovePlan,
    isLoading,
    pendingDrops = {}
}: WalletPlanSectionProps) {
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [search, setSearch] = useState('');

    const filteredPlans = plans.filter(p =>
        p.name.toLowerCase().includes(search.toLowerCase()) ||
        p.description?.toLowerCase().includes(search.toLowerCase())
    );

    return (
        <div className="space-y-4">
            {/* Header */}
            <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 flex flex-col sm:flex-row items-center justify-between gap-4">
                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <Package className="h-5 w-5 text-gray-500" />
                    <h3 className="font-semibold text-gray-900 dark:text-white">
                        Access Plans
                    </h3>
                    <Badge variant="secondary" className="ml-2">
                        {plans.length}
                    </Badge>
                </div>

                <div className="flex items-center gap-3 w-full sm:w-auto">
                    <Input
                        placeholder="Search plans..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="h-9 w-full sm:w-48 lg:w-64"
                    />
                    <Button
                        size="sm"
                        className="gap-2 whitespace-nowrap"
                        onClick={() => setIsAddModalOpen(true)}
                    >
                        <Plus className="h-4 w-4" />
                        Add Plan
                    </Button>
                </div>
            </div>

            {/* List */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {filteredPlans.length === 0 ? (
                    <div className="col-span-full py-12 text-center text-gray-500 border border-dashed border-gray-200 dark:border-gray-700 rounded-xl bg-gray-50/50 dark:bg-gray-900/50">
                        <Package className="h-10 w-10 mx-auto mb-3 opacity-20" />
                        <p>No plans assigned</p>
                    </div>
                ) : (
                    filteredPlans.map(plan => (
                        <PlanCard
                            key={plan.id}
                            plan={plan}
                            onRemove={() => onRemovePlan(plan.id)}
                            pendingPermissions={pendingDrops[plan.id]}
                        />
                    ))
                )}
            </div>

            <AddResourceModal
                isOpen={isAddModalOpen}
                onClose={() => setIsAddModalOpen(false)}
                title="Add Access Plan"
                description="Assign this wallet to an access plan to inherit its permissions."
                items={availablePlans}
                onConfirm={onAddPlan}
                isLoading={isLoading}
                emptyMessage="No available plans found"
            />
        </div>
    );
}
