'use client';

import { useDraggable } from '@dnd-kit/core';
import { Key, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';

import { AddResourceModal } from './add-resource-modal';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { AccessItem } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';
import type { DragItemType } from './types';

interface WalletPermissionSectionProps {
    permissions: AccessItem[];
    availablePermissions: AccessItem[];
    onAddPermission: (permission: AccessItem) => Promise<void>;
    onRemovePermission: (permissionId: string) => Promise<void>;
    isLoading?: boolean;
}

function DraggablePermissionCard({
    permission,
    onRemove
}: {
    permission: AccessItem;
    onRemove: () => void;
}) {
    const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
        id: permission.id,
        data: {
            type: 'permission' as DragItemType,
            name: permission.name,
            id: permission.id
        }
    });

    return (
        <div
            ref={setNodeRef}
            {...listeners}
            {...attributes}
            className={cn(
                "group flex items-center justify-between p-3 rounded-lg border bg-white dark:bg-gray-800 transition-all touch-none",
                isDragging
                    ? "opacity-50 border-blue-400 rotate-2 scale-[1.02] shadow-xl z-50 cursor-grabbing"
                    : "border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-700 hover:shadow-sm cursor-grab"
            )}
        >
            <div className="flex items-center gap-3 overflow-hidden">
                <div className="h-8 w-8 rounded bg-blue-50 dark:bg-blue-900/20 flex items-center justify-center flex-shrink-0">
                    <Key className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                </div>
                <div className="min-w-0">
                    <p className="font-medium text-sm text-gray-900 dark:text-white truncate">
                        {permission.name}
                    </p>
                    {permission.platform && (
                        <p className="text-xs text-gray-500 capitalize">
                            {permission.platform}
                        </p>
                    )}
                </div>
            </div>

            <Button
                variant="ghost"
                size="icon"
                onClick={(e) => {
                    e.stopPropagation(); // Prevent drag start when clicking remove
                    onRemove();
                }}
                onPointerDown={(e) => e.stopPropagation()} // Prevent drag start
                className="h-7 w-7 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
            >
                <Trash2 className="h-3.5 w-3.5" />
            </Button>
        </div>
    );
}

export function WalletPermissionSection({
    permissions,
    availablePermissions,
    onAddPermission,
    onRemovePermission,
    isLoading
}: WalletPermissionSectionProps) {
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [search, setSearch] = useState('');

    const filteredPermissions = permissions.filter(p =>
        p.name.toLowerCase().includes(search.toLowerCase())
    );

    return (
        <div className="space-y-4">
            {/* Header */}
            <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 flex flex-col sm:flex-row items-center justify-between gap-4">
                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <Key className="h-5 w-5 text-gray-500" />
                    <h3 className="font-semibold text-gray-900 dark:text-white">
                        Direct Permissions
                    </h3>
                    <Badge variant="secondary" className="ml-2">
                        {permissions.length}
                    </Badge>
                </div>

                <div className="flex items-center gap-3 w-full sm:w-auto">
                    <Input
                        placeholder="Search permissions..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="h-9 w-full sm:w-48 lg:w-64"
                    />
                    <Button
                        size="sm"
                        className="gap-2 whitespace-nowrap"
                        onClick={() => setIsAddModalOpen(true)}
                    >
                        <Plus className="h-4 w-4" />
                        Add Permission
                    </Button>
                </div>
            </div>

            {/* List */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {filteredPermissions.length === 0 ? (
                    <div className="col-span-full py-12 text-center text-gray-500 border border-dashed border-gray-200 dark:border-gray-700 rounded-xl bg-gray-50/50 dark:bg-gray-900/50">
                        <Key className="h-10 w-10 mx-auto mb-3 opacity-20" />
                        <p>No direct permissions assigned</p>
                    </div>
                ) : (
                    filteredPermissions.map(permission => (
                        <DraggablePermissionCard
                            key={permission.id}
                            permission={permission}
                            onRemove={() => onRemovePermission(permission.id)}
                        />
                    ))
                )}
            </div>

            <AddResourceModal
                isOpen={isAddModalOpen}
                onClose={() => setIsAddModalOpen(false)}
                title="Add permission"
                description="Grant a specific permission directly to this wallet."
                items={availablePermissions}
                onConfirm={onAddPermission}
                isLoading={isLoading}
                emptyMessage="No available permissions found"
            />
        </div>
    );
}
