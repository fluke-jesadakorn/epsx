'use client';

import { useCallback, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';

import { type AccessItem, useWalletAccess } from './use-wallet-access';

export interface PendingChange {
    item: AccessItem;
    action: 'add' | 'remove';
    expiresAt?: string | null;
}

export type DragSource = 'available' | 'authorized';

const EXPIRY_NO_KEY = 'no-expiry';

interface UsePendingChangesContext {
    data: ReturnType<typeof useWalletAccess>['data'];
    batchAssignPermissions: (ids: string[], expiry?: string) => Promise<void>;
    batchRevokePermissions: (ids: string[]) => Promise<void>;
    batchAssignPlans: (ids: string[], expiry?: string) => Promise<void>;
    batchRemovePlans: (ids: string[]) => Promise<void>;
    onSaveComplete?: () => void;
}

export function usePendingChanges(ctx: UsePendingChangesContext) {
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
        if (expiryModalItems === null) { return; }

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
                    } else {
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
                    else { removePlans.push(change.item.id); }
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

export function useSelection() {
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

interface UseSearchAndFilterContext {
    data: ReturnType<typeof useWalletAccess>['data'];
    pendingChanges: Map<string, PendingChange>;
}

export function useSearchAndFilter(ctx: UseSearchAndFilterContext) {
    const [availableSearch, setAvailableSearch] = useState('');
    const [authorizedSearch, setAuthorizedSearch] = useState('');

    const availableItems = useMemo(() => {
        const allGroups = ctx.data.availablePlans;
        let filtered = allGroups;
        if (availableSearch) {
            const lower = availableSearch.toLowerCase();
            filtered = filtered.filter(item =>
                item.name.toLowerCase().includes(lower) ||
                (item.description?.toLowerCase().includes(lower) === true)
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
                (item.description?.toLowerCase().includes(lower) === true)
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

interface UseDragAndDropContext {
    pendingChanges: Map<string, PendingChange>;
    stageRemove: (item: AccessItem) => void;
    setExpiryModalItems: (items: AccessItem[]) => void;
}

export function useDragAndDrop(ctx: UseDragAndDropContext) {
    const [draggedItem, setDraggedItem] = useState<AccessItem | null>(null);
    const [dragSource, setDragSource] = useState<DragSource | null>(null);
    const [dropTarget, setDropTarget] = useState<DragSource | null>(null);
    const availableRef = useRef<HTMLDivElement>(null);
    const authorizedRef = useRef<HTMLDivElement>(null);

    const OPACITY_CLASS = 'opacity-50';
    const handleDragStart = useCallback((e: React.DragEvent, item: AccessItem, source: DragSource) => {
        setDraggedItem(item);
        setDragSource(source);
        e.dataTransfer.setData('text/plain', item.id);
        e.dataTransfer.effectAllowed = 'move';
        const target = e.target as HTMLElement;
        target.classList.add(OPACITY_CLASS);
    }, []);

    const handleDragEnd = useCallback((e: React.DragEvent) => {
        setDraggedItem(null);
        setDragSource(null);
        setDropTarget(null);
        const target = e.target as HTMLElement;
        target.classList.remove(OPACITY_CLASS);
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

        if (draggedItem === null || dragSource === null || dragSource === target) { return; }

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
