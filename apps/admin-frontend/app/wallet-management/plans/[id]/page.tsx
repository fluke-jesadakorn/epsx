/**
 * Plan Detail Page
 * Drag and Drop interface for managing Plan Permissions
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
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, Loader2, Shield } from 'lucide-react';
import Link from 'next/link';
import { useParams, useRouter, useSearchParams } from 'next/navigation';
import { useMemo, useState } from 'react';
import { toast } from 'sonner';

import { TrashDropZone } from '@/components/wallet/TrashDropZone';
import {
    DraggablePermissionItem,
    DroppablePermissionList
} from '@/components/wallet/WalletComponents';
import { Badge } from '@/shared/components/ui/badge';
import { Button } from '@/shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/components/ui/card';
import { Input } from '@/shared/components/ui/input';
import { Label } from '@/shared/components/ui/label';
import { Skeleton } from '@/shared/components/ui/skeleton';
import { Textarea } from '@/shared/components/ui/textarea';

import { deletePlanAction, getPlanAction, getPlansAction, updatePlanAction } from '@/app/wallet-management/plan-actions';
import { useAvailablePermissions } from '@/hooks/usePlanPermissions';
import { PermissionPlan } from '@/lib/api/plan-management-client';

export default function PlanDetailPage() {
    const router = useRouter();
    const params = useParams();
    const searchParams = useSearchParams();
    const from = searchParams.get('from');

    const planId = params['id'] as string;
    const queryClient = useQueryClient();

    // Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // Data Fetching
    const { data: plan, isLoading: planLoading } = useQuery({
        queryKey: ['permission-plan', planId],
        queryFn: async () => {
            try {
                return await getPlanAction(planId);
            } catch (e) {
                const plans = await getPlansAction();
                return plans.find((g: PermissionPlan) => g.id === planId) || null;
            }
        },
        enabled: !!planId
    });

    const { permissions: availablePermissionsRaw, isLoading: permissionsLoading } = useAvailablePermissions();

    // Local State
    const [formData, setFormData] = useState<Partial<PermissionPlan>>({});
    const [assignedPermissionIds, setAssignedPermissionIds] = useState<string[]>([]);
    const [activeDragId, setActiveDragId] = useState<string | null>(null);
    const [isSaving, setIsSaving] = useState(false);
    const [pendingChanges, setPendingChanges] = useState(false);

    // Initialize state when data loads
    useMemo(() => {
        if (plan) {
            setFormData({
                name: plan.name,
                description: plan.description,
                priority_level: plan.priority_level,
                default_expiry_days: plan.default_expiry_days
            });
            setAssignedPermissionIds(plan.permissions || []);
        }
    }, [plan]);

    // Computed Lists
    const availablePermissions = useMemo(() => {
        return availablePermissionsRaw.filter(p => !assignedPermissionIds.includes(p));
    }, [availablePermissionsRaw, assignedPermissionIds]);

    // Handlers
    const handleDragStart = (event: DragStartEvent) => {
        setActiveDragId(event.active.id as string);
    };

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveDragId(null);

        if (!over) return;

        // Dropped in Assigned List
        if (over.id === 'assigned-list-droppable') {
            const permissionId = active.id as string;
            if (!assignedPermissionIds.includes(permissionId)) {
                setAssignedPermissionIds(prev => [...prev, permissionId]);
                setPendingChanges(true);
                toast.success('Permission staged for assignment');
            }
        }

        // Dropped in Trash
        if (over.id === 'trash') {
            const permissionId = active.id as string;
            if (assignedPermissionIds.includes(permissionId)) {
                setAssignedPermissionIds(prev => prev.filter(id => id !== permissionId));
                setPendingChanges(true);
                toast.info('Permission removed');
            }
        }
    };

    const handleSave = async () => {
        if (!plan) return;
        setIsSaving(true);
        try {
            await updatePlanAction(plan.id, {
                ...formData,
                default_expiry_days: formData.default_expiry_days === null ? undefined : formData.default_expiry_days,
                permissions: assignedPermissionIds
            });
            toast.success('Plan updated successfully');
            queryClient.invalidateQueries({ queryKey: ['permission-plan', planId] });
            setPendingChanges(false);
        } catch (err: any) {
            toast.error(err.message || 'Failed to update plan');
        } finally {
            setIsSaving(false);
        }
    };

    const handleDelete = async () => {
        if (!confirm('Are you sure you want to delete this plan?')) return;
        setIsSaving(true);
        try {
            await deletePlanAction(planId);
            toast.success('Plan deleted');
            queryClient.invalidateQueries({ queryKey: ['permission-plans'] });
            router.push(from || '/wallet-management/plans');
        } catch (err: any) {
            toast.error(err.message || 'Failed to delete plan');
            setIsSaving(false);
        }
    };

    if (planLoading || permissionsLoading) {
        return (
            <div className="p-6 max-w-6xl mx-auto space-y-6">
                <Skeleton className="h-12 w-1/3" />
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <Skeleton className="h-96" />
                    <Skeleton className="h-96 lg:col-span-2" />
                </div>
            </div>
        );
    }

    if (!plan) return <div>Plan not found</div>;

    return (
        <DndContext sensors={sensors} onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
            <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-4 sm:p-6 pb-24">
                <div className="max-w-6xl mx-auto space-y-6">

                    {/* Header */}
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-4">
                            <Link href={from || "/wallet-management/plans"} className="p-2 border rounded-xl hover:bg-white transition-colors">
                                <ArrowLeft className="h-5 w-5" />
                            </Link>
                            <div>
                                <h1 className="text-2xl font-bold flex items-center gap-2">
                                    <Shield className="h-6 w-6 text-purple-600" />
                                    Plan Management
                                </h1>
                                <p className="text-gray-500 text-sm">Manage plan details and permissions</p>
                            </div>
                        </div>
                        {pendingChanges && (
                            <Badge variant="outline" className="bg-amber-50 text-amber-600 border-amber-200 animate-pulse">
                                Unsaved Changes
                            </Badge>
                        )}
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">

                        {/* Left Column: Plan Details (4 cols) */}
                        <div className="lg:col-span-5 space-y-6">
                            <Card className="border-t-4 border-t-purple-500 shadow-sm">
                                <CardHeader>
                                    <CardTitle>Edit Plan Details</CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-4">
                                    <div className="space-y-2">
                                        <Label>Plan Name</Label>
                                        <Input
                                            value={formData.name || ''}
                                            onChange={e => { setFormData({ ...formData, name: e.target.value }); setPendingChanges(true); }}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label>Description</Label>
                                        <Textarea
                                            value={formData.description || ''}
                                            onChange={e => { setFormData({ ...formData, description: e.target.value }); setPendingChanges(true); }}
                                        />
                                    </div>
                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="space-y-2">
                                            <Label>Priority</Label>
                                            <Input
                                                type="number"
                                                value={formData.priority_level || 0}
                                                onChange={e => { setFormData({ ...formData, priority_level: parseInt(e.target.value) }); setPendingChanges(true); }}
                                            />
                                        </div>
                                        <div className="space-y-2">
                                            <Label>Expiry (Days)</Label>
                                            <Input
                                                type="number"
                                                value={formData.default_expiry_days || ''}
                                                onChange={e => { setFormData({ ...formData, default_expiry_days: e.target.value ? parseInt(e.target.value) : undefined }); setPendingChanges(true); }}
                                            />
                                        </div>
                                    </div>

                                    <div className="flex gap-2 pt-4">
                                        <Button
                                            variant="destructive"
                                            className="flex-1"
                                            onClick={handleDelete}
                                        >
                                            Delete
                                        </Button>
                                        <Button
                                            className="flex-1 bg-purple-600 hover:bg-purple-700"
                                            onClick={handleSave}
                                            disabled={isSaving}
                                        >
                                            {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                            Save Changes
                                        </Button>
                                    </div>
                                </CardContent>
                            </Card>

                            {/* Assigned Permissions (Droppable) */}
                            <Card className="flex flex-col h-[500px]">
                                <CardHeader className="bg-purple-50 dark:bg-purple-900/10 border-b">
                                    <CardTitle className="text-base flex items-center justify-between">
                                        <span>Assigned Permissions</span>
                                        <Badge variant="secondary">{assignedPermissionIds.length}</Badge>
                                    </CardTitle>
                                </CardHeader>
                                <div className="flex-1 overflow-y-auto p-4 bg-gray-50/50 dark:bg-gray-900/50">
                                    <DroppablePermissionList
                                        id="assigned-list-droppable"
                                        items={assignedPermissionIds}
                                        emptyMessage="Drag permissions here to assign"
                                    />
                                </div>
                            </Card>
                        </div>

                        {/* Right Column: Available Permissions (Draggable) (7 cols) */}
                        <div className="lg:col-span-7 space-y-6">
                            {/* Available Permissions Source */}
                            <Card className="flex flex-col h-[850px]">
                                <CardHeader className="border-b px-4 py-3">
                                    <div className="flex items-center justify-between">
                                        <CardTitle className="text-base">Available Permissions</CardTitle>
                                        <Input placeholder="Search..." className="w-48 h-8" />
                                    </div>
                                </CardHeader>
                                <div className="flex-1 overflow-y-auto p-4">
                                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                                        {availablePermissions.map(permId => (
                                            <DraggablePermissionItem key={permId} id={permId} label={permId} />
                                        ))}
                                    </div>
                                </div>
                            </Card>
                        </div>
                    </div>
                </div>

                <DragOverlay>
                    {activeDragId ? (
                        <div className="px-4 py-2 bg-white rounded-lg shadow-xl border border-purple-500 font-medium">
                            {activeDragId}
                        </div>
                    ) : null}
                </DragOverlay>

                <TrashDropZone isDragging={!!activeDragId} />
            </div>
        </DndContext>
    );
}

