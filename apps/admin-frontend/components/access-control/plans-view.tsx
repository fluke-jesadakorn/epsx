'use client';

import type { DragEndEvent, DragStartEvent, Modifier } from '@dnd-kit/core';
import {
    closestCenter,
    DndContext,
    DragOverlay,
    PointerSensor,
    useSensor,
    useSensors
} from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    useSortable,
    verticalListSortingStrategy
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
    GripVertical,
    Loader2,
    Package,
    Plus,
    RotateCcw,
    Search,
    Trash2
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import { createPlanAction, deletePlanAction, getPlansAction, updatePlanAction } from '@/app/wallet-management/plan-actions';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
    SheetTrigger,
} from '@/components/ui/sheet';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/shared/components/ui/tooltip';
import { logger } from '@/shared/utils/logger';
import { DualPanePermissionSelector } from './dual-pane-permission-selector';

const FREE_PLAN_ID = '00000000-0000-0000-0000-000000000000';

interface PlansViewProps {
    className?: string;
}

export function PlansView({ className }: PlansViewProps) {
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // --- DATA STATE ---
    const [permissions, setPermissions] = useState<PermissionDefinition[]>([]);
    const [plans, setPlans] = useState<PermissionPlan[]>([]);
    const [isLoadingData, setIsLoadingData] = useState(false);

    // --- DRAG AND DROP STATE ---
    const [activeId, setActiveId] = useState<string | null>(null);

    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: {
                distance: 1,
            },
        })
    );

    // Custom modifier to snap overlay closer to cursor
    const snapToCursor = useCallback<Modifier>(({ transform, activatorEvent, draggingNodeRect }) => {
        if (activatorEvent && draggingNodeRect) {
            // Calculate how far down the item we clicked (offset)
            // draggingNodeRect is the initial ClientRect of the item
            // activatorEvent is the initial pointer event
            const event = activatorEvent as unknown as PointerEvent;
            const offsetY = (event.clientY || 0) - draggingNodeRect.top;

            return {
                ...transform,
                // Add the offset to bring the top of the card down to the cursor
                // Subtract 10px to give it a slight "pop" above the cursor or just keep it tight
                y: transform.y + offsetY - 10,
            };
        }

        return transform;
    }, []);

    // --- PLAN MODE STATE ---
    const [planSearch, setPlanSearch] = useState('');
    const [selectedPlan, setSelectedPlan] = useState<PermissionPlan | null>(null);
    const [planEditForm, setPlanEditForm] = useState<{
        name: string;
        description: string;
        priority: number;
        price: number;
        expiryDays: number;
        permissions: string[];
        is_public: boolean;
        is_active: boolean;
        features: string[];
    }>({ name: '', description: '', priority: 0, price: 0, expiryDays: 30, permissions: [], is_public: true, is_active: true, features: [] });

    const [isSavingPlan, setIsSavingPlan] = useState(false);
    const [hasPlanChanges, setHasPlanChanges] = useState(false);
    const [isCreatePlanOpen, setIsCreatePlanOpen] = useState(false);

    const [planDeleteConfirm, setPlanDeleteConfirm] = useState<PermissionPlan | null>(null);
    const [deleteConfirmationInput, setDeleteConfirmationInput] = useState('');

    useEffect(() => {
        if (!planDeleteConfirm) { setDeleteConfirmationInput(''); }
    }, [planDeleteConfirm]);

    // --- DATA FETCHING ---
    const loadAllData = useCallback(async () => {
        setIsLoadingData(true);
        try {
            const [permRes, planRes] = await Promise.all([
                getPermissionsAction(),
                getPlansAction()
            ]);

            if (permRes.success && permRes.data) {
                setPermissions(permRes.data);
            }
            if (planRes) {
                setPlans(planRes);
            }
        } catch (error: unknown) {
            logger.error('Failed to load data:', error instanceof Error ? error.message : String(error));
            toast.error('Failed to load access data');
        } finally {
            setIsLoadingData(false);
        }
    }, []);

    useEffect(() => {
        if (isAuthenticated) {
            void loadAllData();
        }
    }, [isAuthenticated, loadAllData]);

    // --- DERIVED DATA ---
    const filteredPlans = useMemo(() =>
        plans.filter(p => (p.name ?? '').toLowerCase().includes(planSearch.toLowerCase())),
        [plans, planSearch]);

    // --- EVENT HANDLERS ---
    const handleSelectPlan = (plan: PermissionPlan) => {
        setSelectedPlan(plan);
        // Extract features from metadata
        const features = Array.isArray(plan.plan_metadata?.features)
            ? plan.plan_metadata?.features as string[]
            : [];

        setPlanEditForm({
            name: plan.name ?? '',
            description: plan.description ?? '',
            priority: plan.priority_level ?? 0,
            price: plan.price ?? 0,
            expiryDays: plan.default_expiry_days ?? 30,
            permissions: plan.permissions ?? [],
            is_public: plan.is_public !== false,
            is_active: plan.is_active !== false,
            features
        });
        setHasPlanChanges(false);
    };

    // --- DRAG HANDLERS ---
    const handleDragStart = (event: DragStartEvent) => {
        setActiveId(event.active.id as string);
    };

    const handleReorderPlans = (updates: { id: string, display_order: number }[]) => {
        toast.promise(
            Promise.all(updates.map(u =>
                updatePlanAction(u.id, {
                    // We map back to priority_level for the API structure if needed, 
                    // but check backend: usually display_order is what we want.
                    // The backend model uses display_order but the PlanEditForm uses priority maps to priority_level.
                    // Let's assume priority_level maps to display_order or vice versa.
                    // Based on plans.rs: display_order: req.display_order 
                    display_order: u.display_order
                })
            )),
            {
                loading: 'Reordering plans...',
                success: 'Plans reordered',
                error: 'Failed to reorder plans'
            }
        );
    }

    const handleDragEnd = async (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveId(null);

        if (active.id !== over?.id) {
            // Find indices in the CURRENT plans state
            const oldIndex = plans.findIndex((item) => item.id === active.id);
            const newIndex = plans.findIndex((item) => item.id === over?.id);

            if (oldIndex === -1 || newIndex === -1) { return; }

            // Calculate new items array
            const newItems = arrayMove(plans, oldIndex, newIndex);

            // 1. Update UI state optimistically
            setPlans(newItems);

            // 2. Prepare updates for server
            const updates = newItems.map((plan, index) => ({
                id: plan.id,
                display_order: index
            }));

            // 3. Update active selection if needed
            if (selectedPlan) {
                const updatedSelected = updates.find(u => u.id === selectedPlan.id);
                if (updatedSelected) {
                    setPlanEditForm(prev => ({ ...prev, priority: updatedSelected.display_order }));
                }
            }

            // 4. Trigger Server Action (Side Effect)
            void handleReorderPlans(updates);
        }
    };

    const isFreePlan = (id?: string) => id === FREE_PLAN_ID;

    const handleSavePlan = async () => {
        if (!selectedPlan) { return; }
        setIsSavingPlan(true);
        try {
            const updated = await updatePlanAction(selectedPlan.id, {
                name: planEditForm.name,
                description: planEditForm.description,
                priority_level: planEditForm.priority,
                price: isFreePlan(selectedPlan.id) ? undefined : planEditForm.price,
                default_expiry_days: planEditForm.expiryDays,
                permissions: planEditForm.permissions,
                is_public: planEditForm.is_public,
                is_active: planEditForm.is_active,
                // Pass features in plan_metadata
                plan_metadata: {
                    ...selectedPlan.plan_metadata,
                    features: planEditForm.features
                }
            });
            toast.success('Plan updated');
            setPlans(prev => prev.map(p => p.id === updated.id ? updated : p));
            setSelectedPlan(updated);
            setHasPlanChanges(false);
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to update plan');
        }
        finally { setIsSavingPlan(false); }
    };

    const handleDeletePlan = async () => {
        if (!planDeleteConfirm) { return; }
        try {
            await deletePlanAction(planDeleteConfirm.id);
            toast.success('Plan deleted');
            setPlans(prev => prev.filter(p => p.id !== planDeleteConfirm.id));
            if (selectedPlan?.id === planDeleteConfirm.id) { setSelectedPlan(null); }
            setPlanDeleteConfirm(null);
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to delete plan');
        }
    };

    const handleQuickToggle = async (e: React.MouseEvent, plan: PermissionPlan) => {
        e.stopPropagation();
        if (isFreePlan(plan.id)) {
            toast.error('Constant Free Plan status cannot be changed');
            return;
        }
        const newState = !plan.is_active;
        try {
            setPlans(prev => prev.map(p => p.id === plan.id ? { ...p, is_active: newState } : p));
            if (selectedPlan?.id === plan.id) {
                setSelectedPlan(prev => prev ? { ...prev, is_active: newState } : null);
                setPlanEditForm(prev => ({ ...prev, is_active: newState }));
            }

            await updatePlanAction(plan.id, { is_active: newState });
            toast.success(`Plan ${newState ? 'activated' : 'deactivated'}`);
        } catch (_error) {
            setPlans(prev => prev.map(p => p.id === plan.id ? { ...p, is_active: !newState } : p));
            if (selectedPlan?.id === plan.id) {
                setSelectedPlan(prev => prev ? { ...prev, is_active: !newState } : null);
                setPlanEditForm(prev => ({ ...prev, is_active: !newState }));
            }
            toast.error('Failed to update status');
        }
    };

    const handleDiscardChanges = () => {
        if (selectedPlan) {
            handleSelectPlan(selectedPlan);
            toast.info('Changes discarded');
        }
    };

    if (authLoading || (isLoadingData && !plans.length)) {
        return <div className="p-8 flex justify-center"><Loader2 className="animate-spin" /></div>;
    }

    return (
        <div className={cn("grid grid-cols-1 lg:grid-cols-12 gap-6 h-[calc(100vh-250px)] min-h-[500px]", className)}>
            {/* SIDEBAR: PLAN LIST */}
            <div className="lg:col-span-4 flex flex-col gap-4 h-full">
                <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[24px] overflow-hidden flex flex-col h-full">
                    <CardHeader className="p-4 bg-white/5 border-b border-white/5 space-y-4 shrink-0">
                        <div className="flex items-center justify-between">
                            <h2 className="text-sm font-bold flex items-center gap-2 uppercase tracking-wider text-muted-foreground">
                                Active Plans
                            </h2>
                            <Badge variant="secondary" className="bg-cyan-500/10 text-[#1fc7d4]">{plans.length}</Badge>
                        </div>
                        <div className="flex gap-2">
                            <div className="relative flex-1">
                                <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground/50" />
                                <Input
                                    placeholder="Search plans..."
                                    value={planSearch}
                                    onChange={e => setPlanSearch(e.target.value)}
                                    className="h-9 pl-9 text-sm bg-white/5 border-white/5"
                                />
                            </div>
                            <CreatePlanSheet
                                open={isCreatePlanOpen}
                                onOpenChange={setIsCreatePlanOpen}
                                onSuccess={loadAllData}
                            />
                        </div>
                    </CardHeader>
                    <CardContent className="p-0 overflow-y-auto flex-1 scrollbar-thin scrollbar-thumb-white/10">
                        {planSearch ? (
                            <div className="divide-y divide-white/5">
                                {filteredPlans.map((plan, index) => (
                                    <SortablePlanItem
                                        key={plan.id}
                                        plan={plan}
                                        index={index}
                                        selectedPlanId={selectedPlan?.id}
                                        onSelect={handleSelectPlan}
                                        isFreePlan={isFreePlan(plan.id)}
                                        onQuickToggle={handleQuickToggle}
                                        disabled={true} // Disable drag when searching
                                    />
                                ))}
                            </div>
                        ) : (
                            <DndContext
                                sensors={sensors}
                                collisionDetection={closestCenter}
                                onDragStart={handleDragStart}
                                onDragEnd={handleDragEnd}
                                modifiers={[snapToCursor]}
                            >
                                <SortableContext
                                    items={filteredPlans.map(p => p.id)}
                                    strategy={verticalListSortingStrategy}
                                >
                                    <div className="divide-y divide-white/5">
                                        {filteredPlans.map((plan, index) => (
                                            <SortablePlanItem
                                                key={plan.id}
                                                plan={plan}
                                                index={index}
                                                selectedPlanId={selectedPlan?.id}
                                                onSelect={handleSelectPlan}
                                                isFreePlan={isFreePlan(plan.id)}
                                                onQuickToggle={handleQuickToggle}
                                            />
                                        ))}
                                    </div>
                                </SortableContext>
                                <DragOverlay>
                                    {activeId ? (
                                        <div className="bg-slate-800/90 backdrop-blur border border-cyan-500/30 shadow-2xl rounded-lg overflow-hidden cursor-grabbing">
                                            {(() => {
                                                const planIndex = plans.findIndex(p => p.id === activeId);
                                                const plan = plans[planIndex];
                                                if (!plan) { return null; }
                                                return (
                                                    <PlanItem
                                                        plan={plan}
                                                        index={planIndex}
                                                        isFreePlan={isFreePlan(plan.id)}
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
            </div>

            {/* EDITOR */}
            <div className="lg:col-span-8 h-full">
                {selectedPlan ? (
                    <Card className="h-full border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden flex flex-col">
                        <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5 flex flex-row items-center justify-between shrink-0">
                            <div className="flex items-center gap-3">
                                <div className="h-10 w-10 rounded-xl bg-[#1fc7d4]/20 flex items-center justify-center">
                                    <Package className="w-5 h-5 text-[#1fc7d4]" />
                                </div>
                                <div>
                                    <CardTitle className="text-lg font-bold">Edit Plan</CardTitle>
                                    <p className="text-xs text-muted-foreground font-mono">{selectedPlan.id}</p>
                                </div>
                            </div>
                            <div className="flex gap-2">
                                <Button
                                    variant="ghost"
                                    className="text-red-400 hover:text-red-300 hover:bg-red-500/10"
                                    size="sm"
                                    onClick={() => setPlanDeleteConfirm(selectedPlan)}
                                    disabled={isFreePlan(selectedPlan.id)}
                                >
                                    <Trash2 className="w-4 h-4 mr-2" />
                                    Delete
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={handleDiscardChanges}
                                    disabled={!hasPlanChanges || isSavingPlan}
                                    className="text-muted-foreground hover:text-foreground"
                                >
                                    <RotateCcw className="w-4 h-4 mr-2" />
                                    Discard
                                </Button>
                                <Button
                                    size="sm"
                                    onClick={handleSavePlan}
                                    disabled={!hasPlanChanges || isSavingPlan}
                                    className="bg-[#1fc7d4] text-white hover:bg-[#1fc7d4]/90"
                                >
                                    {isSavingPlan && <Loader2 className="w-3 h-3 animate-spin mr-2" />}
                                    Save Changes
                                </Button>
                            </div>
                        </CardHeader>
                        <CardContent className="p-8 space-y-8 overflow-y-auto flex-1">
                            <div className="grid grid-cols-2 gap-6">
                                <div className="space-y-2">
                                    <Label>Plan Name</Label>
                                    <Input
                                        value={planEditForm.name}
                                        onChange={e => { setPlanEditForm(p => ({ ...p, name: e.target.value })); setHasPlanChanges(true); }}
                                        className="bg-white/5 border-white/10"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label>Priority</Label>
                                    <Input
                                        type="text"
                                        inputMode="numeric"
                                        value={planEditForm.priority}
                                        onChange={e => {
                                            const val = e.target.value;
                                            if (val === '-' || val === '') {
                                                setPlanEditForm(p => ({ ...p, priority: val as unknown as number }));
                                            } else {
                                                const parsed = parseInt(val);
                                                if (!isNaN(parsed)) {
                                                    setPlanEditForm(p => ({ ...p, priority: parsed }));
                                                }
                                            }
                                            setHasPlanChanges(true);
                                        }}
                                        className="bg-white/5 border-white/10"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label className="flex items-center gap-2">
                                        Price (USD)
                                        {isFreePlan(selectedPlan.id) && <Tooltip>
                                            <TooltipTrigger asChild>
                                                <div className="h-3.5 w-3.5 rounded-full bg-slate-500/20 flex items-center justify-center cursor-help">
                                                    <span className="text-[10px] font-bold">?</span>
                                                </div>
                                            </TooltipTrigger>
                                            <TooltipContent side="right" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                                <p className="text-xs">Pricing for the Free Plan is permanent and cannot be modified.</p>
                                            </TooltipContent>
                                        </Tooltip>
                                        }
                                    </Label>
                                    <Input
                                        type="text"
                                        inputMode="decimal"
                                        value={planEditForm.price}
                                        onChange={e => {
                                            const val = e.target.value;
                                            if (val === '-' || val === '' || val === '.') {
                                                setPlanEditForm(p => ({ ...p, price: val as unknown as number }));
                                            } else {
                                                const parsed = parseFloat(val);
                                                if (!isNaN(parsed)) {
                                                    setPlanEditForm(p => ({ ...p, price: parsed }));
                                                }
                                            }
                                            setHasPlanChanges(true);
                                        }}
                                        className="bg-white/5 border-white/10"
                                        disabled={isFreePlan(selectedPlan.id)}
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label className="flex items-center gap-2">
                                        Expiry (Days)
                                        <Tooltip>
                                            <TooltipTrigger asChild>
                                                <div className="h-3.5 w-3.5 rounded-full bg-[#1fc7d4]/20 flex items-center justify-center cursor-help">
                                                    <span className="text-[10px] font-bold text-[#1fc7d4]">?</span>
                                                </div>
                                            </TooltipTrigger>
                                            <TooltipContent side="right" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                                <p className="text-xs">Set to -1 for permanent expiry (never expires).</p>
                                            </TooltipContent>
                                        </Tooltip>
                                    </Label>
                                    <Input
                                        type="text"
                                        inputMode="numeric"
                                        value={planEditForm.expiryDays}
                                        onChange={e => {
                                            const val = e.target.value;
                                            if (val === '-' || val === '') {
                                                setPlanEditForm(p => ({ ...p, expiryDays: val as unknown as number }));
                                            } else {
                                                const parsed = parseInt(val);
                                                if (!isNaN(parsed)) {
                                                    setPlanEditForm(p => ({ ...p, expiryDays: parsed }));
                                                }
                                            }
                                            setHasPlanChanges(true);
                                        }}
                                        className="bg-white/5 border-white/10"
                                        placeholder="-1 for permanent"
                                    />
                                </div>
                                <div className="col-span-2 space-y-2">
                                    <Label>Description</Label>
                                    <Textarea
                                        value={planEditForm.description}
                                        onChange={e => { setPlanEditForm(p => ({ ...p, description: e.target.value })); setHasPlanChanges(true); }}
                                        className="bg-white/5 border-white/10"
                                    />
                                </div>
                                <div className="col-span-2 space-y-2">
                                    <Label>Features (One per line)</Label>
                                    <p className="text-xs text-muted-foreground">These features will be displayed on the public pricing page.</p>
                                    <Textarea
                                        value={planEditForm.features.join('\n')}
                                        onChange={e => {
                                            const features_list = e.target.value.split('\n');
                                            setPlanEditForm(p => ({ ...p, features: features_list }));
                                            setHasPlanChanges(true);
                                        }}
                                        className="bg-white/5 border-white/10 min-h-[120px] font-mono text-sm"
                                        placeholder="Advanced analytics&#10;Unlimited stock analysis&#10;Priority support"
                                    />
                                </div>
                                <div className="col-span-2 flex items-center justify-between p-4 rounded-lg bg-white/5 border border-white/10">
                                    <div className="flex flex-col">
                                        <span className="text-sm font-medium text-white/80">Public Visibility</span>
                                        <span className="text-xs text-white/40">Show on pricing page</span>
                                    </div>
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <div className={isFreePlan(selectedPlan.id) ? "cursor-not-allowed opacity-80" : ""}>
                                                <Switch
                                                    checked={planEditForm.is_public}
                                                    disabled={isFreePlan(selectedPlan.id)}
                                                    onCheckedChange={(checked) => {
                                                        setPlanEditForm(p => ({ ...p, is_public: checked }));
                                                        setHasPlanChanges(true);
                                                    }}
                                                />
                                            </div>
                                        </TooltipTrigger>
                                        {isFreePlan(selectedPlan.id) && (
                                            <TooltipContent side="left" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                                <p className="text-xs">Default system plan visibility cannot be changed</p>
                                            </TooltipContent>
                                        )}
                                    </Tooltip>
                                </div>
                                <div className="col-span-2 flex items-center justify-between p-4 rounded-lg bg-white/5 border border-white/10">
                                    <div className="flex flex-col">
                                        <span className="text-sm font-medium text-white/80">Active Status</span>
                                        <span className="text-xs text-white/40">Plan assignments allowed</span>
                                    </div>
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <div className={isFreePlan(selectedPlan.id) ? "cursor-not-allowed opacity-80" : ""}>
                                                <Switch
                                                    checked={planEditForm.is_active}
                                                    disabled={isFreePlan(selectedPlan.id)}
                                                    onCheckedChange={(checked) => {
                                                        setPlanEditForm(p => ({ ...p, is_active: checked }));
                                                        setHasPlanChanges(true);
                                                    }}
                                                />
                                            </div>
                                        </TooltipTrigger>
                                        {isFreePlan(selectedPlan.id) && (
                                            <TooltipContent side="left" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                                <p className="text-xs">Default system plan status cannot be changed</p>
                                            </TooltipContent>
                                        )}
                                    </Tooltip>
                                </div>
                            </div>

                            <div className="space-y-4">
                                <Label className="text-[#1fc7d4] uppercase tracking-wider font-bold text-xs">Permission Assignment</Label>
                                <div className="h-[500px]">
                                    <DualPanePermissionSelector
                                        availablePermissions={permissions}
                                        assignedPermissionStrings={planEditForm.permissions}
                                        onChange={(newPermissions) => {
                                            setPlanEditForm(prev => ({
                                                ...prev,
                                                permissions: newPermissions
                                            }));
                                            setHasPlanChanges(true);
                                        }}
                                    />
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                ) : (
                    <EmptyState icon={Package} title="Select a Plan" description="Select to edit plan details." />
                )}
            </div>

            {/* Delete Confirmation */}
            <Dialog open={Boolean(planDeleteConfirm)} onOpenChange={(o) => !o && setPlanDeleteConfirm(null)}>
                <DialogContent className="max-w-[400px]">
                    <DialogHeader>
                        <div className="h-12 w-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4 mx-auto sm:mx-0">
                            <Trash2 className="h-6 w-6 text-red-500" />
                        </div>
                        <DialogTitle className="text-xl">Delete Plan?</DialogTitle>
                        <DialogDescription className="text-slate-400">
                            Are you sure you want to delete <span className="text-white font-bold">{planDeleteConfirm?.name}</span>?
                            This action cannot be undone and will affect all users currently assigned to this plan.
                        </DialogDescription>
                        <div className="mt-4">
                            <Label className="text-xs text-slate-500 mb-2 block tracking-wider font-bold">Type <span className="text-white select-all">{planDeleteConfirm?.name}</span> to confirm</Label>
                            <Input
                                value={deleteConfirmationInput}
                                onChange={(e) => setDeleteConfirmationInput(e.target.value)}
                                placeholder="Type plan name"
                                className="bg-white/5 border-white/10 text-white placeholder:text-white/20"
                            />
                        </div>
                    </DialogHeader>
                    <DialogFooter className="mt-4 gap-2 sm:gap-0">
                        <Button
                            variant="ghost"
                            onClick={() => setPlanDeleteConfirm(null)}
                            className="text-slate-400 hover:text-white hover:bg-white/5"
                        >
                            Cancel
                        </Button>
                        <Button
                            variant="destructive"
                            onClick={handleDeletePlan}
                            disabled={deleteConfirmationInput !== planDeleteConfirm?.name}
                            className="bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            Yes, Delete Plan
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div >
    );
}

function EmptyState({ icon: Icon, title, description }: { icon: React.ElementType, title: string, description: string }) {
    return (
        <div className="h-full flex flex-col items-center justify-center border border-dashed border-white/10 rounded-[32px] bg-slate-900/20 text-slate-500 p-8 text-center">
            <div className="h-20 w-20 rounded-full bg-white/5 flex items-center justify-center mb-6">
                <Icon className="h-10 w-10 opacity-30" />
            </div>
            <h3 className="text-xl font-bold text-slate-300 mb-2">{title}</h3>
            <p className="text-sm max-w-xs mx-auto">{description}</p>
        </div>
    );
}

function CreatePlanSheet({ open, onOpenChange, onSuccess }: { open: boolean, onOpenChange: (o: boolean) => void, onSuccess: () => void }) {
    const [formData, setFormData] = useState({ name: '', description: '', priority: 0, price: 0, default_expiry_days: 30 });
    const [submitting, setSubmitting] = useState(false);
    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault(); setSubmitting(true);
        try {
            await createPlanAction({
                name: formData.name,
                description: formData.description,
                permissions: [],
                priority_level: formData.priority,
                price: formData.price,
                default_expiry_days: formData.default_expiry_days
            });
            toast.success('Created');
            setFormData({ name: '', description: '', priority: 0, price: 0, default_expiry_days: 30 });
            onSuccess();
            onOpenChange(false);
        } catch (err: unknown) {
            toast.error(err instanceof Error ? err.message : 'Failed to create plan');
        } finally { setSubmitting(false); }
    };
    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetTrigger asChild>
                <Button size="icon" className="h-9 w-9 bg-[#1fc7d4] hover:bg-[#1fc7d4]/90">
                    <Plus className="h-4 w-4" />
                </Button>
            </SheetTrigger>
            <SheetContent side="right" className="w-[400px] sm:w-[540px] bg-slate-900 border-white/5 text-white flex flex-col h-full">
                <SheetHeader>
                    <SheetTitle>Create Plan</SheetTitle>
                    <SheetDescription>Create a new access plan.</SheetDescription>
                </SheetHeader>
                <form onSubmit={handleSubmit} className="space-y-6 pt-6 flex-1 flex flex-col overflow-y-auto">
                    <div className="space-y-2"><Label>Plan Name *</Label><Input required value={formData.name} onChange={e => setFormData({ ...formData, name: e.target.value })} className="bg-white/5" /></div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2"><Label>Priority</Label><Input type="number" value={formData.priority} onChange={e => setFormData({ ...formData, priority: parseInt(e.target.value) })} className="bg-white/5" /></div>
                        <div className="space-y-2"><Label>Price</Label><Input type="number" step="0.01" value={formData.price} onChange={e => setFormData({ ...formData, price: parseFloat(e.target.value) })} className="bg-white/5" /></div>
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Expiry (Days)
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <div className="h-3.5 w-3.5 rounded-full bg-[#1fc7d4]/20 flex items-center justify-center cursor-help">
                                        <span className="text-[10px] font-bold text-[#1fc7d4]">?</span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent side="right" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                    <p className="text-xs">Set to -1 for permanent expiry (never expires).</p>
                                </TooltipContent>
                            </Tooltip>
                        </Label>
                        <Input
                            type="text"
                            inputMode="numeric"
                            value={formData.default_expiry_days}
                            onChange={e => {
                                const val = e.target.value;
                                if (val === '-' || val === '') {
                                    setFormData({ ...formData, default_expiry_days: val as unknown as number });
                                } else {
                                    const parsed = parseInt(val);
                                    if (!isNaN(parsed)) {
                                        setFormData({ ...formData, default_expiry_days: parsed });
                                    }
                                }
                            }}
                            className="bg-white/5"
                            placeholder="-1 for permanent"
                        />
                    </div>
                    <div className="space-y-2"><Label>Description</Label><Textarea value={formData.description} onChange={e => setFormData({ ...formData, description: e.target.value })} className="bg-white/5 min-h-[100px]" /></div>
                    <SheetFooter className="mt-auto pt-6">
                        <Button type="submit" disabled={submitting} className="bg-[#1fc7d4] text-white w-full">
                            {submitting ? <Loader2 className="animate-spin w-4 h-4" /> : 'Create Plan'}
                        </Button>
                    </SheetFooter>
                </form>
            </SheetContent>
        </Sheet>
    );
}

// --- SHARED PLAN ITEM COMPONENT ---
// Used for both the list item (SortablePlanItem) and the drag overlay (PlanItem)
// Ensures pixel-perfect consistency during drag.

interface PlanItemProps {
    plan: PermissionPlan;
    index: number;
    selectedPlanId?: string;
    onSelect?: (plan: PermissionPlan) => void;
    isFreePlan: boolean;
    onQuickToggle?: (e: React.MouseEvent, plan: PermissionPlan) => void;
    isDragging?: boolean;
    disabled?: boolean;
    // Props passed from useSortable listeners/attributes
    dragHandleProps?: Record<string, unknown>;
    // Style override
    style?: React.CSSProperties;
    innerRef?: (node: HTMLElement | null) => void;
}

function PlanItem({
    plan,
    index,
    selectedPlanId,
    onSelect,
    isFreePlan,
    onQuickToggle,
    isDragging,
    disabled,
    dragHandleProps, // { ...listeners, ...attributes }
    style,
    innerRef
}: PlanItemProps) {
    return (
        <div
            ref={innerRef}
            style={style}
            onClick={() => onSelect?.(plan)}
            className={cn(
                "p-4 cursor-pointer hover:bg-white/5 transition-colors border-l-4 group relative bg-transparent", // Explicit bg-transparent for base
                selectedPlanId === plan.id ? "bg-cyan-500/10 border-l-[#1fc7d4]" : "border-l-transparent",
                isDragging ? "opacity-40" : "" // Visual cue for the original item when dragging
            )}
        >
            {/* Drag Handle - Absolutely positioned */}
            <div
                {...(dragHandleProps ?? {})}
                className={cn(
                    "absolute left-1 top-1/2 -translate-y-1/2 p-1 text-slate-500 hover:text-white cursor-grab active:cursor-grabbing opacity-0 group-hover:opacity-100 transition-opacity",
                    disabled ? "hidden" : "",
                )}
                onClick={(e) => e.stopPropagation()}
            >
                <GripVertical className="h-4 w-4" />
            </div>

            {/* Content with padding for handle */}
            <div className={cn("pl-4 transition-all duration-200")}>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        {/* ORDER INDICATOR */}
                        <div className="h-5 w-5 rounded bg-white/5 flex items-center justify-center text-[10px] font-mono text-muted-foreground">
                            {index + 1}
                        </div>
                        <div>
                            <h4 className="font-bold text-sm text-foreground">{plan.name}</h4>
                            <p className="text-xs text-muted-foreground line-clamp-1">{plan.description}</p>
                        </div>
                    </div>
                    <Badge variant="secondary" className="text-[10px] h-5 bg-white/5">{plan.permissions?.length ?? 0}</Badge>
                </div>
                <div className="flex items-center justify-between mt-3 pt-3 border-t border-white/5">
                    <span className={cn("text-[10px] font-medium uppercase tracking-wider", plan.is_active !== false ? "text-emerald-500" : "text-slate-500")}>
                        {plan.is_active !== false ? "Active" : "Inactive"}
                    </span>
                    <Switch
                        checked={plan.is_active !== false}
                        onCheckedChange={() => { }}
                        onClick={(e) => onQuickToggle?.(e, plan)}
                        disabled={isFreePlan}
                        className="scale-75 origin-right"
                    />
                </div>
            </div>
        </div>
    );
}

interface SortablePlanItemProps {
    plan: PermissionPlan;
    index: number;
    selectedPlanId?: string;
    onSelect: (plan: PermissionPlan) => void;
    isFreePlan: boolean;
    onQuickToggle: (e: React.MouseEvent, plan: PermissionPlan) => void;
    disabled?: boolean;
}

function SortablePlanItem({ plan, index, selectedPlanId, onSelect, isFreePlan, onQuickToggle, disabled }: SortablePlanItemProps) {
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging,
    } = useSortable({ id: plan.id, disabled });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        touchAction: 'none' as const
    };

    return (
        <PlanItem
            plan={plan}
            index={index}
            selectedPlanId={selectedPlanId}
            onSelect={onSelect}
            isFreePlan={isFreePlan}
            onQuickToggle={onQuickToggle}
            isDragging={isDragging}
            disabled={disabled}
            dragHandleProps={{ ...attributes, ...listeners } as Record<string, unknown>}
            style={style}
            innerRef={setNodeRef}
        />
    );
}
