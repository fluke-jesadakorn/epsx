/**
 * AccessControlPage Component
 * Unified Master-Detail interface for managing subscription plans and permission definitions
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
    Key,
    Loader2,
    Package,
    Plus,
    Search,
    ShieldAlert,
    Trash2
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { createPermissionAction, deletePermissionAction, getPermissionsAction } from '@/app/wallet-management/access/permission-actions';
import { updatePlanAction } from '@/app/wallet-management/plan-actions';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { DraggablePermissionItem, DroppablePermissionList } from '@/components/wallet/WalletComponents';
import { AccessItem, useWalletAccess } from '@/hooks/useWalletAccess';
import { type CreatePermissionRequest, type PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';

interface AccessControlPageProps {
    className?: string;
}

type ViewMode = 'plans' | 'permissions';

export function AccessControlPage({ className }: AccessControlPageProps) {
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
    const [mode, setMode] = useState<ViewMode>('plans');

    // DND Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // Data Hooks
    const {
        data: accessData,
        isLoading: isAccessLoading,
        refresh: refreshAccess
    } = useWalletAccess('');

    const [permissions, setPermissions] = useState<PermissionDefinition[]>([]);
    const [isPermissionsLoading, setIsPermissionsLoading] = useState(false);

    // Plan State
    const [planSearchQuery, setPlanSearchQuery] = useState('');
    const [selectedPlanId, setSelectedPlanId] = useState<string | null>(null);
    const [planForm, setPlanForm] = useState({ name: '', description: '', priority: 0, expiryDays: 30 });
    const [planPermissions, setPlanPermissions] = useState<string[]>([]);
    const [isSavingPlan, setIsSavingPlan] = useState(false);
    const [hasPlanChanges, setHasPlanChanges] = useState(false);
    const [availablePermissionSearch, setAvailablePermissionSearch] = useState('');

    // Permission State
    const [permissionSearchQuery, setPermissionSearchQuery] = useState('');
    const [selectedPermission, setSelectedPermission] = useState<PermissionDefinition | null>(null);
    const [isCreatePermissionOpen, setIsCreatePermissionOpen] = useState(false);
    const [deletePermissionConfirm, setDeletePermissionConfirm] = useState<PermissionDefinition | null>(null);

    // Validations
    const isLoading = authLoading || isAccessLoading || (mode === 'permissions' && isPermissionsLoading);

    // Initial Load for Permissions Page (independent of wallet access hook)
    const loadPermissions = useCallback(async () => {
        setIsPermissionsLoading(true);
        try {
            const result = await getPermissionsAction();
            if (result.success && result.data) {
                setPermissions(result.data);
            } else {
                console.error('Failed to load permissions:', result.error);
                toast.error('Failed to load permissions');
            }
        } catch (error) {
            console.error('Failed to load permissions:', error);
        } finally {
            setIsPermissionsLoading(false);
        }
    }, []);

    useEffect(() => {
        if (mode === 'permissions' && isAuthenticated) {
            loadPermissions();
        }
    }, [mode, isAuthenticated, loadPermissions]);

    // DERIVED DATA
    const allPlans = useMemo(() => {
        return [...accessData.authorizedPlans, ...accessData.availablePlans];
    }, [accessData]);

    const filteredPlans = useMemo(() => {
        return allPlans.filter(p => p.name.toLowerCase().includes(planSearchQuery.toLowerCase()));
    }, [allPlans, planSearchQuery]);

    const filteredPermissions = useMemo(() => {
        return permissions.filter(p =>
            p.permission_string.toLowerCase().includes(permissionSearchQuery.toLowerCase()) ||
            (p.name && p.name.toLowerCase().includes(permissionSearchQuery.toLowerCase()))
        );
    }, [permissions, permissionSearchQuery]);

    // DND Handlers
    const [activeDragItem, setActiveDragItem] = useState<AccessItem | null>(null);

    const handleDragStart = (event: DragStartEvent) => {
        if (mode !== 'plans') return;
        const { active } = event;
        const allSystemPermissions = [...accessData.availablePermissions, ...accessData.authorizedPermissions];
        // deduplicate logic for dragging source
        const item = active.data.current?.type === 'permission'
            ? allSystemPermissions.find(p => p.id === active.id)
            : null;

        if (item) setActiveDragItem(item);
    };

    const handleDragEnd = (event: DragEndEvent) => {
        if (mode !== 'plans') return;
        const { active, over } = event;
        setActiveDragItem(null);

        if (!over) return;

        if (active.data.current?.type === 'permission' && over.id === 'plan-builder-droppable') {
            const permissionName = active.data.current.name;
            if (selectedPlanId && !planPermissions.includes(permissionName)) {
                setPlanPermissions(prev => [...prev, permissionName]);
                setHasPlanChanges(true);
            }
        }
    };

    // Plan Actions
    const handleSelectPlan = (planId: string) => {
        const plan = allPlans.find(p => p.id === planId);
        if (plan) {
            setSelectedPlanId(planId);
            setPlanPermissions(plan.permissions || []);
            setPlanForm({
                name: plan.name,
                description: plan.description || '',
                // @ts-ignore
                priority: (plan as any).priority_level || 0,
                // @ts-ignore
                expiryDays: (plan as any).default_expiry_days || 30
            });
            setHasPlanChanges(false);
        }
    };

    const handleSavePlan = async () => {
        if (!selectedPlanId) return;
        setIsSavingPlan(true);
        try {
            await updatePlanAction(selectedPlanId, {
                name: planForm.name,
                description: planForm.description,
                permissions: planPermissions,
                priority_level: planForm.priority,
                default_expiry_days: planForm.expiryDays
            });
            toast.success('Plan updated successfully');
            setHasPlanChanges(false);
            refreshAccess();
        } catch (err) {
            toast.error('Failed to save plan');
        } finally {
            setIsSavingPlan(false);
        }
    };

    // Permission Actions
    const handleDeletePermission = async () => {
        if (!deletePermissionConfirm) return;
        try {
            const result = await deletePermissionAction(deletePermissionConfirm.id);
            if (result.success) {
                toast.success('Permission deleted');
                setDeletePermissionConfirm(null);
                setSelectedPermission(null);
                loadPermissions();
            } else {
                toast.error(result.error || 'Failed to delete permission');
            }
        } catch (error) {
            toast.error('Failed to delete permission');
        }
    };

    // Render Helpers
    const renderAvailablePermissionsForPlan = () => {
        // Filter out permissions already in the plan
        const assignedSet = new Set(planPermissions);
        const all = [...accessData.availablePermissions, ...accessData.authorizedPermissions];
        // Unique
        const unique = Array.from(new Map(all.map(item => [item.name, item])).values());

        return unique.filter(p => !assignedSet.has(p.name) && p.name.toLowerCase().includes(availablePermissionSearch.toLowerCase()));
    };

    if (isLoading && mode === 'plans' && !selectedPlanId) {
        // simple loading state
        return <div className="p-8 flex justify-center"><Loader2 className="animate-spin" /></div>;
    }

    return (
        <DndContext sensors={sensors} onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
            <div className={cn("space-y-6", className)}>
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 h-[calc(100vh-200px)] min-h-[600px]">

                    {/* LEFT SIDEBAR (List) */}
                    <div className="lg:col-span-4 flex flex-col gap-4 h-full">
                        {/* 1. Mode Switcher & Search Card */}
                        <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[24px] overflow-hidden flex-shrink-0">
                            <CardHeader className="p-4 bg-white/5 border-b border-white/5 space-y-4">
                                {/* Mode Toggles */}
                                <div className="bg-black/20 p-1 rounded-xl flex">
                                    <button
                                        onClick={() => { setMode('plans'); setSelectedPlanId(null); }}
                                        className={cn(
                                            "flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-bold transition-all",
                                            mode === 'plans'
                                                ? "bg-[#1fc7d4] text-white shadow-lg shadow-cyan-500/20"
                                                : "text-muted-foreground hover:text-white hover:bg-white/5"
                                        )}
                                    >
                                        <Package className="w-3.5 h-3.5" />
                                        <span>PLANS</span>
                                    </button>
                                    <button
                                        onClick={() => { setMode('permissions'); setSelectedPermission(null); }}
                                        className={cn(
                                            "flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-bold transition-all",
                                            mode === 'permissions'
                                                ? "bg-emerald-500 text-white shadow-lg shadow-emerald-500/20"
                                                : "text-muted-foreground hover:text-white hover:bg-white/5"
                                        )}
                                    >
                                        <Key className="w-3.5 h-3.5" />
                                        <span>PERMISSIONS</span>
                                    </button>
                                </div>

                                {/* Search & Add */}
                                <div className="flex gap-2">
                                    <div className="relative flex-1">
                                        <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground/50" />
                                        <Input
                                            placeholder={mode === 'plans' ? "Search plans..." : "Search permissions..."}
                                            value={mode === 'plans' ? planSearchQuery : permissionSearchQuery}
                                            onChange={e => mode === 'plans' ? setPlanSearchQuery(e.target.value) : setPermissionSearchQuery(e.target.value)}
                                            className="h-10 pl-9 text-sm bg-white/5 border-white/5 text-foreground rounded-xl"
                                        />
                                    </div>
                                    {mode === 'permissions' ? (
                                        <CreatePermissionDialog
                                            open={isCreatePermissionOpen}
                                            onOpenChange={setIsCreatePermissionOpen}
                                            onSuccess={() => { setIsCreatePermissionOpen(false); loadPermissions(); }}
                                        />
                                    ) : (
                                        <Button size="icon" className="h-10 w-10 shrink-0 bg-[#1fc7d4] hover:bg-[#1fc7d4]/80 text-white rounded-xl shadow-lg shadow-cyan-500/20" onClick={() => {/* TODO: New Plan */ }}>
                                            <Plus className="h-5 w-5" />
                                        </Button>
                                    )}
                                </div>
                            </CardHeader>

                            {/* List Content */}
                            <CardContent className="p-0 overflow-y-auto h-[400px] scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent">
                                <div className="divide-y divide-white/5">
                                    {mode === 'plans' ? (
                                        filteredPlans.map(plan => (
                                            <div
                                                key={plan.id}
                                                onClick={() => handleSelectPlan(plan.id)}
                                                className={cn(
                                                    "p-4 cursor-pointer hover:bg-white/5 transition-colors border-l-4",
                                                    selectedPlanId === plan.id
                                                        ? "bg-cyan-500/10 border-l-[#1fc7d4]"
                                                        : "border-l-transparent"
                                                )}
                                            >
                                                <div className="flex items-center justify-between">
                                                    <div>
                                                        <h4 className="font-bold text-sm text-foreground">{plan.name}</h4>
                                                        <p className="text-xs text-muted-foreground line-clamp-1">{plan.description}</p>
                                                    </div>
                                                    <Badge variant="secondary" className="text-[10px] h-5 bg-white/5">{plan.permissions?.length || 0}</Badge>
                                                </div>
                                            </div>
                                        ))
                                    ) : (
                                        filteredPermissions.map(perm => (
                                            <div
                                                key={perm.id}
                                                onClick={() => setSelectedPermission(perm)}
                                                className={cn(
                                                    "p-4 cursor-pointer hover:bg-white/5 transition-colors border-l-4",
                                                    selectedPermission?.id === perm.id
                                                        ? "bg-emerald-500/10 border-l-emerald-500"
                                                        : "border-l-transparent"
                                                )}
                                            >
                                                <div className="flex items-center justify-between gap-2">
                                                    <div className="min-w-0">
                                                        <p className="font-mono text-xs font-bold text-emerald-400 truncate">{perm.permission_string}</p>
                                                        <p className="text-xs text-muted-foreground line-clamp-1">{perm.name || 'Unnamed'}</p>
                                                    </div>
                                                    {perm.is_system && <ShieldAlert className="w-3 h-3 text-amber-500 shrink-0" />}
                                                </div>
                                            </div>
                                        ))
                                    )}
                                </div>
                            </CardContent>
                        </Card>

                        {/* Plan Mode Extra: Available Permissions Source */}
                        {mode === 'plans' && selectedPlanId && (
                            <Card className="flex-1 border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[24px] overflow-hidden min-h-0 flex flex-col">
                                <CardHeader className="py-3 px-4 border-b border-white/5 bg-white/5 flex flex-row items-center justify-between">
                                    <span className="text-xs font-bold uppercase tracking-wider text-muted-foreground">Available Permissions</span>
                                    <Input
                                        className="h-7 w-32 text-xs bg-black/20 border-transparent"
                                        placeholder="Filter..."
                                        value={availablePermissionSearch}
                                        onChange={e => setAvailablePermissionSearch(e.target.value)}
                                    />
                                </CardHeader>
                                <CardContent className="p-2 overflow-y-auto flex-1 min-h-0">
                                    <div className="space-y-2">
                                        {renderAvailablePermissionsForPlan().map(perm => (
                                            <DraggablePermissionItem key={perm.id} id={perm.id} label={perm.name} />
                                        ))}
                                    </div>
                                </CardContent>
                            </Card>
                        )}
                    </div>

                    {/* RIGHT CONTENT (Editor) */}
                    <div className="lg:col-span-8 h-full flex flex-col">
                        {mode === 'plans' ? (
                            selectedPlanId ? (
                                <Card className="h-full border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden flex flex-col">
                                    <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5">
                                        <div className="flex items-center justify-between">
                                            <div className="flex items-center gap-3">
                                                <div className="h-10 w-10 rounded-xl bg-[#1fc7d4]/20 flex items-center justify-center">
                                                    <Package className="w-5 h-5 text-[#1fc7d4]" />
                                                </div>
                                                <div>
                                                    <CardTitle className="text-lg font-bold">Edit Plan</CardTitle>
                                                    <p className="text-xs text-muted-foreground font-mono">{selectedPlanId}</p>
                                                </div>
                                            </div>
                                            <div className="flex gap-2">
                                                <Button size="sm" variant="ghost" onClick={() => handleSelectPlan(selectedPlanId)}>Reset</Button>
                                                <Button size="sm" onClick={handleSavePlan} disabled={!hasPlanChanges || isSavingPlan} className="bg-[#1fc7d4] text-white hover:bg-[#1fc7d4]/90">
                                                    {isSavingPlan && <Loader2 className="w-3 h-3 animate-spin mr-2" />}
                                                    Save Changes
                                                </Button>
                                            </div>
                                        </div>
                                    </CardHeader>
                                    <CardContent className="p-8 flex-1 overflow-y-auto space-y-8">
                                        {/* Form */}
                                        <div className="grid grid-cols-2 gap-6">
                                            <div className="space-y-2">
                                                <Label>Plan Name</Label>
                                                <Input value={planForm.name} onChange={e => { setPlanForm(prev => ({ ...prev, name: e.target.value })); setHasPlanChanges(true); }} className="bg-white/5 border-white/10" />
                                            </div>
                                            <div className="space-y-2">
                                                <Label>Priority</Label>
                                                <Input type="number" value={planForm.priority} onChange={e => { setPlanForm(prev => ({ ...prev, priority: parseInt(e.target.value) })); setHasPlanChanges(true); }} className="bg-white/5 border-white/10" />
                                            </div>
                                            <div className="col-span-2 space-y-2">
                                                <Label>Description</Label>
                                                <Textarea value={planForm.description} onChange={e => { setPlanForm(prev => ({ ...prev, description: e.target.value })); setHasPlanChanges(true); }} className="bg-white/5 border-white/10" />
                                            </div>
                                        </div>

                                        {/* Droppable Area */}
                                        <div className="space-y-4">
                                            <div className="flex items-center justify-between">
                                                <Label className="text-[#1fc7d4] uppercase tracking-wider font-bold text-xs">Assigned Permissions</Label>
                                                <Badge variant="outline" className="border-cyan-500/30 text-[#1fc7d4]">{planPermissions.length}</Badge>
                                            </div>
                                            <div className="border-2 border-dashed border-white/10 rounded-2xl bg-black/20 min-h-[200px] p-4">
                                                <DroppablePermissionList
                                                    id="plan-builder-droppable"
                                                    items={planPermissions}
                                                    emptyMessage="Drag permissions here from the bottom-left sidebar"
                                                    onRemoveItem={(perm) => {
                                                        setPlanPermissions(prev => prev.filter(p => p !== perm));
                                                        setHasPlanChanges(true);
                                                    }}
                                                />
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>
                            ) : (
                                <EmptyState icon={Package} title="Select a Plan" description="Choose a plan from the sidebar to edit details and permissions." />
                            )
                        ) : (
                            // PERMISSIONS MODE RIGHT SIDE
                            selectedPermission ? (
                                <Card className="h-full border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden flex flex-col">
                                    <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5 flex flex-row items-center justify-between">
                                        <div className="flex items-center gap-3">
                                            <div className="h-10 w-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
                                                <Key className="w-5 h-5 text-emerald-500" />
                                            </div>
                                            <div>
                                                <CardTitle className="text-lg font-bold">Permission Details</CardTitle>
                                                <div className="flex items-center gap-2 mt-1">
                                                    <code className="text-xs bg-black/30 px-2 py-0.5 rounded text-emerald-400 font-mono">{selectedPermission.permission_string}</code>
                                                    {selectedPermission.is_system && <Badge variant="secondary" className="text-[10px] h-4">System</Badge>}
                                                </div>
                                            </div>
                                        </div>
                                        {!selectedPermission.is_system && (
                                            <Button variant="destructive" size="sm" onClick={() => setDeletePermissionConfirm(selectedPermission)}>
                                                <Trash2 className="w-4 h-4 mr-2" />
                                                Delete Permission
                                            </Button>
                                        )}
                                    </CardHeader>
                                    <CardContent className="p-8 space-y-6">
                                        <div className="space-y-4">
                                            <div className="grid gap-2">
                                                <Label className="text-muted-foreground">Display Name</Label>
                                                <div className="p-3 rounded-xl bg-white/5 border border-white/10 font-medium">{selectedPermission.name || 'N/A'}</div>
                                            </div>
                                            <div className="grid gap-2">
                                                <Label className="text-muted-foreground">Description</Label>
                                                <div className="p-3 rounded-xl bg-white/5 border border-white/10 text-sm text-slate-300 min-h-[80px]">
                                                    {selectedPermission.description || 'No description provided.'}
                                                </div>
                                            </div>
                                            <div className="grid grid-cols-2 gap-4">
                                                <div className="grid gap-2">
                                                    <Label className="text-muted-foreground">Platform</Label>
                                                    <div className="p-3 rounded-xl bg-white/5 border border-white/10 text-sm">{selectedPermission.platform}</div>
                                                </div>
                                                <div className="grid gap-2">
                                                    <Label className="text-muted-foreground">Category</Label>
                                                    <div className="p-3 rounded-xl bg-white/5 border border-white/10 text-sm">{selectedPermission.category || 'General'}</div>
                                                </div>
                                            </div>
                                        </div>

                                        <div className="p-4 rounded-xl bg-blue-500/10 border border-blue-500/20 flex gap-3 text-sm text-blue-200">
                                            <ShieldAlert className="w-5 h-5 shrink-0" />
                                            <div>
                                                <p className="font-bold mb-1">Permission Usage</p>
                                                <p className="opacity-80">This permission definition is referenced by its unique string. Renaming or modifying the string is not supported to prevent breaking dependent policies.</p>
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>
                            ) : (
                                <EmptyState icon={Key} title="Select a Permission" description="Choose a permission to view details." />
                            )
                        )}
                    </div>
                </div>

                {/* Dialogs & Overlays */}
                <DragOverlay>
                    {activeDragItem && (
                        <div className={cn(
                            "flex items-center gap-2 px-3 py-2 bg-slate-800 rounded-lg shadow-xl border border-white/10 opacity-90 scale-105 pointer-events-none text-white",
                            activeDragItem.type === 'permission' ? "border-emerald-500" : "border-cyan-500"
                        )}>
                            <Key className="h-4 w-4" />
                            <span className="font-medium text-sm">{activeDragItem.name}</span>
                        </div>
                    )}
                </DragOverlay>

                {/* Delete Permission Dialog */}
                <Dialog open={!!deletePermissionConfirm} onOpenChange={(open) => !open && setDeletePermissionConfirm(null)}>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Delete Permission</DialogTitle>
                            <DialogDescription>
                                Are you sure you want to delete <span className="font-mono font-medium text-foreground">{deletePermissionConfirm?.permission_string}</span>?
                                This cannot be undone.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setDeletePermissionConfirm(null)}>Cancel</Button>
                            <Button variant="destructive" onClick={handleDeletePermission}>Delete</Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>
        </DndContext>
    );
}

