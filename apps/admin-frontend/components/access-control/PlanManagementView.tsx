/**
 * Plan Management View Component
 * Copied from wallet-management/[address]/page.tsx Plan Builder section
 * For use in Access Control tab
 */
'use client';

import {
    DndContext,
    DragEndEvent,
    DragOverlay,
    DragStartEvent,
    MouseSensor,
    TouchSensor,
    useSensor,
    useSensors
} from '@dnd-kit/core';
import {
    ArrowLeft,
    Key,
    Loader2,
    Package,
    Plus,
    Save,
    Search
} from 'lucide-react';
import { useMemo, useState } from 'react';
import { toast } from 'sonner';

import { updatePlanAction } from '@/app/wallet-management/plan-actions';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { DraggablePermissionItem, DroppablePermissionList } from '@/components/wallet/WalletComponents';
import { AccessItem, useWalletAccess } from '@/hooks/useWalletAccess';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';

interface PlanManagementViewProps {
    className?: string;
}

export function PlanManagementView({ className }: PlanManagementViewProps) {
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // DND Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // We use a dummy wallet address to fetch all available plans/permissions
    // The useWalletAccess hook provides access to all system plans and permissions
    const {
        data: accessData,
        isLoading: isAccessLoading,
        refresh: refreshAccess
    } = useWalletAccess('');

    // DND State
    const [activeDragItem, setActiveDragItem] = useState<AccessItem | null>(null);

    // Filter State
    const [planBuilderSearchQuery, setPlanBuilderSearchQuery] = useState('');
    const [permissionSearchQuery, setPermissionSearchQuery] = useState('');

    // Plan Builder State
    const [builderSelectedPlanId, setBuilderSelectedPlanId] = useState<string | null>(null);
    const [builderPermissions, setBuilderPermissions] = useState<string[]>([]);
    const [builderForm, setBuilderForm] = useState({ name: '', description: '', priority: 0, expiryDays: 30 });
    const [isSavingBuilder, setIsSavingBuilder] = useState(false);
    const [hasBuilderChanges, setHasBuilderChanges] = useState(false);

    // Derived Plan Lists
    const allPlans = useMemo(() => [...accessData.authorizedPlans, ...accessData.availablePlans], [accessData.authorizedPlans, accessData.availablePlans]);

    // Available Permissions for Builder (All Permissions)
    const filteredAvailablePermissions = useMemo(() => {
        const assignedSet = new Set(builderPermissions);
        const allSystemPermissions = [...accessData.availablePermissions, ...accessData.authorizedPermissions];

        // Remove duplicates
        const uniqueSystemPermissions = Array.from(new Map(allSystemPermissions.map(item => [item.name, item])).values());

        return uniqueSystemPermissions.filter(p => !assignedSet.has(p.name) && p.name.toLowerCase().includes(permissionSearchQuery.toLowerCase()));
    }, [accessData.availablePermissions, accessData.authorizedPermissions, builderPermissions, permissionSearchQuery]);

    // Builder Logic
    const handleSelectPlanForBuilder = (planId: string) => {
        const plan = allPlans.find(g => g.id === planId);
        if (plan) {
            setBuilderSelectedPlanId(planId);
            setBuilderPermissions(plan.permissions || []);
            setBuilderForm({
                name: plan.name,
                description: plan.description || '',
                // @ts-ignore
                priority: (plan as any).priority_level || 0,
                // @ts-ignore
                expiryDays: (plan as any).default_expiry_days || 30
            });
            setHasBuilderChanges(false);
        }
    };

    const handleSavePlan = async () => {
        if (!builderSelectedPlanId) return;
        setIsSavingBuilder(true);
        try {
            await updatePlanAction(builderSelectedPlanId, {
                name: builderForm.name,
                description: builderForm.description,
                permissions: builderPermissions,
                priority_level: builderForm.priority,
                default_expiry_days: builderForm.expiryDays
            });
            toast.success('Plan updated successfully');
            setHasBuilderChanges(false);
            refreshAccess();
        } catch (err) {
            toast.error('Failed to save plan details');
            console.error(err);
        } finally {
            setIsSavingBuilder(false);
        }
    };

    // DND Handlers
    const handleDragStart = (event: DragStartEvent) => {
        const { active } = event;
        const allPermissions = [...accessData.availablePermissions, ...accessData.authorizedPermissions];

        const item = active.data.current?.type === 'plan'
            ? allPlans.find(p => p.id === active.id)
            : allPermissions.find(p => p.id === active.id);

        if (item) {
            setActiveDragItem(item);
        }
    };

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveDragItem(null);

        if (!over) { return; }

        // Dropped Permission into Builder List
        if (active.data.current?.type === 'permission' && over.id === 'builder-plan-permissions') {
            const permissionName = active.data.current.name;
            if (builderSelectedPlanId && !builderPermissions.includes(permissionName)) {
                setBuilderPermissions(prev => [...prev, permissionName]);
                setHasBuilderChanges(true);
            }
        }
    };

    if (authLoading || isAccessLoading) {
        return (
            <div className="space-y-4">
                <div className="flex gap-4">
                    <Skeleton className="h-10 w-64" />
                    <Skeleton className="h-10 w-32" />
                </div>
                <Skeleton className="h-[600px] w-full rounded-xl" />
            </div>
        );
    }

    return (
        <DndContext
            sensors={sensors}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
        >
            <div className={cn("space-y-6", className)}>
                {/* Plan & Plan Management Builder */}
                <div className="space-y-6">
                    <div className="flex items-center justify-between">
                        <div>
                            <h2 className="text-2xl font-black text-white flex items-center gap-3 tracking-tight">
                                <Package className="h-6 w-6 text-[#7645d9]" />
                                Plan Management
                            </h2>
                            <p className="text-sm font-medium text-muted-foreground mt-1">Configure user plans and permission sets</p>
                        </div>
                        <Button
                            size="lg"
                            className="bg-[#1fc7d4] hover:bg-[#1fc7d4]/90 text-white font-bold rounded-2xl shadow-lg shadow-cyan-500/20 px-8 transition-all active:scale-95"
                            onClick={() => {/* TODO: Create New Plan Handler */ }}
                        >
                            <Plus className="h-5 w-5 mr-2" />
                            New Plan
                        </Button>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
                        {/* LEFT COLUMN: Selection */}
                        <div className="lg:col-span-5">
                            <div className="flex flex-col gap-4">
                                {/* Top: Plan List */}
                                <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden">
                                    <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5">
                                        <div className="flex items-center justify-between mb-4">
                                            <CardTitle className="text-lg font-bold text-foreground">Select Plan</CardTitle>
                                            <Badge variant="outline" className="text-xs font-bold border-cyan-500/30 text-[#1fc7d4] rounded-lg">
                                                {allPlans.length} plans
                                            </Badge>
                                        </div>
                                        <div className="relative">
                                            <Search className="absolute left-4 top-3 h-4 w-4 text-muted-foreground/50" />
                                            <Input
                                                placeholder="Search plans..."
                                                value={planBuilderSearchQuery}
                                                onChange={e => setPlanBuilderSearchQuery(e.target.value)}
                                                className="h-10 pl-11 text-sm bg-white/5 border-white/5 text-foreground rounded-xl placeholder:text-muted-foreground/30"
                                            />
                                        </div>
                                    </CardHeader>
                                    <CardContent className="p-0 overflow-y-auto max-h-[300px]">
                                        <div className="divide-y divide-white/5">
                                            {allPlans
                                                .filter(g => g.name.toLowerCase().includes(planBuilderSearchQuery.toLowerCase()))
                                                .map(plan => (
                                                    <div
                                                        key={plan.id}
                                                        onClick={() => handleSelectPlanForBuilder(plan.id)}
                                                        className={cn(
                                                            "p-3 flex items-center justify-between cursor-pointer hover:bg-white/5 transition-colors",
                                                            builderSelectedPlanId === plan.id ? "bg-purple-500/10 hover:bg-purple-500/20 border-l-4 border-l-purple-500" : "border-l-4 border-l-transparent"
                                                        )}
                                                    >
                                                        <div className="min-w-0 flex-1">
                                                            <p className="font-medium text-sm text-slate-200">{plan.name}</p>
                                                            <p className="text-xs text-slate-500 truncate">{plan.description || 'No description'}</p>
                                                        </div>
                                                        <Button size="icon" variant="ghost" className="h-6 w-6 text-slate-500 hover:text-white flex-shrink-0">
                                                            <span className="sr-only">Edit</span>
                                                            <ArrowLeft className="h-3 w-3 rotate-180" />
                                                        </Button>
                                                    </div>
                                                ))}
                                        </div>
                                    </CardContent>
                                </Card>

                                {/* Bottom: Available Permissions */}
                                <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden">
                                    <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5">
                                        <div className="flex items-center justify-between mb-4">
                                            <CardTitle className="text-lg font-bold text-foreground">Available Permissions</CardTitle>
                                            <Badge variant="outline" className="text-xs font-bold border-purple-500/30 text-[#7645d9] rounded-lg">
                                                {filteredAvailablePermissions.length} available
                                            </Badge>
                                        </div>
                                        <div className="relative">
                                            <Search className="absolute left-4 top-3 h-4 w-4 text-muted-foreground/50" />
                                            <Input
                                                placeholder="Search permissions..."
                                                value={permissionSearchQuery}
                                                onChange={e => setPermissionSearchQuery(e.target.value)}
                                                className="h-10 pl-11 text-sm bg-white/5 border-white/5 text-foreground rounded-xl placeholder:text-muted-foreground/30"
                                            />
                                        </div>
                                    </CardHeader>
                                    <CardContent className="p-4 overflow-y-auto max-h-[450px]">
                                        <div className="grid grid-cols-1 gap-2">
                                            {filteredAvailablePermissions.map(perm => (
                                                <DraggablePermissionItem
                                                    key={perm.id}
                                                    id={perm.id}
                                                    label={perm.name}
                                                />
                                            ))}
                                            {filteredAvailablePermissions.length === 0 && (
                                                <div className="flex flex-col items-center justify-center py-8 text-slate-500">
                                                    <Key className="h-8 w-8 mb-2 opacity-20" />
                                                    <p className="text-sm">No permissions found</p>
                                                </div>
                                            )}
                                        </div>
                                    </CardContent>
                                </Card>
                            </div>
                        </div>

                        {/* RIGHT COLUMN: Editor */}
                        <div className="lg:col-span-7 flex flex-col gap-4">
                            {builderSelectedPlanId ? (
                                <>
                                    {/* Edit Plan Details */}
                                    <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden shrink-0">
                                        <div className="h-1.5 bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]" />
                                        <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5">
                                            <CardTitle className="text-sm font-black uppercase tracking-widest text-[#ffb237]">
                                                Edit Plan Details
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="p-8 grid grid-cols-2 gap-6">
                                            <div className="col-span-2 md:col-span-1 space-y-3">
                                                <Label className="text-[10px] font-black uppercase tracking-widest text-muted-foreground ml-1">Plan Name</Label>
                                                <Input
                                                    placeholder="e.g. Premium Plan"
                                                    value={builderForm.name}
                                                    onChange={e => { setBuilderForm(p => ({ ...p, name: e.target.value })); setHasBuilderChanges(true); }}
                                                    className="h-12 bg-white/5 border-white/5 text-foreground rounded-2xl placeholder:text-muted-foreground/30 font-bold"
                                                />
                                            </div>
                                            <div className="col-span-2 md:col-span-1 space-y-2">
                                                <Label className="text-slate-400">Priority Order</Label>
                                                <Input
                                                    type="number"
                                                    placeholder="0"
                                                    value={builderForm.priority}
                                                    onChange={e => { setBuilderForm(p => ({ ...p, priority: parseInt(e.target.value) || 0 })); setHasBuilderChanges(true); }}
                                                    className="bg-slate-950/50 border-white/10 text-slate-200 placeholder:text-slate-600"
                                                />
                                            </div>
                                            <div className="col-span-2 md:col-span-1 space-y-2">
                                                <Label className="text-slate-400">Default Expiry (Days)</Label>
                                                <Input
                                                    type="number"
                                                    placeholder="30"
                                                    value={builderForm.expiryDays}
                                                    onChange={e => { setBuilderForm(p => ({ ...p, expiryDays: parseInt(e.target.value) || 0 })); setHasBuilderChanges(true); }}
                                                    className="bg-slate-950/50 border-white/10 text-slate-200 placeholder:text-slate-600"
                                                />
                                            </div>
                                            <div className="col-span-2 md:col-span-1 space-y-2">
                                                <Label className="text-slate-400">Description</Label>
                                                <Input
                                                    placeholder="Description of this plan..."
                                                    value={builderForm.description}
                                                    onChange={e => { setBuilderForm(p => ({ ...p, description: e.target.value })); setHasBuilderChanges(true); }}
                                                    className="bg-slate-950/50 border-white/10 text-slate-200 placeholder:text-slate-600"
                                                />
                                            </div>
                                        </CardContent>
                                    </Card>

                                    {/* Assigned Permissions */}
                                    <Card className="flex-1 flex flex-col border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden min-h-0">
                                        <div className="h-1.5 bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
                                        <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5">
                                            <CardTitle className="text-sm font-black uppercase tracking-widest text-[#1fc7d4] flex justify-between items-center">
                                                <span>Assigned Permissions ({builderPermissions.length})</span>
                                                <Badge variant="outline" className="text-[10px] font-bold border-cyan-500/30 text-[#1fc7d4] rounded-lg">
                                                    Drag here
                                                </Badge>
                                            </CardTitle>
                                        </CardHeader>
                                        <CardContent className="p-0 flex flex-col min-h-0 overflow-hidden">
                                            <div className="p-8 overflow-y-auto max-h-[400px]">
                                                <DroppablePermissionList
                                                    id="builder-plan-permissions"
                                                    items={builderPermissions}
                                                    emptyMessage="Drag permissions here from the left"
                                                />
                                            </div>
                                            <div className="p-6 border-t border-white/5 bg-black/20 flex gap-3 justify-end mt-auto">
                                                <Button
                                                    variant="ghost"
                                                    size="sm"
                                                    onClick={() => setBuilderSelectedPlanId(null)}
                                                    className="text-muted-foreground hover:text-white font-bold px-6"
                                                >
                                                    Discard
                                                </Button>
                                                <Button
                                                    variant="destructive"
                                                    size="sm"
                                                    onClick={() => {/* TODO: Delete Plan */ }}
                                                    className="bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white font-bold px-6 border-none rounded-xl"
                                                >
                                                    Delete
                                                </Button>
                                                <Button
                                                    size="sm"
                                                    className="bg-[#7645d9] hover:bg-[#7645d9]/90 text-white font-bold px-8 rounded-xl shadow-lg shadow-purple-500/20 active:scale-95 transition-all"
                                                    disabled={!hasBuilderChanges || isSavingBuilder}
                                                    onClick={handleSavePlan}
                                                >
                                                    {isSavingBuilder ? <Loader2 className="h-4 w-4 animate-spin mr-1" /> : <Save className="h-4 w-4 mr-1" />}
                                                    Save Changes
                                                </Button>
                                            </div>
                                        </CardContent>
                                    </Card>
                                </>
                            ) : (
                                <div className="h-full flex flex-col items-center justify-center border-2 border-dashed border-white/10 rounded-xl bg-slate-900/30 text-slate-500 py-16">
                                    <Package className="h-16 w-16 mb-4 opacity-20" />
                                    <h3 className="text-lg font-semibold text-slate-400">No Plan Selected</h3>
                                    <p className="text-sm">Select a plan from the left to edit details and permissions</p>
                                </div>
                            )}
                        </div>
                    </div>
                </div>

                {/* DND Overlay */}
                <DragOverlay>
                    {activeDragItem && (
                        <div className={cn(
                            "flex items-center gap-2 px-3 py-2 bg-white dark:bg-gray-800 rounded-lg shadow-xl border opacity-90 scale-105 pointer-events-none",
                            activeDragItem.type === 'permission' ? "border-blue-500" : "border-purple-500"
                        )}>
                            {activeDragItem.type === 'permission' ? (
                                <Key className="h-4 w-4 text-purple-500" />
                            ) : (
                                <Package className="h-4 w-4 text-purple-500" />
                            )}
                            <span className="font-medium text-sm">{activeDragItem.name}</span>
                        </div>
                    )}
                </DragOverlay>
            </div>
        </DndContext>
    );
}
