'use client';

import { Search, X } from 'lucide-react';
import { useMemo, useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import {
    filterPermissions,
    sortPermissions,
} from '@/lib/utils/permission-utils';
import { PermissionItem } from './PermissionItem';

interface PermissionPaneProps {
    title: string;
    permissions: PermissionDefinition[];
    selectedPermissions: Set<string>;
    onTogglePermission: (permissionString: string) => void;
    onSelectAll: () => void;
    onSelectNone: () => void;
    emptyMessage?: string;
    className?: string;
}

export function PermissionPane({
    title,
    permissions,
    selectedPermissions,
    onTogglePermission,
    onSelectAll,
    onSelectNone,
    emptyMessage = 'No permissions available',
    className,
}: PermissionPaneProps) {
    const [searchQuery, setSearchQuery] = useState('');

    // Filter and sort permissions
    const filteredPerms = useMemo(
        () => sortPermissions(filterPermissions(permissions, searchQuery)),
        [permissions, searchQuery]
    );

    const selectedCount = permissions.filter((p) =>
        selectedPermissions.has(p.permission_string)
    ).length;

    return (
        <div className={cn('flex flex-col h-full', className)}>
            {/* Header */}
            <div className="shrink-0 p-4 border-b border-white/10 bg-white/5">
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                        <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground ascii-box">
                            {title}
                        </h3>
                        <Badge variant="secondary" className="text-[10px] h-5">
                            {filteredPerms.length}
                        </Badge>
                        {selectedCount > 0 && (
                            <Badge variant="outline" className="text-[10px] h-5 border-cyan-500/30 text-cyan-400">
                                {selectedCount} selected
                            </Badge>
                        )}
                    </div>

                    <div className="flex gap-1">
                        <Button
                            size="sm"
                            variant="ghost"
                            onClick={onSelectAll}
                            className="h-6 px-2 text-[10px]"
                        >
                            All
                        </Button>
                        <Button
                            size="sm"
                            variant="ghost"
                            onClick={onSelectNone}
                            className="h-6 px-2 text-[10px]"
                        >
                            None
                        </Button>
                    </div>
                </div>

                {/* Search Filter */}
                <div className="relative">
                    <Search className="absolute left-2.5 top-2 h-3.5 w-3.5 text-muted-foreground/50" />
                    <Input
                        placeholder="Filter permissions..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="h-8 pl-8 pr-8 text-xs bg-black/20 border-white/10"
                    />
                    {searchQuery && (
                        <button
                            onClick={() => setSearchQuery('')}
                            className="absolute right-2 top-2 text-muted-foreground/50 hover:text-foreground"
                        >
                            <X className="h-3.5 w-3.5" />
                        </button>
                    )}
                </div>
            </div>

            {/* Permission List */}
            <div className="flex-1 overflow-y-auto p-0 scrollbar-thin scrollbar-thumb-white/10">
                {filteredPerms.length === 0 ? (
                    <div className="flex items-center justify-center h-full text-center">
                        <div className="text-muted-foreground text-sm">
                            <div className="mb-2 opacity-50">∅</div>
                            <div>{emptyMessage}</div>
                        </div>
                    </div>
                ) : (
                    <div>
                        {filteredPerms.map((perm) => (
                            <PermissionItem
                                key={perm.id}
                                permission={perm}
                                isSelected={selectedPermissions.has(perm.permission_string)}
                                onToggle={onTogglePermission}
                            />
                        ))}
                    </div>
                )}
            </div>

            {/* Footer with keyboard hints */}
            <div className="shrink-0 px-4 py-2 border-t border-white/10 bg-black/20">
                <div className="text-[10px] text-muted-foreground/70 font-mono space-y-0.5">
                    <div>↑↓ Navigate • Space Toggle • Ctrl+A All • Ctrl+N None</div>
                </div>
            </div>
        </div>
    );
}
