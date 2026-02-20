'use client';

import { ChevronsLeft, ChevronsRight } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { deletePermissionAction } from '@/app/wallet-management/access/permission-actions';
import { Button } from '@/components/ui/button';
import type { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { PermissionPane } from './permission-pane';
import { CreatePermissionSheet } from './permissions/create-permission-sheet';
import { DeletePermissionDialog } from './permissions/delete-permission-dialog';
import { EditPermissionSheet } from './permissions/edit-permission-sheet';

interface DualPanePermissionSelectorProps {
    availablePermissions: PermissionDefinition[];
    assignedPermissionStrings: string[];
    onChange: (assignedPermissions: string[]) => void;
    className?: string;
    onPermissionsChanged?: () => void;
}

export function DualPanePermissionSelector({
    availablePermissions,
    assignedPermissionStrings,
    onChange,
    className,
    onPermissionsChanged,
}: DualPanePermissionSelectorProps) {
    const [leftSelected, setLeftSelected] = useState<Set<string>>(new Set());
    const [rightSelected, setRightSelected] = useState<Set<string>>(new Set());

    const [createOpen, setCreateOpen] = useState(false);
    const [editTarget, setEditTarget] = useState<PermissionDefinition | null>(null);
    const [deleteTarget, setDeleteTarget] = useState<PermissionDefinition | null>(null);

    const assignedSet = useMemo(
        () => new Set(assignedPermissionStrings),
        [assignedPermissionStrings]
    );

    const unassignedPerms = useMemo(
        () => availablePermissions.filter((p) => !assignedSet.has(p.permission_string)),
        [availablePermissions, assignedSet]
    );

    const assignedPerms = useMemo(
        () => availablePermissions.filter((p) => assignedSet.has(p.permission_string)),
        [availablePermissions, assignedSet]
    );

    // Toggle helpers
    const toggleLeft = useCallback((ps: string) => {
        setLeftSelected((prev) => {
            const next = new Set(prev);
            if (next.has(ps)) { next.delete(ps); } else { next.add(ps); }
            return next;
        });
    }, []);

    const toggleRight = useCallback((ps: string) => {
        setRightSelected((prev) => {
            const next = new Set(prev);
            if (next.has(ps)) { next.delete(ps); } else { next.add(ps); }
            return next;
        });
    }, []);

    const selectAllLeft = useCallback(() => {
        setLeftSelected(new Set(unassignedPerms.map((p) => p.permission_string)));
    }, [unassignedPerms]);

    const selectNoneLeft = useCallback(() => setLeftSelected(new Set()), []);

    const selectAllRight = useCallback(() => {
        setRightSelected(new Set(assignedPerms.map((p) => p.permission_string)));
    }, [assignedPerms]);

    const selectNoneRight = useCallback(() => setRightSelected(new Set()), []);

    // Batch assign (left → right)
    const handleAssign = useCallback(() => {
        if (leftSelected.size === 0) return;
        onChange([...assignedPermissionStrings, ...Array.from(leftSelected)]);
        setLeftSelected(new Set());
    }, [leftSelected, assignedPermissionStrings, onChange]);

    // Batch remove (right → left)
    const handleRemove = useCallback(() => {
        if (rightSelected.size === 0) return;
        onChange(assignedPermissionStrings.filter((p) => !rightSelected.has(p)));
        setRightSelected(new Set());
    }, [rightSelected, assignedPermissionStrings, onChange]);

    // Double-click instant transfer
    const transferRight = useCallback((ps: string) => {
        onChange([...assignedPermissionStrings, ps]);
        setLeftSelected((prev) => { const n = new Set(prev); n.delete(ps); return n; });
    }, [assignedPermissionStrings, onChange]);

    const transferLeft = useCallback((ps: string) => {
        onChange(assignedPermissionStrings.filter((p) => p !== ps));
        setRightSelected((prev) => { const n = new Set(prev); n.delete(ps); return n; });
    }, [assignedPermissionStrings, onChange]);

    // Delete permission
    const handleDelete = useCallback(async () => {
        if (!deleteTarget) return;
        try {
            const res = await deletePermissionAction(deleteTarget.id);
            if (res.success) {
                toast.success('Permission deleted');
                onPermissionsChanged?.();
            } else {
                toast.error(res.error ?? 'Failed to delete');
            }
        } catch (err: unknown) {
            toast.error(err instanceof Error ? err.message : 'Failed to delete');
        }
        setDeleteTarget(null);
    }, [deleteTarget, onPermissionsChanged]);

    return (
        <div className={cn('flex flex-col h-full', className)}>
            {/* 3-column: left pane | center actions | right pane */}
            <div className="flex-1 grid grid-cols-[1fr_48px_1fr] gap-0 min-h-0">
                {/* Left: Available */}
                <PermissionPane
                    title="Available"
                    permissions={unassignedPerms}
                    selectedPermissions={leftSelected}
                    onTogglePermission={toggleLeft}
                    onSelectAll={selectAllLeft}
                    onSelectNone={selectNoneLeft}
                    emptyMessage="All permissions assigned"
                    onCreate={onPermissionsChanged ? () => setCreateOpen(true) : undefined}
                    onEdit={onPermissionsChanged ? setEditTarget : undefined}
                    onDelete={onPermissionsChanged ? setDeleteTarget : undefined}
                    onDoubleClickItem={transferRight}
                />

                {/* Center: Action buttons */}
                <div className="flex flex-col items-center justify-center gap-2">
                    <Button
                        size="icon"
                        variant="ghost"
                        onClick={handleAssign}
                        disabled={leftSelected.size === 0}
                        className="h-8 w-8 text-cyan-400 hover:bg-cyan-500/10 disabled:opacity-30"
                        title="Assign selected"
                    >
                        <ChevronsRight className="w-4 h-4" />
                    </Button>

                    {(leftSelected.size > 0 || rightSelected.size > 0) && (
                        <span className="text-[10px] text-muted-foreground tabular-nums">
                            {leftSelected.size || rightSelected.size}
                        </span>
                    )}

                    <Button
                        size="icon"
                        variant="ghost"
                        onClick={handleRemove}
                        disabled={rightSelected.size === 0}
                        className="h-8 w-8 text-red-400 hover:bg-red-500/10 disabled:opacity-30"
                        title="Remove selected"
                    >
                        <ChevronsLeft className="w-4 h-4" />
                    </Button>
                </div>

                {/* Right: Assigned */}
                <PermissionPane
                    title="Assigned"
                    permissions={assignedPerms}
                    selectedPermissions={rightSelected}
                    onTogglePermission={toggleRight}
                    onSelectAll={selectAllRight}
                    onSelectNone={selectNoneRight}
                    emptyMessage="No permissions assigned"
                    onEdit={onPermissionsChanged ? setEditTarget : undefined}
                    onDelete={onPermissionsChanged ? setDeleteTarget : undefined}
                    onDoubleClickItem={transferLeft}
                />
            </div>

            {/* Permission CRUD dialogs */}
            {onPermissionsChanged && (
                <>
                    <CreatePermissionSheet
                        open={createOpen}
                        onOpenChange={setCreateOpen}
                        onSuccess={() => onPermissionsChanged()}
                    />
                    <EditPermissionSheet
                        perm={editTarget}
                        onOpenChange={(o) => { if (!o) setEditTarget(null); }}
                        onSuccess={() => onPermissionsChanged()}
                    />
                    <DeletePermissionDialog
                        permToDelete={deleteTarget}
                        onClose={() => setDeleteTarget(null)}
                        onConfirm={() => void handleDelete()}
                    />
                </>
            )}
        </div>
    );
}
