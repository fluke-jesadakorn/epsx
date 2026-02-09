'use client';

import type {
    DragEndEvent,
    DragStartEvent
} from '@dnd-kit/core';
import {
    DndContext,
    DragOverlay,
    MouseSensor,
    TouchSensor,
    useSensor,
    useSensors
} from '@dnd-kit/core';
import {
    Key,
    Loader2,
    Plus,
    Search,
    ShieldAlert,
    Trash2
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { createPermissionAction, deletePermissionAction, getPermissionsAction, updatePermissionAction } from '@/app/wallet-management/access/permission-actions';
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
import { Textarea } from '@/components/ui/textarea';
import type { CreatePermissionRequest, PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth';
import { logger } from '@/shared/utils/logger';

interface PermissionsViewProps {
    className?: string;
}

// eslint-disable-next-line max-lines-per-function, complexity
export function PermissionsView({ className }: PermissionsViewProps) {
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // DND Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // --- DATA STATE ---
    const [permissions, setPermissions] = useState<PermissionDefinition[]>([]);
    const [isLoadingData, setIsLoadingData] = useState(false);

    // --- PERMISSION MODE STATE ---
    const [permSearch, setPermSearch] = useState('');
    const [selectedPerm, setSelectedPerm] = useState<PermissionDefinition | null>(null);
    const [permEditForm, setPermEditForm] = useState<Partial<PermissionDefinition>>({});
    const [isSavingPerm, setIsSavingPerm] = useState(false);
    const [hasPermChanges, setHasPermChanges] = useState(false);
    const [isCreatePermOpen, setIsCreatePermOpen] = useState(false);
    const [permDeleteConfirm, setPermDeleteConfirm] = useState<PermissionDefinition | null>(null);

    // DND State
    const [activeDragId, setActiveDragId] = useState<string | null>(null);

    // --- DATA FETCHING ---
    const loadData = useCallback(async () => {
        setIsLoadingData(true);
        try {
            const permRes = await getPermissionsAction();
            if (permRes.success && permRes.data) {
                setPermissions(permRes.data);
            }
        } catch (error: unknown) {
            logger.error('Failed to load permissions:', error instanceof Error ? error.message : String(error));
            toast.error('Failed to load permissions data');
        } finally {
            setIsLoadingData(false);
        }
    }, []);

    useEffect(() => {
        if (isAuthenticated) {
            void loadData();
        }
    }, [isAuthenticated, loadData]);

    // --- DERIVED DATA ---
    const filteredPermissions = useMemo(() =>
        permissions.filter(p =>
            p.permission_string.toLowerCase().includes(permSearch.toLowerCase()) ||
            (p.name !== null && p.name !== '' && p.name.toLowerCase().includes(permSearch.toLowerCase()))
        ),
        [permissions, permSearch]);

    // --- EVENT HANDLERS ---
    const handleSelectPermission = (perm: PermissionDefinition) => {
        setSelectedPerm(perm);
        setPermEditForm({
            name: perm.name ?? '',
            description: perm.description ?? '',
            category: perm.category ?? ''
        });
        setHasPermChanges(false);
    };

    const handleSavePermission = async () => {
        if (!selectedPerm) { return; }
        setIsSavingPerm(true);
        try {
            const result = await updatePermissionAction(selectedPerm.id, {
                name: permEditForm.name ?? undefined,
                description: permEditForm.description ?? undefined,
                category: permEditForm.category ?? undefined
            });
            if (result.success && result.data) {
                toast.success('Permission updated');
                const updated = result.data;
                setPermissions(prev => prev.map(p => p.id === updated.id ? updated : p));
                setSelectedPerm(updated);
                setHasPermChanges(false);
            } else {
                toast.error(result.error ?? 'Failed to update');
            }
        } catch (err: unknown) {
            logger.error('Failed to update permission:', err instanceof Error ? err.message : String(err));
            toast.error('Failed to update permission');
        } finally { setIsSavingPerm(false); }
    };

    const handleDeletePermission = async () => {
        if (!permDeleteConfirm) { return; }
        try {
            const result = await deletePermissionAction(permDeleteConfirm.id);
            if (result.success) {
                toast.success('Permission deleted');
                setPermissions(prev => prev.filter(p => p.id !== permDeleteConfirm.id));
                if (selectedPerm?.id === permDeleteConfirm.id) { setSelectedPerm(null); }
                setPermDeleteConfirm(null);
            } else {
                toast.error(result.error);
            }
        } catch (err: unknown) {
            logger.error('Failed to delete permission:', err instanceof Error ? err.message : String(err));
            toast.error('Failed to delete permission');
        }
    };

    const handleDragStart = (event: DragStartEvent) => {
        setActiveDragId(event.active.id as string);
    };

    const handleDragEnd = (_event: DragEndEvent) => {
        setActiveDragId(null);
    };

    if (authLoading || (isLoadingData && !permissions.length)) {
        return <div className="p-8 flex justify-center"><Loader2 className="animate-spin" /></div>;
    }

    return (
        <DndContext sensors={sensors} onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
            <div className={cn("grid grid-cols-1 lg:grid-cols-12 gap-6 h-[calc(100vh-250px)] min-h-[500px]", className)}>
                {/* SIDEBAR */}
                <div className="lg:col-span-4 flex flex-col gap-4 h-full">
                    <Card className="border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[24px] overflow-hidden flex flex-col h-full">
                        <CardHeader className="p-4 bg-white/5 border-b border-white/5 space-y-4 shrink-0">
                            <div className="flex items-center justify-between">
                                <h2 className="text-sm font-bold flex items-center gap-2 uppercase tracking-wider text-muted-foreground">
                                    All Permissions
                                </h2>
                                <Badge variant="secondary" className="bg-emerald-500/10 text-emerald-500">{permissions.length}</Badge>
                            </div>
                            <div className="flex gap-2">
                                <div className="relative flex-1">
                                    <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground/50" />
                                    <Input
                                        placeholder="Search..."
                                        value={permSearch}
                                        onChange={e => setPermSearch(e.target.value)}
                                        className="h-9 pl-9 text-sm bg-white/5 border-white/5"
                                    />
                                </div>
                                <CreatePermissionSheet
                                    open={isCreatePermOpen}
                                    onOpenChange={setIsCreatePermOpen}
                                    onSuccess={() => void loadData()}
                                />
                            </div>
                        </CardHeader>
                        <CardContent className="p-0 overflow-y-auto flex-1 scrollbar-thin scrollbar-thumb-white/10">
                            <div className="divide-y divide-white/5">
                                {filteredPermissions.map(perm => (
                                    <div
                                        key={perm.id}
                                        onClick={() => handleSelectPermission(perm)}
                                        className={cn(
                                            "p-4 cursor-pointer hover:bg-white/5 transition-colors border-l-4",
                                            selectedPerm?.id === perm.id ? "bg-emerald-500/10 border-l-emerald-500" : "border-l-transparent"
                                        )}
                                    >
                                        <div className="flex items-center justify-between gap-2">
                                            <div className="min-w-0">
                                                <p className="font-mono text-xs font-bold text-emerald-400 truncate">{perm.permission_string}</p>
                                                <p className="text-xs text-muted-foreground line-clamp-1">{perm.name ?? 'Unnamed'}</p>
                                            </div>
                                            {perm.is_system && <ShieldAlert className="w-3 h-3 text-amber-500 shrink-0" />}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {/* EDITOR */}
                <div className="lg:col-span-8 h-full">
                    {selectedPerm ? (
                        <Card className="h-full border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden flex flex-col">
                            <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5 flex flex-row items-center justify-between shrink-0">
                                <div className="flex items-center gap-3">
                                    <div className="h-10 w-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
                                        <Key className="w-5 h-5 text-emerald-500" />
                                    </div>
                                    <div>
                                        <CardTitle className="text-lg font-bold">Edit Permission</CardTitle>
                                        <div className="flex items-center gap-2 mt-1">
                                            <code className="text-xs bg-black/30 px-2 py-0.5 rounded text-emerald-400 font-mono">{selectedPerm.permission_string}</code>
                                            {selectedPerm.is_system && <Badge variant="secondary" className="text-[10px] h-4">System</Badge>}
                                        </div>
                                    </div>
                                </div>
                                <div className="flex gap-2">
                                    {!selectedPerm.is_system && (
                                        <Button variant="ghost" className="text-red-400 hover:text-red-300 hover:bg-red-500/10" size="sm" onClick={() => setPermDeleteConfirm(selectedPerm)}>
                                            <Trash2 className="w-4 h-4 mr-2" />
                                            Delete
                                        </Button>
                                    )}
                                    <Button size="sm" onClick={() => void handleSavePermission()} disabled={!hasPermChanges || isSavingPerm} className="bg-emerald-500 text-white hover:bg-emerald-600">
                                        {isSavingPerm && <Loader2 className="w-3 h-3 animate-spin mr-2" />}
                                        Save Changes
                                    </Button>
                                </div>
                            </CardHeader>
                            <CardContent className="p-8 space-y-6 overflow-y-auto flex-1">
                                <div className="space-y-4 max-w-2xl">
                                    <div className="grid gap-2">
                                        <Label>Display Name</Label>
                                        <Input
                                            value={permEditForm.name ?? ''}
                                            onChange={e => { setPermEditForm(p => ({ ...p, name: e.target.value })); setHasPermChanges(true); }}
                                            className="bg-white/5 border-white/10"
                                        />
                                    </div>
                                    <div className="grid gap-2">
                                        <Label>Category</Label>
                                        <Input
                                            value={permEditForm.category ?? ''}
                                            onChange={e => { setPermEditForm(p => ({ ...p, category: e.target.value })); setHasPermChanges(true); }}
                                            className="bg-white/5 border-white/10"
                                        />
                                    </div>
                                    <div className="grid gap-2">
                                        <Label>Description</Label>
                                        <Textarea
                                            value={permEditForm.description ?? ''}
                                            onChange={e => { setPermEditForm(p => ({ ...p, description: e.target.value })); setHasPermChanges(true); }}
                                            className="bg-white/5 border-white/10 min-h-[100px]"
                                        />
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    ) : (
                        <EmptyState icon={Key} title="Select a permission" description="Select to edit details." />
                    )}
                </div>

                {/* Drag Overlay */}
                <DragOverlay>
                    {activeDragId !== null && (
                        <div className="flex items-center gap-2 px-3 py-2 bg-slate-800 rounded-lg shadow-xl border border-emerald-500 opacity-90 scale-105 pointer-events-none text-white">
                            <Key className="h-4 w-4" />
                            <span className="font-medium text-sm">Permission Item</span>
                        </div>
                    )}
                </DragOverlay>

                {/* Delete Confirmation */}
                <Dialog open={Boolean(permDeleteConfirm)} onOpenChange={(o) => !o && setPermDeleteConfirm(null)}>
                    <DialogContent className="max-w-[400px]">
                        <DialogHeader>
                            <div className="h-12 w-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4 mx-auto sm:mx-0">
                                <Trash2 className="h-6 w-6 text-red-500" />
                            </div>
                            <DialogTitle className="text-xl">Delete Permission?</DialogTitle>
                            <DialogDescription className="text-slate-400">
                                Are you sure you want to delete <span className="text-white font-bold">{permDeleteConfirm?.permission_string}</span>?
                                This action cannot be undone and may cause system errors if this permission is still in use.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter className="mt-4 gap-2 sm:gap-0">
                            <Button
                                variant="ghost"
                                onClick={() => setPermDeleteConfirm(null)}
                                className="text-slate-400 hover:text-white hover:bg-white/5"
                            >
                                No, Keep it
                            </Button>
                            <Button
                                variant="destructive"
                                onClick={() => void handleDeletePermission()}
                                className="bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/20"
                            >
                                Yes, Delete Permission
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>
        </DndContext>
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

function CreatePermissionSheet({ open, onOpenChange, onSuccess }: { open: boolean, onOpenChange: (o: boolean) => void, onSuccess: () => void }) {
    const [formData, setFormData] = useState<CreatePermissionRequest>({ permission: '', name: '', description: '', platform: '', category: '' });
    const [submitting, setSubmitting] = useState(false);
    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault(); setSubmitting(true);
        try {
            const res = await createPermissionAction(formData);
            if (res.success) {
                toast.success('Created');
                setFormData({ permission: '', name: '', description: '', platform: '', category: '' });
                onSuccess();
                onOpenChange(false);
            } else {
                toast.error(res.error);
            }
        } catch (err: unknown) {
            toast.error(err instanceof Error ? err.message : 'Failed to create permission');
        } finally {
            setSubmitting(false);
        }
    };
    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetTrigger asChild>
                <Button size="icon" className="h-9 w-9 bg-emerald-500 hover:bg-emerald-600">
                    <Plus className="h-4 w-4" />
                </Button>
            </SheetTrigger>
            <SheetContent side="right" className="w-[400px] sm:w-[540px] bg-slate-900 border-white/5 text-white flex flex-col h-full">
                <SheetHeader>
                    <SheetTitle>Create Permission</SheetTitle>
                    <SheetDescription>Define a new system permission.</SheetDescription>
                </SheetHeader>
                <form onSubmit={(e) => void handleSubmit(e)} className="space-y-6 pt-6 flex-1 flex flex-col overflow-y-auto">
                    <div className="space-y-2">
                        <Label>Permission String *</Label>
                        <Input
                            placeholder="e.g. admin:users:delete"
                            required
                            value={formData.permission}
                            onChange={e => setFormData({ ...formData, permission: e.target.value })}
                            className="font-mono bg-white/5"
                        />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2"><Label>Name</Label><Input value={formData.name} onChange={e => setFormData({ ...formData, name: e.target.value })} className="bg-white/5" /></div>
                        <div className="space-y-2"><Label>Category</Label><Input value={formData.category} onChange={e => setFormData({ ...formData, category: e.target.value })} className="bg-white/5" /></div>
                    </div>
                    <div className="space-y-2"><Label>Description</Label><Textarea value={formData.description} onChange={e => setFormData({ ...formData, description: e.target.value })} className="bg-white/5 min-h-[100px]" /></div>
                    <SheetFooter className="mt-auto pt-6">
                        <Button type="submit" disabled={submitting} className="bg-emerald-500 w-full">
                            {submitting ? <Loader2 className="animate-spin w-4 h-4" /> : 'Create permission'}
                        </Button>
                    </SheetFooter>
                </form>
            </SheetContent>
        </Sheet>
    );
}