function EmptyState({ icon: Icon, title, description }: { icon: any, title: string, description: string }) {
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

function CreatePermissionDialog({
    open,
    onOpenChange,
    onSuccess
}: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess: () => void;
}) {
    const [formData, setFormData] = useState<CreatePermissionRequest>({
        permission: '',
        name: '',
        description: '',
        platform: '',
        category: '',
    });
    const [isSubmitting, setIsSubmitting] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsSubmitting(true);

        try {
            const result = await createPermissionAction(formData);
            if (result.success) {
                toast.success('Permission created successfully');
                setFormData({
                    permission: '',
                    name: '',
                    description: '',
                    platform: '',
                    category: '',
                });
                onSuccess();
            } else {
                toast.error(result.error || 'Failed to create permission');
            }
        } catch (error) {
            console.error('Failed to create permission:', error);
            toast.error(error instanceof Error ? error.message : 'Failed to create permission');
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogTrigger asChild>
                <Button size="icon" className="h-10 w-10 shrink-0 bg-emerald-500 hover:bg-emerald-600 text-white rounded-xl shadow-lg shadow-emerald-500/20">
                    <Plus className="h-5 w-5" />
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[500px]">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>Create Permission</DialogTitle>
                        <DialogDescription>
                            Define a new permission string. Format: <code>platform:resource:action</code>
                        </DialogDescription>
                    </DialogHeader>

                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label htmlFor="permission">Permission String *</Label>
                            <Input
                                id="permission"
                                placeholder="e.g. admin:users:delete"
                                value={formData.permission}
                                onChange={(e) => setFormData({ ...formData, permission: e.target.value })}
                                required
                                className="font-mono bg-white/5"
                            />
                            <p className="text-[0.8rem] text-muted-foreground">
                                Must contain at least 3 parts separated by colons.
                            </p>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="grid gap-2">
                                <Label htmlFor="name">Display Name</Label>
                                <Input
                                    id="name"
                                    placeholder="Delete Users"
                                    value={formData.name || ''}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="bg-white/5"
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="category">Category</Label>
                                <Input
                                    id="category"
                                    placeholder="User Management"
                                    value={formData.category || ''}
                                    onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                                    className="bg-white/5"
                                />
                            </div>
                        </div>

                        <div className="grid gap-2">
                            <Label htmlFor="description">Description</Label>
                            <Textarea
                                id="description"
                                placeholder="Allows deleting user accounts..."
                                value={formData.description || ''}
                                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                                className="bg-white/5"
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                            Cancel
                        </Button>
                        <Button type="submit" disabled={isSubmitting} className="bg-emerald-500 hover:bg-emerald-600 border-none">
                            {isSubmitting ? 'Creating...' : 'Create Permission'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
}
