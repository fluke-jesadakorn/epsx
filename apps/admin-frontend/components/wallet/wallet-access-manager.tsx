/**
 * Wallet Access Manager Component
 * Unified component for managing wallet permissions and groups
 * Features: Staged changes with Apply button, bulk selection, search/filter
 */

'use client';

import { AlertTriangle, Check, CheckSquare, ChevronLeft, ChevronRight, Key, Loader2, Package, Plus, RefreshCw, Search, Shield, Square, Users, X } from 'lucide-react';
import React, { useCallback, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

import { ExpiryDatePicker } from './expiry-date-picker';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { type AccessItem, useWalletAccess } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';

// ============================================================================
// TYPES
// ============================================================================

interface WalletAccessManagerProps {
    walletAddress: string;
    className?: string;
    onSaveComplete?: () => void;
}

interface PendingChange {
    item: AccessItem;
    action: 'add' | 'remove';
    expiresAt?: string | null;
}

type DragSource = 'available' | 'authorized';

// ============================================================================
// PENDING CHANGES HOOK
// ============================================================================

interface UsePendingChangesContext {
    data: ReturnType<typeof useWalletAccess>['data'];
    batchAssignPermissions: (ids: string[], expiry?: string) => Promise<void>;
    batchRevokePermissions: (ids: string[]) => Promise<void>;
    batchAssignPlans: (ids: string[], expiry?: string) => Promise<void>;
    batchRemovePlans: (ids: string[]) => Promise<void>;
    onSaveComplete?: () => void;
}

const EXPIRY_NO_KEY = 'no-expiry';

function usePendingChanges(ctx: UsePendingChangesContext) {
    const [pendingChanges, setPendingChanges] = useState<Map<string, PendingChange>>(new Map());
    const [isApplying, setIsApplying] = useState(false);
    const [expiryModalItems, setExpiryModalItems] = useState<AccessItem[] | null>(null);

    const changesSummary = useMemo(() => {
        let addPermissions = 0, removePermissions = 0, addPlans = 0, removePlans = 0;
        pendingChanges.forEach(change => {
            if (change.action === 'add') {
                if (change.item.type === 'permission') { addPermissions++; }
                else { addPlans++; }
            } else if (change.item.type === 'permission') { removePermissions++; }
            else { removePlans++; }
        });
        return { addPermissions, removePermissions, addPlans, removePlans };
    }, [pendingChanges]);

    const handleExpiryConfirm = useCallback((expiresAt: Date | null) => {
        if (!expiryModalItems) { return; }

        setPendingChanges(prev => {
            const next = new Map(prev);
            expiryModalItems.forEach(item => {
                next.set(item.id, { item, action: 'add', expiresAt: expiresAt?.toISOString() ?? null });
            });
            return next;
        });
        setExpiryModalItems(null);
    }, [expiryModalItems]);

    const handleExpiryCancel = useCallback(() => {
        setExpiryModalItems(null);
    }, []);

    const stageAssign = useCallback((item: AccessItem) => {
        setExpiryModalItems([item]);
    }, []);

    const stageRemove = useCallback((item: AccessItem) => {
        setPendingChanges(prev => {
            const next = new Map(prev);
            if (next.has(item.id)) {
                next.delete(item.id);
            } else {
                next.set(item.id, { item, action: 'remove' });
            }
            return next;
        });
    }, []);

    const stageBulkAssign = useCallback((items: AccessItem[]) => {
        if (items.length > 0) {
            setExpiryModalItems(items);
        }
    }, []);

    const stageBulkRemove = useCallback((items: AccessItem[]) => {
        setPendingChanges(prev => {
            const next = new Map(prev);
            items.forEach(item => {
                next.set(item.id, { item, action: 'remove' });
            });
            return next;
        });
    }, []);

    const discardChanges = useCallback(() => {
        setPendingChanges(new Map());
    }, []);

    const applyChanges = useCallback(async () => {
        if (pendingChanges.size === 0) { return; }

        setIsApplying(true);
        try {
            const permissionsByExpiry = new Map<string, string[]>();
            const plansByExpiry = new Map<string, string[]>();

            pendingChanges.forEach((change) => {
                if (change.action === 'add') {
                    const expiryKey = change.expiresAt ?? EXPIRY_NO_KEY;
                    if (change.item.type === 'permission') {
                        const arr = permissionsByExpiry.get(expiryKey) ?? [];
                        arr.push(change.item.id);
                        permissionsByExpiry.set(expiryKey, arr);
                    } else if (change.item.type === 'plan') {
                        const arr = plansByExpiry.get(expiryKey) ?? [];
                        arr.push(change.item.id);
                        plansByExpiry.set(expiryKey, arr);
                    }
                }
            });

            const removePermissions: string[] = [];
            const removePlans: string[] = [];
            pendingChanges.forEach((change) => {
                if (change.action === 'remove') {
                    if (change.item.type === 'permission') { removePermissions.push(change.item.id); }
                    else if (change.item.type === 'plan') { removePlans.push(change.item.id); }
                }
            });

            const operations: Promise<void>[] = [];

            permissionsByExpiry.forEach((ids, expiryKey) => {
                const expiry = expiryKey === EXPIRY_NO_KEY ? undefined : expiryKey;
                operations.push(ctx.batchAssignPermissions(ids, expiry));
            });

            plansByExpiry.forEach((ids, expiryKey) => {
                const expiry = expiryKey === EXPIRY_NO_KEY ? undefined : expiryKey;
                operations.push(ctx.batchAssignPlans(ids, expiry));
            });

            if (removePermissions.length > 0) { operations.push(ctx.batchRevokePermissions(removePermissions)); }
            if (removePlans.length > 0) { operations.push(ctx.batchRemovePlans(removePlans)); }

            await Promise.all(operations);

            toast.success(`Applied ${pendingChanges.size} changes successfully`);
            setPendingChanges(new Map());
            ctx.onSaveComplete?.();
        } catch (err) {
            toast.error(err instanceof Error ? err.message : 'Failed to apply changes');
        } finally {
            setIsApplying(false);
        }
    }, [pendingChanges, ctx]);

    return {
        pendingChanges, setPendingChanges, isApplying, expiryModalItems, setExpiryModalItems,
        changesSummary, stageAssign, stageRemove, stageBulkAssign, stageBulkRemove,
        discardChanges, applyChanges, handleExpiryConfirm, handleExpiryCancel
    };
}

// ============================================================================
// SELECTION HOOK
// ============================================================================

function useSelection() {
    const [selectedAvailable, setSelectedAvailable] = useState<Set<string>>(new Set());
    const [selectedAuthorized, setSelectedAuthorized] = useState<Set<string>>(new Set());

    const handleSelectAvailable = useCallback((item: AccessItem, selected: boolean) => {
        setSelectedAvailable(prev => {
            const next = new Set(prev);
            if (selected) { next.add(item.id); }
            else { next.delete(item.id); }
            return next;
        });
    }, []);

    const handleSelectAuthorized = useCallback((item: AccessItem, selected: boolean) => {
        setSelectedAuthorized(prev => {
            const next = new Set(prev);
            if (selected) { next.add(item.id); }
            else { next.delete(item.id); }
            return next;
        });
    }, []);

    const handleSelectAllAvailable = useCallback((items: AccessItem[], selected: boolean) => {
        setSelectedAvailable(prev => {
            const next = new Set(prev);
            items.forEach(item => {
                if (selected) { next.add(item.id); }
                else { next.delete(item.id); }
            });
            return next;
        });
    }, []);

    const handleSelectAllAuthorized = useCallback((items: AccessItem[], selected: boolean) => {
        setSelectedAuthorized(prev => {
            const next = new Set(prev);
            items.forEach(item => {
                if (selected) { next.add(item.id); }
                else { next.delete(item.id); }
            });
            return next;
        });
    }, []);

    const clearSelections = useCallback(() => {
        setSelectedAvailable(new Set());
        setSelectedAuthorized(new Set());
    }, []);

    return {
        selectedAvailable, setSelectedAvailable, selectedAuthorized, setSelectedAuthorized,
        handleSelectAvailable, handleSelectAuthorized, handleSelectAllAvailable, handleSelectAllAuthorized,
        clearSelections
    };
}

// ============================================================================
// SEARCH & FILTER HOOK
// ============================================================================

interface UseSearchAndFilterContext {
    data: ReturnType<typeof useWalletAccess>['data'];
    pendingChanges: Map<string, PendingChange>;
}

function useSearchAndFilter(ctx: UseSearchAndFilterContext) {
    const [availableSearch, setAvailableSearch] = useState('');
    const [authorizedSearch, setAuthorizedSearch] = useState('');

    const availableItems = useMemo(() => {
        const allGroups = ctx.data.availablePlans;
        let filtered = allGroups;
        if (availableSearch) {
            const lower = availableSearch.toLowerCase();
            filtered = filtered.filter(item =>
                item.name.toLowerCase().includes(lower) ||
                item.description?.toLowerCase().includes(lower)
            );
        }

        return [
            ...filtered.filter(item => {
                const pending = ctx.pendingChanges.get(item.id);
                return pending?.action !== 'add';
            }),
            ...Array.from(ctx.pendingChanges.values())
                .filter(p => p.action === 'remove' && p.item.type === 'plan')
                .map(p => p.item)
        ].sort((a, b) => a.name.localeCompare(b.name));
    }, [ctx.data.availablePlans, availableSearch, ctx.pendingChanges]);

    const authorizedItems = useMemo(() => {
        const allItems = [...ctx.data.authorizedPlans, ...ctx.data.authorizedPermissions];
        let filtered = allItems;
        if (authorizedSearch) {
            const lower = authorizedSearch.toLowerCase();
            filtered = filtered.filter(item =>
                item.name.toLowerCase().includes(lower) ||
                item.description?.toLowerCase().includes(lower)
            );
        }

        return [
            ...filtered.filter(item => {
                const pending = ctx.pendingChanges.get(item.id);
                return pending?.action !== 'remove';
            }),
            ...Array.from(ctx.pendingChanges.values())
                .filter(p => p.action === 'add')
                .map(p => p.item)
        ].sort((a, b) => {
            if (a.type !== b.type) { return a.type.localeCompare(b.type); }
            return a.name.localeCompare(b.name);
        });
    }, [ctx.data.authorizedPlans, ctx.data.authorizedPermissions, authorizedSearch, ctx.pendingChanges]);

    return { availableSearch, setAvailableSearch, authorizedSearch, setAuthorizedSearch, availableItems, authorizedItems };
}

// ============================================================================
// DRAG AND DROP HOOK
// ============================================================================

interface UseDragAndDropContext {
    pendingChanges: Map<string, PendingChange>;
    stageRemove: (item: AccessItem) => void;
    setExpiryModalItems: (items: AccessItem[]) => void;
}

function useDragAndDrop(ctx: UseDragAndDropContext) {
    const [draggedItem, setDraggedItem] = useState<AccessItem | null>(null);
    const [dragSource, setDragSource] = useState<DragSource | null>(null);
    const [dropTarget, setDropTarget] = useState<DragSource | null>(null);
    const availableRef = useRef<HTMLDivElement>(null);
    const authorizedRef = useRef<HTMLDivElement>(null);

    const handleDragStart = useCallback((e: React.DragEvent, item: AccessItem, source: DragSource) => {
        setDraggedItem(item);
        setDragSource(source);
        e.dataTransfer.setData('text/plain', item.id);
        e.dataTransfer.effectAllowed = 'move';
        const target = e.target as HTMLElement;
        target.classList.add('opacity-50');
    }, []);

    const handleDragEnd = useCallback((e: React.DragEvent) => {
        setDraggedItem(null);
        setDragSource(null);
        setDropTarget(null);
        const target = e.target as HTMLElement;
        target.classList.remove('opacity-50');
    }, []);

    const handleDragOver = useCallback((e: React.DragEvent, target: DragSource) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        setDropTarget(target);
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        if (e.currentTarget.contains(e.relatedTarget as Node)) {
            return;
        }
        setDropTarget(null);
    }, []);

    const handleDrop = useCallback((e: React.DragEvent, target: DragSource) => {
        e.preventDefault();
        setDropTarget(null);

        if (!draggedItem ?? !dragSource ?? dragSource === target) { return; }

        if (target === 'authorized') {
            ctx.setExpiryModalItems([draggedItem]);
        } else {
            ctx.stageRemove(draggedItem);
        }

        setDraggedItem(null);
        setDragSource(null);
    }, [draggedItem, dragSource, ctx]);

    return {
        draggedItem, dragSource, dropTarget, availableRef, authorizedRef,
        handleDragStart, handleDragEnd, handleDragOver, handleDragLeave, handleDrop
    };
}

