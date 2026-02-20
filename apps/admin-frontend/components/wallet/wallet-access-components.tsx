'use client';

import { AlertTriangle, Check, CheckSquare, ChevronLeft, ChevronRight, Key, Loader2, Package, Plus, RefreshCw, Search, Shield, Square, Users, X } from 'lucide-react';
import React from 'react';

import type { AccessItem } from '@/hooks/use-wallet-access';
import type { DragSource, PendingChange } from '@/hooks/use-wallet-access-manager';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';

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

interface ItemCardExpiryProps {
    isAuthorized?: boolean;
    expiresAt?: string | null;
    isPendingAdd: boolean;
    pendingExpiresAt?: string | null;
}

function ItemCardExpiry({ isAuthorized, expiresAt, isPendingAdd, pendingExpiresAt }: ItemCardExpiryProps) {
    if (isAuthorized === true && expiresAt !== null && expiresAt !== undefined) {
        return (
            <p className="text-xs text-amber-600 dark:text-amber-400 font-medium">
                Expires: {new Date(expiresAt).toLocaleDateString()}
            </p>
        );
    }
    if (isPendingAdd === true && pendingExpiresAt !== null && pendingExpiresAt !== undefined) {
        return (
            <p className="text-xs text-green-600 dark:text-green-400 font-medium">
                Will expire: {new Date(pendingExpiresAt).toLocaleDateString()}
            </p>
        );
    }
    return null;
}

interface ItemCardInfoProps {
    item: AccessItem;
    isAuthorized?: boolean;
    isPendingAdd: boolean;
    pendingChange?: PendingChange;
}

function ItemCardInfo({ item, isAuthorized, isPendingAdd, pendingChange }: ItemCardInfoProps) {
    const config = (ITEM_CONFIG[item.type as keyof typeof ITEM_CONFIG] as typeof ITEM_CONFIG.permission | undefined) ?? ITEM_CONFIG.permission;

    return (
        <>
            <span className={cn('flex-shrink-0', config.color)}>{config.icon}</span>
            <div className="flex-1 min-w-0 overflow-hidden">
                <p className="text-sm font-semibold text-gray-900 dark:text-white truncate">
                    {item.name}
                </p>
                {item.description !== undefined && item.description !== '' && (
                    <p className="text-xs text-gray-600 dark:text-gray-400 truncate" title={item.description}>
                        {item.description}
                    </p>
                )}
                {item.type === 'plan' && item.permissionCount !== undefined && (
                    <p className="text-xs text-purple-600 dark:text-purple-400 font-medium">
                        {item.permissionCount} permissions
                    </p>
                )}
                <ItemCardExpiry
                    isAuthorized={isAuthorized}
                    expiresAt={item.expiresAt}
                    isPendingAdd={isPendingAdd}
                    pendingExpiresAt={pendingChange?.expiresAt}
                />
            </div>
        </>
    );
}

interface ItemCardStatusProps {
    isPending: boolean;
    isPendingAdd: boolean;
    isAuthorized?: boolean;
}

function ItemCardStatus({ isPending, isPendingAdd, isAuthorized }: ItemCardStatusProps) {
    if (isPending) {
        return (
            <Badge variant={isPendingAdd ? 'default' : 'destructive'} className="text-xs flex-shrink-0">
                {isPendingAdd ? '+' : '-'}
            </Badge>
        );
    }
    return (
        <span className={cn(
            'flex-shrink-0 p-1 rounded',
            isAuthorized === true
                ? 'text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20'
                : 'text-green-500 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20'
        )}>
            {isAuthorized === true ? <X className="h-4 w-4" /> : <Plus className="h-4 w-4" />}
        </span>
    );
}

interface ItemCardProps {
    item: AccessItem;
    onClick: () => void;
    isAuthorized?: boolean;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    pendingChange?: PendingChange;
    onDragStart?: (e: React.DragEvent) => void;
    onDragEnd?: (e: React.DragEvent) => void;
}

