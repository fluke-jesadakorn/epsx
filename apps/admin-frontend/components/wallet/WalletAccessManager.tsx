/**
 * Wallet Access Manager Component
 * Unified component for managing wallet permissions and groups
 * Features: Staged changes with Apply button, bulk selection, search/filter
 */

'use client';

import { AlertTriangle, Check, CheckSquare, ChevronLeft, ChevronRight, Key, Loader2, Package, Plus, RefreshCw, Search, Shield, Square, Users, X } from 'lucide-react';
import React, { useCallback, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';

import { type AccessItem, useWalletAccess } from '@/hooks/useWalletAccess';
import { ExpiryDatePicker } from './ExpiryDatePicker';

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
    group: {
        icon: <Users className="h-4 w-4" />,
        color: 'text-purple-600 dark:text-purple-400',
    },
    permission: {
        icon: <Key className="h-4 w-4" />,
        color: 'text-blue-600 dark:text-blue-400',
    }
};

function ItemCard({ item, onClick, isAuthorized, isSelected, onSelect, pendingChange, onDragStart, onDragEnd }: ItemCardProps) {
    const isPending = !!pendingChange;
    const isPendingAdd = pendingChange?.action === 'add';
    const isPendingRemove = pendingChange?.action === 'remove';
    const config = ITEM_CONFIG[item.type as keyof typeof ITEM_CONFIG] || ITEM_CONFIG.permission;

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
                    {item.description && (
                        <p className="text-xs text-gray-600 dark:text-gray-400 truncate" title={item.description}>
                            {item.description}
                        </p>
                    )}
                    {/* Extra info based on type */}
                    {item.type === 'group' && item.permissionCount !== undefined && (
                        <p className="text-xs text-purple-600 dark:text-purple-400 font-medium">
                            {item.permissionCount} permissions
                        </p>
                    )}
                    {isAuthorized && item.expiresAt && (
                        <p className="text-xs text-amber-600 dark:text-amber-400 font-medium">
                            Expires: {new Date(item.expiresAt).toLocaleDateString()}
                        </p>
                    )}
                    {/* Show pending expiry date */}
                    {isPendingAdd && pendingChange?.expiresAt && (
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
                        isAuthorized
                            ? 'text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20'
                            : 'text-green-500 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20'
                    )}>
                        {isAuthorized ? <X className="h-4 w-4" /> : <Plus className="h-4 w-4" />}
                    </span>
                )}
            </div>
        </div>
    );
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function WalletAccessManager({
    walletAddress,
    className,
    onSaveComplete,
}: WalletAccessManagerProps) {
    // UI State
    const [availableSearch, setAvailableSearch] = useState('');
    const [authorizedSearch, setAuthorizedSearch] = useState('');

    // Selection State
    const [selectedAvailable, setSelectedAvailable] = useState<Set<string>>(new Set());
    const [selectedAuthorized, setSelectedAuthorized] = useState<Set<string>>(new Set());

    // Staged Changes State
    const [pendingChanges, setPendingChanges] = useState<Map<string, PendingChange>>(new Map());
    const [isApplying, setIsApplying] = useState(false);

    // Drag-and-Drop State
    const [draggedItem, setDraggedItem] = useState<AccessItem | null>(null);
    const [dragSource, setDragSource] = useState<DragSource | null>(null);
    const [dropTarget, setDropTarget] = useState<DragSource | null>(null);
    const availableRef = useRef<HTMLDivElement>(null);
    const authorizedRef = useRef<HTMLDivElement>(null);

    // Expiry Date Modal State
    const [expiryModalItems, setExpiryModalItems] = useState<AccessItem[] | null>(null);

    // Hook
    const {
        data,
        isLoading,
        error,
        batchAssignPermissions,
        batchRevokePermissions,
        batchAssignGroups,
        batchRemoveGroups,
        refresh,
    } = useWalletAccess(walletAddress);

    // Expiry Modal Handlers
    const handleExpiryConfirm = useCallback((expiresAt: Date | null) => {
        if (expiryModalItems) {
            setPendingChanges(prev => {
                const next = new Map(prev);
                expiryModalItems.forEach(item => {
                    next.set(item.id, { item, action: 'add', expiresAt: expiresAt?.toISOString() || null });
                });
                return next;
            });
            // Clear selection if we came from bulk
            if (expiryModalItems.length > 1) {
                setSelectedAvailable(new Set());
            }
        }
        setExpiryModalItems(null);
    }, [expiryModalItems]);

    const handleExpiryCancel = useCallback(() => {
        setExpiryModalItems(null);
    }, []);

    // Staging Handlers
    const stageAssign = useCallback((item: AccessItem) => {
        setExpiryModalItems([item]);
    }, []);

    const stageRemove = useCallback((item: AccessItem) => {
        setPendingChanges(prev => {
            const next = new Map(prev);
            if (next.has(item.id)) {
                // Toggle off if already pending
                next.delete(item.id);
            } else {
                next.set(item.id, { item, action: 'remove' });
            }
            return next;
        });
    }, []);

    // Drag-and-Drop Handlers
    const handleDragStart = useCallback((e: React.DragEvent, item: AccessItem, source: DragSource) => {
        setDraggedItem(item);
        setDragSource(source);
        e.dataTransfer.setData('text/plain', item.id);
        e.dataTransfer.effectAllowed = 'move';
        // Add visual feedback
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

        if (!draggedItem || !dragSource || dragSource === target) return;

        if (target === 'authorized') {
            // Moving to authorized -> show expiry picker
            setExpiryModalItems([draggedItem]);
        } else {
            // Moving to available (revoking)
            stageRemove(draggedItem);
        }

        setDraggedItem(null);
        setDragSource(null);
    }, [draggedItem, dragSource, stageRemove]);

    // Bulk Actions
    const stageBulkAssign = useCallback(() => {
        const items = data.availableGroups.filter(g => selectedAvailable.has(g.id));
        if (items.length > 0) {
            setExpiryModalItems(items);
        }
    }, [data, selectedAvailable]);

    const stageBulkRemove = useCallback(() => {
        const items = [
            ...data.authorizedPermissions.filter(p => selectedAuthorized.has(p.id)),
            ...data.authorizedGroups.filter(g => selectedAuthorized.has(g.id)),
        ];
        setPendingChanges(prev => {
            const next = new Map(prev);
            items.forEach(item => {
                next.set(item.id, { item, action: 'remove' });
            });
            return next;
        });
        setSelectedAuthorized(new Set());
    }, [data, selectedAuthorized]);

    // Discard all pending changes
    const discardChanges = useCallback(() => {
        setPendingChanges(new Map());
        setSelectedAvailable(new Set());
        setSelectedAuthorized(new Set());
    }, []);

    // Apply all pending changes
    const applyChanges = useCallback(async () => {
        if (pendingChanges.size === 0) return;

        setIsApplying(true);
        try {
            const addPermissions: string[] = [];
            const removePermissions: string[] = [];
            const addGroups: string[] = [];
            const removeGroups: string[] = [];

            pendingChanges.forEach((change) => {
                if (change.action === 'add') {
                    if (change.item.type === 'permission') {
                        addPermissions.push(change.item.id);
                    } else if (change.item.type === 'group') {
                        addGroups.push(change.item.id);
                    }
                } else {
                    if (change.item.type === 'permission') {
                        removePermissions.push(change.item.id);
                    } else if (change.item.type === 'group') {
                        removeGroups.push(change.item.id);
                    }
                }
            });

            // Group additions by expiry date
            const permissionsByExpiry = new Map<string, string[]>();
            const groupsByExpiry = new Map<string, string[]>();

            pendingChanges.forEach((change) => {
                if (change.action === 'add') {
                    const expiryKey = change.expiresAt || 'no-expiry';
                    if (change.item.type === 'permission') {
                        const arr = permissionsByExpiry.get(expiryKey) || [];
                        arr.push(change.item.id);
                        permissionsByExpiry.set(expiryKey, arr);
                    } else if (change.item.type === 'group') {
                        const arr = groupsByExpiry.get(expiryKey) || [];
                        arr.push(change.item.id);
                        groupsByExpiry.set(expiryKey, arr);
                    }
                }
            });

            // Execute batch operations with expiry dates
            const operations: Promise<void>[] = [];

            // Add permissions
            permissionsByExpiry.forEach((ids, expiryKey) => {
                const expiry = expiryKey === 'no-expiry' ? undefined : expiryKey;
                operations.push(batchAssignPermissions(ids, expiry));
            });

            // Add groups
            groupsByExpiry.forEach((ids, expiryKey) => {
                const expiry = expiryKey === 'no-expiry' ? undefined : expiryKey;
                operations.push(batchAssignGroups(ids, expiry));
            });

            // Remove operations
            if (removePermissions.length > 0) operations.push(batchRevokePermissions(removePermissions));
            if (removeGroups.length > 0) operations.push(batchRemoveGroups(removeGroups));

            await Promise.all(operations);

            toast.success(`Applied ${pendingChanges.size} changes successfully`);
            setPendingChanges(new Map());
            onSaveComplete?.();
        } catch (err) {
            toast.error(err instanceof Error ? err.message : 'Failed to apply changes');
        } finally {
            setIsApplying(false);
        }
    }, [pendingChanges, batchAssignPermissions, batchRevokePermissions, batchAssignGroups, batchRemoveGroups, onSaveComplete]);

    // Selection handlers
    const handleSelectAvailable = useCallback((item: AccessItem, selected: boolean) => {
        setSelectedAvailable(prev => {
            const next = new Set(prev);
            if (selected) next.add(item.id);
            else next.delete(item.id);
            return next;
        });
    }, []);

    const handleSelectAuthorized = useCallback((item: AccessItem, selected: boolean) => {
        setSelectedAuthorized(prev => {
            const next = new Set(prev);
            if (selected) next.add(item.id);
            else next.delete(item.id);
            return next;
        });
    }, []);

    const handleSelectAllAvailable = useCallback((items: AccessItem[], selected: boolean) => {
        setSelectedAvailable(prev => {
            const next = new Set(prev);
            items.forEach(item => {
                if (selected) next.add(item.id);
                else next.delete(item.id);
            });
            return next;
        });
    }, []);

    const handleSelectAllAuthorized = useCallback((items: AccessItem[], selected: boolean) => {
        setSelectedAuthorized(prev => {
            const next = new Set(prev);
            items.forEach(item => {
                if (selected) next.add(item.id);
                else next.delete(item.id);
            });
            return next;
        });
    }, []);

    // Filter available items (Groups Only by default request)
    const availableItems = useMemo(() => {
        const allGroups = data.availableGroups;

        // Filter by search
        let filtered = allGroups;
        if (availableSearch) {
            const lower = availableSearch.toLowerCase();
            filtered = filtered.filter(item =>
                item.name.toLowerCase().includes(lower) ||
                item.description?.toLowerCase().includes(lower)
            );
        }

        // Apply pending changes logic (Optimistic UI)
        return [
            ...filtered.filter(item => {
                // If item has a pending ADD, remove it from available list
                const pending = pendingChanges.get(item.id);
                return !pending || pending.action !== 'add';
            }),
            // If item has a pending REMOVE, add it back to available list
            ...Array.from(pendingChanges.values())
                .filter(p => p.action === 'remove' && p.item.type === 'group')
                .map(p => p.item)
        ].sort((a, b) => a.name.localeCompare(b.name));
    }, [data.availableGroups, availableSearch, pendingChanges]);

    // Filter authorized items (Both Groups and Permissions)
    const authorizedItems = useMemo(() => {
        const allItems = [...data.authorizedGroups, ...data.authorizedPermissions];

        // Filter by search
        let filtered = allItems;
        if (authorizedSearch) {
            const lower = authorizedSearch.toLowerCase();
            filtered = filtered.filter(item =>
                item.name.toLowerCase().includes(lower) ||
                item.description?.toLowerCase().includes(lower)
            );
        }

        // Apply pending changes logic (Optimistic UI)
        return [
            ...filtered.filter(item => {
                // If item has a pending REMOVE, remove it from authorized list
                const pending = pendingChanges.get(item.id);
                return !pending || pending.action !== 'remove';
            }),
            // If item has a pending ADD, add it to authorized list
            ...Array.from(pendingChanges.values())
                .filter(p => p.action === 'add')
                .map(p => p.item)
        ].sort((a, b) => {
            // Sort by type then name
            if (a.type !== b.type) return a.type.localeCompare(b.type);
            return a.name.localeCompare(b.name);
        });
    }, [data.authorizedGroups, data.authorizedPermissions, authorizedSearch, pendingChanges]);

    // Count changes
    const changesSummary = useMemo(() => {
        let addPermissions = 0, removePermissions = 0, addGroups = 0, removeGroups = 0;
        pendingChanges.forEach(change => {
            if (change.action === 'add') {
                if (change.item.type === 'permission') addPermissions++;
                else addGroups++;
            } else {
                if (change.item.type === 'permission') removePermissions++;
                else removeGroups++;
            }
        });
        return { addPermissions, removePermissions, addGroups, removeGroups };
    }, [pendingChanges]);

    const hasChanges = pendingChanges.size > 0;
    const allAvailableSelected = availableItems.length > 0 && availableItems.every(item => selectedAvailable.has(item.id));
    const allAuthorizedSelected = authorizedItems.length > 0 && authorizedItems.every(item => selectedAuthorized.has(item.id));

    if (error) {
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
                                {pendingChanges.size} pending
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
                                {(changesSummary.addPermissions > 0 || changesSummary.addGroups > 0) && (
                                    <span className="text-green-600">
                                        +{(changesSummary.addPermissions + changesSummary.addGroups)} added
                                    </span>
                                )}
                                {(changesSummary.removePermissions > 0 || changesSummary.removeGroups > 0) && (
                                    <span className="text-red-600">
                                        -{(changesSummary.removePermissions + changesSummary.removeGroups)} removed
                                    </span>
                                )}
                            </div>

                            <div className="flex items-center gap-2">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={discardChanges}
                                    disabled={isApplying}
                                    className="h-8"
                                >
                                    <X className="h-3.5 w-3.5 mr-1" />
                                    Discard
                                </Button>
                                <Button
                                    size="sm"
                                    onClick={applyChanges}
                                    disabled={isApplying}
                                    className="bg-green-600 hover:bg-green-700 h-8"
                                >
                                    {isApplying ? (
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

                {/* Three-Column Layout (Central Actions) */}
                <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-4 p-4 bg-gray-100 dark:bg-gray-800">

                    {/* AVAILABLE COLUMN (GROUPS ONLY) */}
                    <div
                        ref={availableRef}
                        onDragOver={(e) => handleDragOver(e, 'available')}
                        onDragLeave={handleDragLeave}
                        onDrop={(e) => handleDrop(e, 'available')}
                        className={cn(
                            'bg-gray-50 dark:bg-gray-900/50 rounded-xl border border-gray-200 dark:border-gray-700 transition-all h-[500px] flex flex-col overflow-hidden',
                            dropTarget === 'available' && dragSource === 'authorized' && 'ring-2 ring-inset ring-green-500/50 bg-green-50/20 dark:bg-green-900/10'
                        )}
                    >
                        <div className="flex items-center justify-between mb-3 px-3 pt-3">
                            <h4 className="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                                AVAILABLE GROUPS
                            </h4>
                            <Badge variant="secondary" className="text-xs">
                                {availableItems.length}
                            </Badge>
                        </div>

                        {/* Search & Select All */}
                        <div className="px-3 pb-3 space-y-2">
                            <div className="relative">
                                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                                <Input
                                    value={availableSearch}
                                    onChange={(e) => setAvailableSearch(e.target.value)}
                                    placeholder="Search groups..."
                                    className="pl-9 text-sm h-9"
                                />
                            </div>
                            {availableItems.length > 0 && (
                                <div className="flex items-center">
                                    <button
                                        onClick={() => handleSelectAllAvailable(availableItems, !allAvailableSelected)}
                                        className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-900"
                                    >
                                        {allAvailableSelected ? (
                                            <CheckSquare className="h-3.5 w-3.5 text-blue-600" />
                                        ) : (
                                            <Square className="h-3.5 w-3.5" />
                                        )}
                                        Select All
                                    </button>
                                </div>
                            )}
                        </div>

                        {/* List */}
                        {isLoading ? (
                            <div className="flex items-center justify-center py-8">
                                <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
                            </div>
                        ) : (
                            <div className="flex-1 overflow-y-auto px-3 pb-2">
                                {availableItems.length === 0 ? (
                                    <p className="text-xs text-gray-400 text-center py-4">No groups available</p>
                                ) : (
                                    availableItems.map(item => (
                                        <ItemCard
                                            key={item.id}
                                            item={item}
                                            onClick={() => stageAssign(item)}
                                            isSelected={selectedAvailable.has(item.id)}
                                            onSelect={(selected) => handleSelectAvailable(item, selected)}
                                            pendingChange={pendingChanges.get(item.id)}
                                            onDragStart={(e) => handleDragStart(e, item, 'available')}
                                            onDragEnd={handleDragEnd}
                                        />
                                    ))
                                )}
                            </div>
                        )}
                    </div>

                    {/* CENTRAL ACTIONS */}
                    <div className="flex flex-row md:flex-col items-center justify-center gap-2 py-2 md:py-0">
                        <Button
                            variant="secondary"
                            size="icon"
                            disabled={selectedAvailable.size === 0}
                            onClick={stageBulkAssign}
                            className={cn(
                                "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                                selectedAvailable.size > 0
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
                            disabled={selectedAuthorized.size === 0}
                            onClick={stageBulkRemove}
                            className={cn(
                                "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                                selectedAuthorized.size > 0
                                    ? "bg-white dark:bg-gray-700 text-red-600 hover:bg-red-50 hover:text-red-700 hover:shadow-md border-red-200"
                                    : "opacity-50"
                            )}
                            title="Revoke Selected"
                        >
                            <ChevronLeft className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
                        </Button>
                    </div>

                    {/* AUTHORIZED COLUMN (MIXED) */}
                    <div
                        ref={authorizedRef}
                        onDragOver={(e) => handleDragOver(e, 'authorized')}
                        onDragLeave={handleDragLeave}
                        onDrop={(e) => handleDrop(e, 'authorized')}
                        className={cn(
                            'bg-white dark:bg-gray-900/50 rounded-xl border border-gray-200 dark:border-gray-700 transition-all h-[500px] flex flex-col overflow-hidden',
                            dropTarget === 'authorized' && dragSource === 'available' && 'ring-2 ring-inset ring-blue-500/50 bg-blue-50/20 dark:bg-blue-900/10'
                        )}
                    >
                        <div className="flex items-center justify-between mb-3 px-3 pt-3">
                            <h4 className="text-xs font-bold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                                AUTHORIZED ACCESS
                            </h4>
                            <Badge variant="default" className="text-xs bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                                {authorizedItems.length}
                            </Badge>
                        </div>

                        {/* Search & Select All */}
                        <div className="px-3 pb-3 space-y-2">
                            <div className="relative">
                                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                                <Input
                                    value={authorizedSearch}
                                    onChange={(e) => setAuthorizedSearch(e.target.value)}
                                    placeholder="Search authorized..."
                                    className="pl-9 text-sm h-9"
                                />
                            </div>
                            {authorizedItems.length > 0 && (
                                <div className="flex items-center">
                                    <button
                                        onClick={() => handleSelectAllAuthorized(authorizedItems, !allAuthorizedSelected)}
                                        className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-900"
                                    >
                                        {allAuthorizedSelected ? (
                                            <CheckSquare className="h-3.5 w-3.5 text-blue-600" />
                                        ) : (
                                            <Square className="h-3.5 w-3.5" />
                                        )}
                                        Select All
                                    </button>
                                </div>
                            )}
                        </div>

                        {/* List */}
                        {isLoading ? (
                            <div className="flex items-center justify-center py-8">
                                <Loader2 className="h-6 w-6 animate-spin text-gray-400" />
                            </div>
                        ) : authorizedItems.length === 0 ? (
                            <div className="flex flex-col items-center justify-center py-12 text-gray-400 dark:text-gray-500">
                                <Package className="h-10 w-10 mb-3 opacity-50" />
                                <p className="text-sm">Drag groups here to authorize</p>
                            </div>
                        ) : (
                            <div className="flex-1 overflow-y-auto px-3 pb-2">
                                {authorizedItems.map(item => (
                                    <ItemCard
                                        key={item.id}
                                        item={item}
                                        onClick={() => stageRemove(item)}
                                        isSelected={selectedAuthorized.has(item.id)}
                                        onSelect={(selected) => handleSelectAuthorized(item, selected)}
                                        isAuthorized
                                        pendingChange={pendingChanges.get(item.id)}
                                        onDragStart={(e) => handleDragStart(e, item, 'authorized')}
                                        onDragEnd={handleDragEnd}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Expiry Date Picker Modal */}
            <ExpiryDatePicker
                itemName={expiryModalItems && expiryModalItems.length > 1
                    ? `${expiryModalItems.length} items`
                    : expiryModalItems?.[0]?.name || ''}
                itemType={expiryModalItems && expiryModalItems.length > 1
                    ? 'items'
                    : (expiryModalItems?.[0]?.type || 'permission') as any}
                isOpen={!!expiryModalItems}
                onConfirm={handleExpiryConfirm}
                onCancel={handleExpiryCancel}
            />
        </>
    );
}

export default WalletAccessManager;