// ============================================================================
// COLUMN COMPONENTS
// ============================================================================

interface AvailableColumnProps {
    items: AccessItem[];
    selectedItems: Set<string>;
    isLoading: boolean;
    search: string;
    pendingChanges: Map<string, PendingChange>;
    dragState: { dropTarget: DragSource | null; dragSource: DragSource | null };
    onSearchChange: (value: string) => void;
    onSelectAll: (items: AccessItem[], selected: boolean) => void;
    onSelectItem: (item: AccessItem, selected: boolean) => void;
    onDragStart: (e: React.DragEvent, item: AccessItem) => void;
    onDragEnd: (e: React.DragEvent) => void;
    onDragOver: (e: React.DragEvent) => void;
    onDragLeave: (e: React.DragEvent) => void;
    onDrop: (e: React.DragEvent) => void;
    onItemClick: (item: AccessItem) => void;
    ref: React.RefObject<HTMLDivElement>;
}

function AvailableColumn(props: AvailableColumnProps) {
    const allSelected = props.items.length > 0 && props.items.every(item => props.selectedItems.has(item.id));

    return (
        <div
            ref={props.ref}
            onDragOver={props.onDragOver}
            onDragLeave={props.onDragLeave}
            onDrop={props.onDrop}
            className={cn(
                'bg-gray-50 dark:bg-gray-900/50 rounded-xl border border-gray-200 dark:border-gray-700 transition-all h-[500px] flex flex-col overflow-hidden',
                props.dragState.dropTarget === 'available' && props.dragState.dragSource === 'authorized' && 'ring-2 ring-inset ring-green-500/50 bg-green-50/20 dark:bg-green-900/10'
            )}
        >
            <div className="flex items-center justify-between mb-3 px-3 pt-3">
                <h4 className="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                    AVAILABLE PLANS
                </h4>
                <Badge variant="secondary" className="text-xs">
                    {props.items.length}
                </Badge>
            </div>

            <div className="px-3 pb-3 space-y-2">
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                    <Input
                        value={props.search}
                        onChange={(e) => props.onSearchChange(e.target.value)}
                        placeholder="Search groups..."
                        className="pl-9 text-sm h-9"
                    />
                </div>
                {props.items.length > 0 && (
                    <div className="flex items-center">
                        <button
                            onClick={() => props.onSelectAll(props.items, !allSelected)}
                            className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-900"
                        >
                            {allSelected ? (
                                <CheckSquare className="h-3.5 w-3.5 text-blue-600" />
                            ) : (
                                <Square className="h-3.5 w-3.5" />
                            )}
                            Select All
                        </button>
                    </div>
                )}
            </div>

            {props.isLoading ? (
                <div className="flex items-center justify-center py-8">
                    <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
                </div>
            ) : (
                <div className="flex-1 overflow-y-auto px-3 pb-2">
                    {props.items.length === 0 ? (
                        <p className="text-xs text-gray-400 text-center py-4">No groups available</p>
                    ) : (
                        props.items.map(item => (
                            <ItemCard
                                key={item.id}
                                item={item}
                                onClick={() => props.onItemClick(item)}
                                isSelected={props.selectedItems.has(item.id)}
                                onSelect={(selected) => props.onSelectItem(item, selected)}
                                pendingChange={props.pendingChanges.get(item.id)}
                                onDragStart={(e) => props.onDragStart(e, item)}
                                onDragEnd={props.onDragEnd}
                            />
                        ))
                    )}
                </div>
            )}
        </div>
    );
}

