'use client';

import { ArrowLeft, ArrowRight } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';

import { Button } from '@/components/ui/button';
import { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { PermissionPane } from './PermissionPane';

interface DualPanePermissionSelectorProps {
    availablePermissions: PermissionDefinition[];
    assignedPermissionStrings: string[];
    onChange: (assignedPermissions: string[]) => void;
    className?: string;
}

export function DualPanePermissionSelector({
    availablePermissions,
    assignedPermissionStrings,
    onChange,
    className,
}: DualPanePermissionSelectorProps) {
    // Selection state for each pane
    const [leftSelected, setLeftSelected] = useState<Set<string>>(new Set());
    const [rightSelected, setRightSelected] = useState<Set<string>>(new Set());

    // Split permissions into available and assigned
    const assignedSet = useMemo(
        () => new Set(assignedPermissionStrings),
        [assignedPermissionStrings]
    );

    const unassignedPermissions = useMemo(
        () => availablePermissions.filter((p) => !assignedSet.has(p.permission_string)),
        [availablePermissions, assignedSet]
    );

    const assignedPermissions = useMemo(
        () => availablePermissions.filter((p) => assignedSet.has(p.permission_string)),
        [availablePermissions, assignedSet]
    );

    // Toggle selection in left pane
    const toggleLeftSelection = useCallback((permString: string) => {
        setLeftSelected((prev) => {
            const next = new Set(prev);
            if (next.has(permString)) {
                next.delete(permString);
            } else {
                next.add(permString);
            }
            return next;
        });
    }, []);

    // Toggle selection in right pane
    const toggleRightSelection = useCallback((permString: string) => {
        setRightSelected((prev) => {
            const next = new Set(prev);
            if (next.has(permString)) {
                next.delete(permString);
            } else {
                next.add(permString);
            }
            return next;
        });
    }, []);

    // Select all in left pane
    const selectAllLeft = useCallback(() => {
        setLeftSelected(new Set(unassignedPermissions.map((p) => p.permission_string)));
    }, [unassignedPermissions]);

    // Select none in left pane
    const selectNoneLeft = useCallback(() => {
        setLeftSelected(new Set());
    }, []);

    // Select all in right pane
    const selectAllRight = useCallback(() => {
        setRightSelected(new Set(assignedPermissions.map((p) => p.permission_string)));
    }, [assignedPermissions]);

    // Select none in right pane
    const selectNoneRight = useCallback(() => {
        setRightSelected(new Set());
    }, []);

    // Assign selected permissions (move from left to right)
    const handleAssign = useCallback(() => {
        if (leftSelected.size === 0) return;

        const newAssigned = [...assignedPermissionStrings, ...Array.from(leftSelected)];
        onChange(newAssigned);
        setLeftSelected(new Set());
    }, [leftSelected, assignedPermissionStrings, onChange]);

    // Remove selected permissions (move from right to left)
    const handleRemove = useCallback(() => {
        if (rightSelected.size === 0) return;

        const newAssigned = assignedPermissionStrings.filter((p) => !rightSelected.has(p));
        onChange(newAssigned);
        setRightSelected(new Set());
    }, [rightSelected, assignedPermissionStrings, onChange]);

    // Keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Ctrl+A - Select all in focused pane
            if (e.ctrlKey && e.key === 'a') {
                e.preventDefault();
                // Determine which pane is focused (simplified)
                selectAllLeft();
            }

            // Ctrl+N - Select none
            if (e.ctrlKey && e.key === 'n') {
                e.preventDefault();
                selectNoneLeft();
                selectNoneRight();
            }

            // Enter - Assign/Remove
            if (e.key === 'Enter') {
                if (leftSelected.size > 0) {
                    handleAssign();
                } else if (rightSelected.size > 0) {
                    handleRemove();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [
        leftSelected,
        rightSelected,
        handleAssign,
        handleRemove,
        selectAllLeft,
        selectNoneLeft,
        selectNoneRight,
    ]);

    return (
        <div className={cn('flex flex-col h-full', className)}>
            {/* Dual Pane Layout */}
            <div className="flex-1 grid grid-cols-2 gap-4 min-h-0">
                {/* Left Pane: Available Permissions */}
                <div className="border border-white/10 rounded-lg bg-slate-900/40 backdrop-blur-xl overflow-hidden flex flex-col">
                    <PermissionPane
                        title="◀ AVAILABLE"
                        permissions={unassignedPermissions}
                        selectedPermissions={leftSelected}
                        onTogglePermission={toggleLeftSelection}
                        onSelectAll={selectAllLeft}
                        onSelectNone={selectNoneLeft}
                        emptyMessage="All permissions assigned"
                    />
                </div>

                {/* Right Pane: Assigned Permissions */}
                <div className="border border-white/10 rounded-lg bg-slate-900/40 backdrop-blur-xl overflow-hidden flex flex-col">
                    <PermissionPane
                        title="ASSIGNED ▶"
                        permissions={assignedPermissions}
                        selectedPermissions={rightSelected}
                        onTogglePermission={toggleRightSelection}
                        onSelectAll={selectAllRight}
                        onSelectNone={selectNoneRight}
                        emptyMessage="No permissions assigned"
                    />
                </div>
            </div>

            {/* Action Buttons */}
            <div className="shrink-0 flex items-center justify-center gap-4 mt-4">
                <Button
                    onClick={handleAssign}
                    disabled={leftSelected.size === 0}
                    className="bg-cyan-500 hover:bg-cyan-600 text-white font-mono text-xs"
                    size="sm"
                >
                    <ArrowRight className="w-4 h-4 mr-2" />
                    Assign Selected ({leftSelected.size})
                </Button>

                <Button
                    onClick={handleRemove}
                    disabled={rightSelected.size === 0}
                    variant="outline"
                    className="border-red-500/30 text-red-400 hover:bg-red-500/10 font-mono text-xs"
                    size="sm"
                >
                    <ArrowLeft className="w-4 h-4 mr-2" />
                    Remove Selected ({rightSelected.size})
                </Button>
            </div>
        </div>
    );
}
