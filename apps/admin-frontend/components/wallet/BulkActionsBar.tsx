/**
 * Bulk Actions Bar Component
 * Action bar for multi-select wallet operations
 */
'use client';

import { AlertTriangle, Bell, Minus, Plus, X } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface BulkActionsBarProps {
    selectedCount: number;
    onClearSelection: () => void;
    onAddPermission?: () => void;
    onRemovePermission?: () => void;
    onDisable?: () => void;
    onNotify?: () => void;
    className?: string;
}

/**
 *
 * @param root0
 * @param root0.selectedCount
 * @param root0.onClearSelection
 * @param root0.onAddPermission
 * @param root0.onRemovePermission
 * @param root0.onDisable
 * @param root0.onNotify
 * @param root0.className
 */
export function BulkActionsBar({
    selectedCount,
    onClearSelection,
    onAddPermission,
    onRemovePermission,
    onDisable,
    onNotify,
    className,
}: BulkActionsBarProps) {
    if (selectedCount === 0) {return null;}

    return (
        <div className={cn(
            'fixed bottom-6 left-1/2 -translate-x-1/2 z-50',
            'bg-white dark:bg-gray-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-700',
            'px-6 py-4 flex items-center gap-4',
            className
        )}>
            {/* Selection Count */}
            <div className="flex items-center gap-3">
                <button
                    onClick={onClearSelection}
                    className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-500"
                >
                    <X className="h-4 w-4" />
                </button>
                <span className="font-semibold text-gray-900 dark:text-white">
                    {selectedCount} selected
                </span>
            </div>

            <div className="h-6 w-px bg-gray-200 dark:bg-gray-700" />

            {/* Actions */}
            <div className="flex items-center gap-2">
                {onAddPermission && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onAddPermission}
                        className="gap-2"
                    >
                        <Plus className="h-4 w-4" />
                        Add Permission
                    </Button>
                )}

                {onRemovePermission && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onRemovePermission}
                        className="gap-2"
                    >
                        <Minus className="h-4 w-4" />
                        Remove Permission
                    </Button>
                )}

                {onDisable && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onDisable}
                        className="gap-2 text-amber-600 border-amber-300 hover:bg-amber-50 dark:text-amber-400 dark:border-amber-700 dark:hover:bg-amber-900/20"
                    >
                        <AlertTriangle className="h-4 w-4" />
                        Disable
                    </Button>
                )}

                {onNotify && (
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={onNotify}
                        className="gap-2"
                    >
                        <Bell className="h-4 w-4" />
                        Notify
                    </Button>
                )}
            </div>
        </div>
    );
}