interface AuthorizedColumnProps {
    items: AccessItem[];
    selectedItems: Set<string>;
    isLoading: boolean;
    search: string;
    pendingChanges: Map<string, PendingChange>;
    dragState: { dropTarget: DragSource | null; dragSource: DragSource | null };
    onSearchChange: (value: string) => void;
    onSelectAll: (items: AccessItem[], selected: boolean) => void;
    onSelectItem: (item: AccessItem, selected: boolean) => void;
    onDragStart: (e: React.DragEvent, item: AccessItem) => void;
    onDragEnd: (e: React.DragEvent) => void;
    onDragOver: (e: React.DragEvent) => void;
    onDragLeave: (e: React.DragEvent) => void;
    onDrop: (e: React.DragEvent) => void;
    onItemClick: (item: AccessItem) => void;
    ref: React.RefObject<HTMLDivElement>;
}

function AuthorizedColumn(props: AuthorizedColumnProps) {
    const allSelected = props.items.length > 0 && props.items.every(item => props.selectedItems.has(item.id));

    return (
        <div
            ref={props.ref}
            onDragOver={props.onDragOver}
            onDragLeave={props.onDragLeave}
            onDrop={props.onDrop}
            className={cn(
                'bg-white dark:bg-gray-900/50 rounded-xl border border-gray-200 dark:border-gray-700 transition-all h-[500px] flex flex-col overflow-hidden',
                props.dragState.dropTarget === 'authorized' && props.dragState.dragSource === 'available' && 'ring-2 ring-inset ring-blue-500/50 bg-blue-50/20 dark:bg-blue-900/10'
            )}
        >
            <div className="flex items-center justify-between mb-3 px-3 pt-3">
                <h4 className="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                    AUTHORIZED ACCESS
                </h4>
                <Badge variant="default" className="text-xs bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                    {props.items.length}
                </Badge>
            </div>

            <div className="px-3 pb-3 space-y-2">
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                    <Input
                        value={props.search}
                        onChange={(e) => props.onSearchChange(e.target.value)}
                        placeholder="Search authorized..."
                        className="pl-9 text-sm h-9"
                    />
                </div>
                {props.items.length > 0 && (
                    <div className="flex items-center">
                        <button
                            onClick={() => props.onSelectAll(props.items, !allSelected)}
                            className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-900"
                        >
                            {allSelected ? (
                                <CheckSquare className="h-3.5 w-3.5 text-blue-600" />
                            ) : (
                                <Square className="h-3.5 w-3.5" />
                            )}
                            Select All
                        </button>
                    </div>
                )}
            </div>

            {props.isLoading ? (
                <div className="flex items-center justify-center py-8">
                    <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
                </div>
            ) : props.items.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-gray-400 dark:text-gray-500">
                    <Package className="h-10 w-10 mb-3 opacity-50" />
                    <p className="text-sm">Drag groups here to authorize</p>
                </div>
            ) : (
                <div className="flex-1 overflow-y-auto px-3 pb-2">
                    {props.items.map(item => (
                        <ItemCard
                            key={item.id}
                            item={item}
                            onClick={() => props.onItemClick(item)}
                            isSelected={props.selectedItems.has(item.id)}
                            onSelect={(selected) => props.onSelectItem(item, selected)}
                            isAuthorized
                            pendingChange={props.pendingChanges.get(item.id)}
                            onDragStart={(e) => props.onDragStart(e, item)}
                            onDragEnd={props.onDragEnd}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}

// ============================================================================
// ITEM CARD COMPONENT
// ============================================================================

interface ItemCardProps {
    item: AccessItem;
    onClick: () => void;
    isAuthorized?: boolean;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    pendingChange?: PendingChange;
    // Drag-and-drop props
    onDragStart?: (e: React.DragEvent) => void;
    onDragEnd?: (e: React.DragEvent) => void;
}

// Config for icons/colors
const ITEM_CONFIG = {
    permission: {
        icon: <Key className="h-4 w-4" />,
        color: 'text-blue-600 dark:text-blue-400',
    },
    plan: {
        icon: <Users className="h-4 w-4" />,
        color: 'text-purple-600 dark:text-purple-400',
    }
};

function ItemCard({ item, onClick, isAuthorized, isSelected, onSelect, pendingChange, onDragStart, onDragEnd }: ItemCardProps) {
    const isPending = Boolean(pendingChange);
    const isPendingAdd = pendingChange?.action === 'add';
    const isPendingRemove = pendingChange?.action === 'remove';
    const config = ITEM_CONFIG[item.type as keyof typeof ITEM_CONFIG] ?? ITEM_CONFIG.permission;

    return (
        <div className="flex items-center gap-2 mb-2">
            {/* Selection Checkbox */}
            <button
                onClick={(e) => {
                    e.stopPropagation();
                    onSelect?.(!isSelected);
                }}
                className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded flex-shrink-0"
            >
                {isSelected ? (
                    <CheckSquare className="h-4 w-4 text-blue-600" />
                ) : (
                    <Square className="h-4 w-4 text-gray-400" />
                )}
            </button>

            {/* Item Card - Draggable */}
            <div
                draggable
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
                onClick={onClick}
                className={cn(
                    'flex-1 flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-all cursor-grab active:cursor-grabbing',
                    'border hover:shadow-sm select-none overflow-hidden',
                    isPendingAdd && 'border-dashed border-green-400 bg-green-50/50 dark:bg-green-900/10 opacity-60 blur-[0.5px]',
                    isPendingRemove && 'border-dashed border-red-400 bg-red-50/50 dark:bg-red-900/10 opacity-60 blur-[0.5px]',
                    !isPending && isAuthorized
                        ? 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-red-300 dark:hover:border-red-700'
                        : !isPending ? 'bg-white dark:bg-gray-900/50 border-gray-200 dark:border-gray-800 hover:border-green-300 dark:hover:border-green-700' : ''
                )}
            >
                <span className={cn('flex-shrink-0', config.color)}>{config.icon}</span>
                <div className="flex-1 min-w-0 overflow-hidden">
                    <p className="text-sm font-semibold text-gray-900 dark:text-white truncate">
                        {item.name}
                    </p>
                    {Boolean(item.description) && (
                        <p className="text-xs text-gray-600 dark:text-gray-400 truncate" title={item.description}>
                            {item.description}
                        </p>
                    )}
                    {/* Extra info based on type */}
                    {item.type === 'plan' && item.permissionCount !== undefined && (
                        <p className="text-xs text-purple-600 dark:text-purple-400 font-medium">
                            {item.permissionCount} permissions
                        </p>
                    )}
                    {isAuthorized === true && item.expiresAt != null && (
                        <p className="text-xs text-amber-600 dark:text-amber-400 font-medium">
                            Expires: {new Date(item.expiresAt).toLocaleDateString()}
                        </p>
                    )}
                    {/* Show pending expiry date */}
                    {isPendingAdd && pendingChange?.expiresAt != null && (
                        <p className="text-xs text-green-600 dark:text-green-400 font-medium">
                            Will expire: {new Date(pendingChange.expiresAt).toLocaleDateString()}
                        </p>
                    )}
                </div>

                {/* Pending Badge or Status Icon */}
                {isPending ? (
                    <Badge variant={isPendingAdd ? 'default' : 'destructive'} className="text-xs flex-shrink-0">
                        {isPendingAdd ? '+' : '-'}
                    </Badge>
                ) : (
                    <span className={cn(
                        'flex-shrink-0 p-1 rounded',
                        isAuthorized === true
                            ? 'text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20'
                            : 'text-green-500 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20'
                    )}>
                        {isAuthorized === true ? <X className="h-4 w-4" /> : <Plus className="h-4 w-4" />}
                    </span>
                )}
            </div>
        </div>
    );
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

/**
 *
 * @param root0
 * @param root0.walletAddress
 * @param root0.className
 * @param root0.onSaveComplete
 */
export function WalletAccessManager({
    walletAddress,
    className,
    onSaveComplete,
}: WalletAccessManagerProps) {
    const {
        data,
        isLoading,
        error,
        batchAssignPermissions,
        batchRevokePermissions,
        batchAssignPlans,
        batchRemovePlans,
        refresh,
    } = useWalletAccess(walletAddress);

    // Extract hooks
    const pendingCtx = usePendingChanges({
        data, batchAssignPermissions, batchRevokePermissions, batchAssignPlans, batchRemovePlans, onSaveComplete
    });
    const selectionCtx = useSelection();
    const filterCtx = useSearchAndFilter({ data, pendingChanges: pendingCtx.pendingChanges });
    const dragCtx = useDragAndDrop({
        pendingChanges: pendingCtx.pendingChanges,
        stageRemove: pendingCtx.stageRemove,
        setExpiryModalItems: pendingCtx.setExpiryModalItems
    });

    // Handle bulk operations
    const stageBulkAssign = useCallback(() => {
        const items = filterCtx.availableItems.filter(g => selectionCtx.selectedAvailable.has(g.id));
        pendingCtx.stageBulkAssign(items);
    }, [filterCtx.availableItems, selectionCtx.selectedAvailable, pendingCtx]);

    const stageBulkRemove = useCallback(() => {
        const items = filterCtx.authorizedItems.filter(item => selectionCtx.selectedAuthorized.has(item.id));
        pendingCtx.stageBulkRemove(items);
        selectionCtx.setSelectedAuthorized(new Set());
    }, [filterCtx.authorizedItems, selectionCtx.selectedAuthorized, pendingCtx, selectionCtx]);

    const handleDiscard = useCallback(() => {
        pendingCtx.discardChanges();
        selectionCtx.clearSelections();
    }, [pendingCtx, selectionCtx]);

    const hasChanges = pendingCtx.pendingChanges.size > 0;
    const allAvailableSelected = filterCtx.availableItems.length > 0 && filterCtx.availableItems.every(item => selectionCtx.selectedAvailable.has(item.id));
    const allAuthorizedSelected = filterCtx.authorizedItems.length > 0 && filterCtx.authorizedItems.every(item => selectionCtx.selectedAuthorized.has(item.id));

    if (error ?? false) {
        return (
            <div className={cn('rounded-xl border border-red-200 dark:border-red-800 p-4', className)}>
                <p className="text-red-600 dark:text-red-400">{error}</p>
                <Button variant="outline" onClick={refresh} className="mt-2">
                    Retry
                </Button>
            </div>
        );
    }

    return (
        <>
            <div className={cn('rounded-xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700', className)}>
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-800">
                    <h3 className="font-semibold text-gray-900 dark:text-white flex items-center gap-2">
                        <Shield className="h-5 w-5 text-blue-600" />
                        Access Permissions
                        {hasChanges && (
                            <Badge variant="outline" className="ml-2 text-amber-600 border-amber-300">
                                {pendingCtx.pendingChanges.size} pending
                            </Badge>
                        )}
                    </h3>
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={refresh}
                        disabled={isLoading}
                    >
                        <RefreshCw className={cn('h-4 w-4', isLoading && 'animate-spin')} />
                    </Button>
                </div>

                <div className="p-4 border-b border-gray-100 dark:border-gray-800 bg-blue-50/30 dark:bg-blue-900/10">
                    <p className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-2">
                        <Key className="h-4 w-4 text-blue-500" />
                        Need a new permission? <a href="/permissions" className="text-blue-600 dark:text-blue-400 hover:underline font-medium">Go to Permission Registry</a> to define it first.
                    </p>
                </div>

                {/* Action Bar */}
                {hasChanges && (
                    <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-800 bg-amber-50/50 dark:bg-amber-900/10 animate-in fade-in slide-in-from-top-1">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-4 text-sm">
                                <span className="flex items-center gap-1 text-amber-600 dark:text-amber-400 font-medium">
                                    <AlertTriangle className="h-4 w-4" />
                                    Pending Changes:
                                </span>
                                {(pendingCtx.changesSummary.addPermissions > 0 ?? pendingCtx.changesSummary.addPlans > 0) && (
                                    <span className="text-green-600">
                                        +{(pendingCtx.changesSummary.addPermissions + pendingCtx.changesSummary.addPlans)} added
                                    </span>
                                )}
                                {(pendingCtx.changesSummary.removePermissions > 0 ?? pendingCtx.changesSummary.removePlans > 0) && (
                                    <span className="text-red-600">
                                        -{(pendingCtx.changesSummary.removePermissions + pendingCtx.changesSummary.removePlans)} removed
                                    </span>
                                )}
                            </div>

                            <div className="flex items-center gap-2">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={handleDiscard}
                                    disabled={pendingCtx.isApplying}
                                    className="h-8"
                                >
                                    <X className="h-3.5 w-3.5 mr-1" />
                                    Discard
                                </Button>
                                <Button
                                    size="sm"
                                    onClick={pendingCtx.applyChanges}
                                    disabled={pendingCtx.isApplying}
                                    className="bg-green-600 hover:bg-green-700 h-8"
                                >
                                    {pendingCtx.isApplying ? (
                                        <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                                    ) : (
                                        <Check className="h-3.5 w-3.5 mr-1" />
                                    )}
                                    Apply
                                </Button>
                            </div>
                        </div>
                    </div>
                )}

                {/* Three-Column Layout */}
                <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-4 p-4 bg-gray-100 dark:bg-gray-800">
                    <AvailableColumn
                        ref={dragCtx.availableRef}
                        items={filterCtx.availableItems}
                        selectedItems={selectionCtx.selectedAvailable}
                        isLoading={isLoading}
                        search={filterCtx.availableSearch}
                        pendingChanges={pendingCtx.pendingChanges}
                        dragState={{ dropTarget: dragCtx.dropTarget, dragSource: dragCtx.dragSource }}
                        onSearchChange={filterCtx.setAvailableSearch}
                        onSelectAll={selectionCtx.handleSelectAllAvailable}
                        onSelectItem={selectionCtx.handleSelectAvailable}
                        onDragStart={(e, item) => dragCtx.handleDragStart(e, item, 'available')}
                        onDragEnd={dragCtx.handleDragEnd}
                        onDragOver={(e) => dragCtx.handleDragOver(e, 'available')}
                        onDragLeave={dragCtx.handleDragLeave}
                        onDrop={(e) => dragCtx.handleDrop(e, 'available')}
                        onItemClick={pendingCtx.stageAssign}
                    />

                    {/* CENTRAL ACTIONS */}
                    <div className="flex flex-row md:flex-col items-center justify-center gap-2 py-2 md:py-0">
                        <Button
                            variant="secondary"
                            size="icon"
                            disabled={selectionCtx.selectedAvailable.size === 0}
                            onClick={stageBulkAssign}
                            className={cn(
                                "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                                selectionCtx.selectedAvailable.size > 0
                                    ? "bg-white dark:bg-gray-700 text-green-600 hover:bg-green-50 hover:text-green-700 hover:shadow-md border-green-200"
                                    : "opacity-50"
                            )}
                            title="Assign Selected"
                        >
                            <ChevronRight className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
                        </Button>

                        <Button
                            variant="secondary"
                            size="icon"
                            disabled={selectionCtx.selectedAuthorized.size === 0}
                            onClick={stageBulkRemove}
                            className={cn(
                                "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                                selectionCtx.selectedAuthorized.size > 0
                                    ? "bg-white dark:bg-gray-700 text-red-600 hover:bg-red-50 hover:text-red-700 hover:shadow-md border-red-200"
                                    : "opacity-50"
                            )}
                            title="Revoke Selected"
                        >
                            <ChevronLeft className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
                        </Button>
                    </div>

                    <AuthorizedColumn
                        ref={dragCtx.authorizedRef}
                        items={filterCtx.authorizedItems}
                        selectedItems={selectionCtx.selectedAuthorized}
                        isLoading={isLoading}
                        search={filterCtx.authorizedSearch}
                        pendingChanges={pendingCtx.pendingChanges}
                        dragState={{ dropTarget: dragCtx.dropTarget, dragSource: dragCtx.dragSource }}
                        onSearchChange={filterCtx.setAuthorizedSearch}
                        onSelectAll={selectionCtx.handleSelectAllAuthorized}
                        onSelectItem={selectionCtx.handleSelectAuthorized}
                        onDragStart={(e, item) => dragCtx.handleDragStart(e, item, 'authorized')}
                        onDragEnd={dragCtx.handleDragEnd}
                        onDragOver={(e) => dragCtx.handleDragOver(e, 'authorized')}
                        onDragLeave={dragCtx.handleDragLeave}
                        onDrop={(e) => dragCtx.handleDrop(e, 'authorized')}
                        onItemClick={pendingCtx.stageRemove}
                    />
                </div>
            </div>

            {/* Expiry Date Picker Modal */}
            <ExpiryDatePicker
                itemName={pendingCtx.expiryModalItems && pendingCtx.expiryModalItems.length > 1
                    ? `${pendingCtx.expiryModalItems.length} items`
                    : pendingCtx.expiryModalItems?.[0]?.name ?? ''}
                itemType={pendingCtx.expiryModalItems && pendingCtx.expiryModalItems.length > 1
                    ? 'items'
                    : (pendingCtx.expiryModalItems?.[0]?.type ?? 'permission') as 'permission' | 'plan' | 'items'}
                isOpen={Boolean(pendingCtx.expiryModalItems)}
                onConfirm={pendingCtx.handleExpiryConfirm}
                onCancel={pendingCtx.handleExpiryCancel}
            />
        </>
    );
}

export default WalletAccessManager;
