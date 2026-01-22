/**
 * Create Plan Page
 * Page for creating a new permission plan
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
import { useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, Loader2, Plus } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useMemo, useState } from 'react';
import { toast } from 'sonner';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { TrashDropZone } from '@/components/wallet/TrashDropZone';
import {
    DraggablePermissionItem,
    DroppablePermissionList
} from '@/components/wallet/WalletComponents';

import { useAvailablePermissions } from '@/hooks/usePlanPermissions';
import { CreatePlanRequest, planMgmt } from '@/lib/api/plan-management-client';

export default function CreatePlanPage() {
    const router = useRouter();
    const queryClient = useQueryClient();

    // Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // Data Fetching
    const { permissions: availablePermissionsRaw } = useAvailablePermissions();

    // Local State
    const [formData, setFormData] = useState<Partial<CreatePlanRequest>>({
        name: '',
        description: '',
        priority_level: 0,
    });
    const [assignedPermissionIds, setAssignedPermissionIds] = useState<string[]>([]);
    const [activeDragId, setActiveDragId] = useState<string | null>(null);
    const [isSaving, setIsSaving] = useState(false);

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
                toast.success('Permission staged');
            }
        }

        // Dropped in Trash
        if (over.id === 'trash') {
            const permissionId = active.id as string;
            if (assignedPermissionIds.includes(permissionId)) {
                setAssignedPermissionIds(prev => prev.filter(id => id !== permissionId));
            }
        }
    };

    const handleCreate = async () => {
        if (!formData.name) {
            toast.error('Plan name is required');
            return;
        }

        setIsSaving(true);
        try {
            await planMgmt.createPlan({
                name: formData.name,
                description: formData.description,
                priority_level: formData.priority_level || 0,
                default_expiry_days: formData.default_expiry_days,
                permissions: assignedPermissionIds
            } as CreatePlanRequest);

            toast.success('Plan created successfully');
            queryClient.invalidateQueries({ queryKey: ['permission-plans'] });
            router.push('/wallet-management/plans');
        } catch (err: any) {
            toast.error(err.message || 'Failed to create plan');
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <DndContext sensors={sensors} onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
            <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-4 sm:p-6 pb-24">
                <div className="max-w-6xl mx-auto space-y-6">

                    {/* Header */}
                    <div className="flex items-center gap-4">
                        <Link href="/wallet-management/plans" className="p-2 border rounded-xl hover:bg-white transition-colors">
                            <ArrowLeft className="h-5 w-5" />
                        </Link>
                        <div>
                            <h1 className="text-2xl font-bold flex items-center gap-2">
                                <Plus className="h-6 w-6 text-purple-600" />
                                Create New Plan
                            </h1>
                            <p className="text-gray-500 text-sm">Define a new set of permissions</p>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">

                        {/* Left Column: Plan Details */}
                        <div className="lg:col-span-5 space-y-6">
                            <Card className="border-t-4 border-t-purple-500 shadow-sm">
                                <CardHeader>
                                    <CardTitle>Plan Details</CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-4">
                                    <div className="space-y-2">
                                        <Label>Plan Name <span className="text-red-500">*</span></Label>
                                        <Input
                                            value={formData.name || ''}
                                            onChange={e => setFormData({ ...formData, name: e.target.value })}
                                            placeholder="e.g. VIP Trader"
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label>Description</Label>
                                        <Textarea
                                            value={formData.description || ''}
                                            onChange={e => setFormData({ ...formData, description: e.target.value })}
                                            placeholder="Describe the purpose of this plan"
                                        />
                                    </div>
                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="space-y-2">
                                            <Label>Priority</Label>
                                            <Input
                                                type="number"
                                                value={formData.priority_level}
                                                onChange={e => setFormData({ ...formData, priority_level: parseInt(e.target.value) || 0 })}
                                            />
                                        </div>
                                        <div className="space-y-2">
                                            <Label>Expiry (Days)</Label>
                                            <Input
                                                type="number"
                                                value={formData.default_expiry_days || ''}
                                                onChange={e => setFormData({ ...formData, default_expiry_days: e.target.value ? parseInt(e.target.value) : undefined })}
                                                placeholder="Optional"
                                            />
                                        </div>
                                    </div>

                                    <div className="pt-4">
                                        <Button
                                            className="w-full bg-purple-600 hover:bg-purple-700"
                                            onClick={handleCreate}
                                            disabled={isSaving}
                                        >
                                            {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                            Create Plan
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

                        {/* Right Column: Available Permissions (Draggable) */}
                        <div className="lg:col-span-7 space-y-6">
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