export function ItemCard({ item, onClick, isAuthorized, isSelected, onSelect, pendingChange, onDragStart, onDragEnd }: ItemCardProps) {
    const isPending = pendingChange !== undefined;
    const isPendingAdd = pendingChange?.action === 'add';
    const isPendingRemove = pendingChange?.action === 'remove';

    return (
        <div className="flex items-center gap-2 mb-2">
            <button
                onClick={(e) => {
                    e.stopPropagation();
                    onSelect?.(isSelected !== true);
                }}
                className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded flex-shrink-0"
            >
                {isSelected === true ? (
                    <CheckSquare className="h-4 w-4 text-blue-600" />
                ) : (
                    <Square className="h-4 w-4 text-gray-400" />
                )}
            </button>

            <div
                draggable
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
                onClick={onClick}
                className={cn(
                    'flex-1 flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-all cursor-grab active:cursor-grabbing',
                    'border hover:shadow-sm select-none overflow-hidden',
                    isPendingAdd === true && 'border-dashed border-green-400 bg-green-50/50 dark:bg-green-900/10 opacity-60 blur-[0.5px]',
                    isPendingRemove === true && 'border-dashed border-red-400 bg-red-50/50 dark:bg-red-900/10 opacity-60 blur-[0.5px]',
                    isPending === false && isAuthorized === true
                        ? 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-red-300 dark:hover:border-red-700'
                        : isPending === false ? 'bg-white dark:bg-slate-900/50 border-gray-200 dark:border-gray-800 hover:border-green-300 dark:hover:border-green-700' : ''
                )}
            >
                <ItemCardInfo
                    item={item}
                    isAuthorized={isAuthorized}
                    isPendingAdd={isPendingAdd}
                    pendingChange={pendingChange}
                />
                <ItemCardStatus
                    isPending={isPending}
                    isPendingAdd={isPendingAdd}
                    isAuthorized={isAuthorized}
                />
            </div>
        </div>
    );
}

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
}

