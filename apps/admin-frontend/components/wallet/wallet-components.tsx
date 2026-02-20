'use client';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useDraggable, useDroppable } from '@dnd-kit/core';
import { Key, Package } from 'lucide-react';

// --- Utilities ---
function formatTimeRemaining(expiryDate: string): string {
    const diff = new Date(expiryDate).getTime() - new Date().getTime();
    if (diff <= 0) { return "(Expired)"; }

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    if (days > 0) { return `(${days} days left)`; }

    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours > 0) { return `(${hours} hours left)`; }

    return "(Less than an hour left)";
}

// --- Permissions ---

export function DraggablePermissionItem({ id, label, onRemove }: { id: string; label: string; onRemove?: () => void }) {
    const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
        id,
        data: { type: 'permission', id, name: label }
    });

    return (
        <div
            ref={setNodeRef}
            {...listeners}
            {...attributes}
            className={cn(
                "group flex items-center justify-between gap-3 p-3 rounded-lg border bg-white dark:bg-gray-800 cursor-grab hover:border-purple-400 transition-all select-none",
                isDragging ? "opacity-50 ring-2 ring-purple-500 z-50" : "border-gray-200 dark:border-gray-700 hover:shadow-sm"
            )}
        >
            <div className="flex items-center gap-3 min-w-0">
                <div className="h-8 w-8 rounded bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center flex-shrink-0 text-purple-600 dark:text-purple-400">
                    <Key className="h-4 w-4" />
                </div>
                <span className="text-sm font-medium truncate text-gray-900 dark:text-gray-100" title={label}>{label}</span>
            </div>
            {onRemove && (
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity text-slate-400 hover:text-red-500 hover:bg-red-500/10"
                    onPointerDown={(e) => e.stopPropagation()}
                    onClick={(e) => {
                        e.stopPropagation();
                        onRemove();
                    }}
                >
                    <span className="sr-only">Remove</span>
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18" /><path d="m6 6 12 12" /></svg>
                </Button>
            )}
        </div>
    );
}