export const AvailableColumn = React.forwardRef<HTMLDivElement, AvailableColumnProps>((props, ref) => {
    const allSelected = props.items.length > 0 && props.items.every(item => props.selectedItems.has(item.id));

    return (
        <div
            ref={ref}
            onDragOver={props.onDragOver}
            onDragLeave={props.onDragLeave}
            onDrop={props.onDrop}
            className={cn(
                'bg-gray-50 dark:bg-slate-900/50 rounded-xl border border-gray-200 dark:border-gray-700 transition-all h-[500px] flex flex-col overflow-hidden',
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
});
AvailableColumn.displayName = 'AvailableColumn';

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
}

export const AuthorizedColumn = React.forwardRef<HTMLDivElement, AuthorizedColumnProps>((props, ref) => {
    const allSelected = props.items.length > 0 && props.items.every(item => props.selectedItems.has(item.id));

    return (
        <div
            ref={ref}
            onDragOver={props.onDragOver}
            onDragLeave={props.onDragLeave}
            onDrop={props.onDrop}
            className={cn(
                'bg-white dark:bg-slate-900/50 rounded-xl border border-gray-200 dark:border-gray-700 transition-all h-[500px] flex flex-col overflow-hidden',
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
});
AuthorizedColumn.displayName = 'AuthorizedColumn';

export function WalletAccessInfoBar() {
    return (
        <div className="p-4 border-b border-gray-100 dark:border-gray-800 bg-blue-50/30 dark:bg-blue-900/10">
            <p className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-2">
                <Key className="h-4 w-4 text-blue-500" />
                Need a new permission? <a href="/permissions" className="text-blue-600 dark:text-blue-400 hover:underline font-medium">Go to Permission Registry</a> to define it first.
            </p>
        </div>
    );
}

interface WalletAccessActionBarProps {
    hasChanges: boolean;
    summary: { addPermissions: number; removePermissions: number; addPlans: number; removePlans: number };
    onDiscard: () => void;
    onApply: () => void;
    isApplying: boolean;
}

export function WalletAccessActionBar({ hasChanges, summary, onDiscard, onApply, isApplying }: WalletAccessActionBarProps) {
    if (hasChanges === false) { return null; }

    return (
        <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-800 bg-amber-50/50 dark:bg-amber-900/10 animate-in fade-in slide-in-from-top-1">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4 text-sm">
                    <span className="flex items-center gap-1 text-amber-600 dark:text-amber-400 font-medium">
                        <AlertTriangle className="h-4 w-4" />
                        Pending Changes:
                    </span>
                    {(summary.addPermissions > 0 || summary.addPlans > 0) && (
                        <span className="text-green-600">
                            +{(summary.addPermissions + summary.addPlans)} added
                        </span>
                    )}
                    {(summary.removePermissions > 0 || summary.removePlans > 0) && (
                        <span className="text-red-600">
                            -{(summary.removePermissions + summary.removePlans)} removed
                        </span>
                    )}
                </div>

                <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" onClick={onDiscard} disabled={isApplying} className="h-8">
                        <X className="h-3.5 w-3.5 mr-1" />
                        Discard
                    </Button>
                    <Button
                        size="sm"
                        onClick={onApply}
                        disabled={isApplying}
                        className="bg-green-600 hover:bg-green-700 h-8"
                    >
                        {isApplying === true ? (
                            <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                        ) : (
                            <Check className="h-3.5 w-3.5 mr-1" />
                        )}
                        Apply
                    </Button>
                </div>
            </div>
        </div>
    );
}

interface WalletAccessHeaderProps {
    hasChanges: boolean;
    pendingCount: number;
    isLoading: boolean;
    onRefresh: () => void;
}

export function WalletAccessHeader({ hasChanges, pendingCount, isLoading, onRefresh }: WalletAccessHeaderProps) {
    return (
        <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-800">
            <h3 className="font-semibold text-gray-900 dark:text-white flex items-center gap-2">
                <Shield className="h-5 w-5 text-blue-600" />
                Access Permissions
                {hasChanges && (
                    <Badge variant="outline" className="ml-2 text-amber-600 border-amber-300">
                        {pendingCount} pending
                    </Badge>
                )}
            </h3>
            <Button
                variant="ghost"
                size="sm"
                onClick={onRefresh}
                disabled={isLoading}
            >
                <RefreshCw className={cn('h-4 w-4', isLoading === true && 'animate-spin')} />
            </Button>
        </div>
    );
}

interface WalletAccessErrorProps {
    error: string;
    onRetry: () => void;
}

export function WalletAccessError({ error, onRetry }: WalletAccessErrorProps) {
    return (
        <div className="rounded-xl border border-red-200 dark:border-red-800 p-4">
            <p className="text-red-600 dark:text-red-400">{error}</p>
            <Button variant="outline" onClick={onRetry} className="mt-2">
                Retry
            </Button>
        </div>
    );
}

interface WalletAccessColumnsActionsProps {
    selectedAvailable: Set<string>;
    selectedAuthorized: Set<string>;
    availableItems: AccessItem[];
    authorizedItems: AccessItem[];
    onBulkAssign: (items: AccessItem[]) => void;
    onBulkRemove: (items: AccessItem[]) => void;
    onClearAuthorizedSelection: () => void;
}

export function WalletAccessColumnsActions({
    selectedAvailable,
    selectedAuthorized,
    availableItems,
    authorizedItems,
    onBulkAssign,
    onBulkRemove,
    onClearAuthorizedSelection
}: WalletAccessColumnsActionsProps) {
    return (
        <div className="flex flex-row md:flex-col items-center justify-center gap-2 py-2 md:py-0">
            <Button
                variant="secondary"
                size="icon"
                disabled={selectedAvailable.size === 0}
                onClick={() => {
                    const items = availableItems.filter(g => selectedAvailable.has(g.id));
                    onBulkAssign(items);
                }}
                className={cn(
                    "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                    selectedAvailable.size > 0
                        ? "bg-white dark:bg-gray-700 text-green-600 hover:bg-green-50 hover:text-green-700 hover:shadow-md border-green-200"
                        : "opacity-50"
                )}
            >
                <ChevronRight className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
            </Button>

            <Button
                variant="secondary"
                size="icon"
                disabled={selectedAuthorized.size === 0}
                onClick={() => {
                    const items = authorizedItems.filter(item => selectedAuthorized.has(item.id));
                    onBulkRemove(items);
                    onClearAuthorizedSelection();
                }}
                className={cn(
                    "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                    selectedAuthorized.size > 0
                        ? "bg-white dark:bg-gray-700 text-red-600 hover:bg-red-50 hover:text-red-700 hover:shadow-md border-red-200"
                        : "opacity-50"
                )}
            >
                <ChevronLeft className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
            </Button>
        </div>
    );
}