export function DroppablePermissionList({
    id,
    items,
    emptyMessage,
    onRemoveItem
}: {
    id: string;
    items: string[];
    emptyMessage: string;
    onRemoveItem?: (item: string) => void;
}) {
    const { setNodeRef, isOver } = useDroppable({ id });

    return (
        <div
            ref={setNodeRef}
            className={cn(
                "min-h-[200px] h-full rounded-xl transition-all duration-200 p-2",
                isOver ? "bg-blue-50/50 dark:bg-blue-900/10 ring-2 ring-blue-400 ring-inset" : "bg-transparent"
            )}
        >
            {items.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-gray-400 border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-xl p-6">
                    <Key className="h-8 w-8 mb-2 opacity-20" />
                    <p className="text-sm text-center">{emptyMessage}</p>
                </div>
            ) : (
                <div className="grid gap-2">
                    {items.map(item => (
                        <DraggablePermissionItem
                            key={item}
                            id={item}
                            label={item}
                            onRemove={onRemoveItem ? () => onRemoveItem(item) : undefined}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}

// --- Plans ---

export function DraggablePlanItem({ id, label, description, isAssigned = false, onManage }: { id: string; label: string; description?: string, isAssigned?: boolean, onManage?: () => void }) {
    const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
        id,
        data: { type: 'plan', id, name: label },
        disabled: isAssigned
    });

    if (isAssigned) {
        return (
            <div className="flex items-center gap-3 p-4 rounded-xl border bg-gray-50 dark:bg-gray-900/50 border-gray-200 dark:border-gray-800 opacity-60 cursor-not-allowed">
                <div className="h-10 w-10 rounded-lg bg-gray-200 dark:bg-gray-800 flex items-center justify-center flex-shrink-0 text-gray-400">
                    <Package className="h-5 w-5" />
                </div>
                <div>
                    <p className="text-sm font-medium text-gray-500">{label}</p>
                    <p className="text-xs text-gray-400">Assigned</p>
                </div>
            </div>
        )
    }

    return (
        <div
            ref={setNodeRef}
            className={cn(
                "group flex items-center gap-4 p-4 rounded-xl border transition-all select-none",
                "bg-gray-100 dark:bg-slate-800/50 border-slate-700 hover:border-blue-500/50 hover:bg-gray-100 dark:bg-slate-800",
                isDragging ? "opacity-50 ring-2 ring-blue-500 z-50 scale-105 shadow-2xl" : "shadow-sm"
            )}
            {...listeners}
            {...attributes}
        >
            <div className="h-10 w-10 rounded-lg bg-blue-500/10 flex items-center justify-center flex-shrink-0 text-blue-400 group-hover:text-blue-300 group-hover:bg-blue-500/20 transition-colors">
                <Package className="h-5 w-5" />
            </div>
            <div className="min-w-0 flex-1">
                <p className="font-semibold text-sm text-slate-200 group-hover:text-white transition-colors">{label}</p>
                {description && <p className="text-xs text-slate-400 truncate mt-0.5">{description}</p>}
            </div>
            {onManage && (
                <Button
                    variant="secondary"
                    size="sm"
                    className="h-7 text-xs opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap"
                    onPointerDown={(e) => e.stopPropagation()}
                    onClick={(e) => {
                        e.stopPropagation();
                        onManage();
                    }}
                >
                    Manage
                </Button>
            )}
        </div>
    );
}

interface PlanItem {
    id: string;
    name: string;
    expiresAt?: string | null;
    isPending?: boolean;
    type?: string;
}

export function DroppablePlanList({
    id,
    items,
    emptyMessage,
    pendingItems = [],
    onEdit,
    onManage,
    onDelete
}: {
    id: string;
    items: PlanItem[];
    emptyMessage: string;
    pendingItems?: PlanItem[];
    onEdit?: (item: PlanItem) => void;
    onManage?: (item: PlanItem) => void;
    onDelete?: (id: string) => void;
}) {
    const { setNodeRef, isOver } = useDroppable({ id });

    const allItems = [...items, ...(pendingItems.map(i => ({ ...i, isPending: true })))];

    return (
        <div
            ref={setNodeRef}
            className={cn(
                "min-h-[500px] h-full rounded-2xl transition-all duration-200 p-4 border-2 border-dashed",
                isOver
                    ? "bg-blue-500/5 border-blue-500/50"
                    : "bg-transparent border-slate-700/50 hover:border-slate-600/50"
            )}
        >
            {allItems.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-slate-500">
                    <Package className="h-10 w-10 mb-3 opacity-20" />
                    <p className="text-sm text-center font-medium">{emptyMessage}</p>
                </div>
            ) : (
                <div className="space-y-3">
                    {allItems.map(plan => (
                        <div key={plan.id} className={cn(
                            "flex items-center justify-between p-4 rounded-xl border bg-slate-800/80 border-slate-700 shadow-sm transition-all",
                            plan.isPending && "border-amber-500/50 bg-amber-500/10"
                        )}>
                            <div className="flex items-center gap-4">
                                <div className={cn(
                                    "h-10 w-10 rounded-lg flex items-center justify-center flex-shrink-0 transition-colors",
                                    plan.isPending ? "bg-amber-500/20 text-amber-400" : "bg-purple-500/20 text-purple-400"
                                )}>
                                    <Package className="h-5 w-5" />
                                </div>
                                <div className="min-w-0">
                                    <p className="font-semibold text-sm text-slate-200">
                                        {plan.name}
                                    </p>
                                    {plan.isPending ? (
                                        <div className="flex flex-col items-start mt-0.5">
                                            <span className="text-[10px] text-amber-400 font-bold uppercase tracking-wider leading-none mb-1">Pending Add</span>
                                            <span className="text-[10px] text-slate-400/80 font-medium leading-none">
                                                {plan.expiresAt ? `Expires: ${new Date(plan.expiresAt).toLocaleDateString()} ${formatTimeRemaining(plan.expiresAt)}` : "Permanent"}
                                            </span>
                                        </div>
                                    ) : (
                                        <p className="text-xs text-slate-400 mt-0.5">
                                            {plan.expiresAt ? `Expires: ${new Date(plan.expiresAt).toLocaleDateString()} ${formatTimeRemaining(plan.expiresAt)}` : "Permanent"}
                                        </p>
                                    )}
                                </div>
                            </div>

                            {/* Actions for Assigned Items */}
                            <div className="flex items-center gap-2">
                                {onManage && !plan.isPending && (
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-8 px-3 text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-700"
                                        onClick={() => onManage(plan)}
                                    >
                                        Manage
                                    </Button>
                                )}
                                {onEdit && (
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-8 px-3 text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-700"
                                        onClick={() => onEdit(plan)}
                                    >
                                        Edit
                                    </Button>
                                )}
                                {onDelete && (
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-8 px-3 text-xs font-medium text-red-500/70 hover:text-red-400 hover:bg-red-500/10"
                                        onClick={() => onDelete(plan.id)}
                                    >
                                        Del
                                    </Button>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
