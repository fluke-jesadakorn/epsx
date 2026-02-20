'use client';

import { ChevronDown, ChevronRight, Plus, Search, X } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import {
    filterPermissions,
    getPlatformColorClass,
    getPlatformDisplayName,
    groupPermissionsByPlatform,
    sortPermissions,
} from '@/lib/utils/permission-utils';
import { PermissionItem } from './permission-item';

interface PermissionPaneProps {
    title: string;
    permissions: PermissionDefinition[];
    selectedPermissions: Set<string>;
    onTogglePermission: (permissionString: string) => void;
    onSelectAll: () => void;
    onSelectNone: () => void;
    emptyMessage?: string;
    className?: string;
    onCreate?: () => void;
    onEdit?: (perm: PermissionDefinition) => void;
    onDelete?: (perm: PermissionDefinition) => void;
    onDoubleClickItem?: (permissionString: string) => void;
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
    onCreate,
    onEdit,
    onDelete,
    onDoubleClickItem,
}: PermissionPaneProps) {
    const [searchQuery, setSearchQuery] = useState('');
    const [collapsed, setCollapsed] = useState<Set<string>>(new Set());

    const filteredPerms = useMemo(
        () => sortPermissions(filterPermissions(permissions, searchQuery)),
        [permissions, searchQuery]
    );

    const grouped = useMemo(
        () => groupPermissionsByPlatform(filteredPerms),
        [filteredPerms]
    );

    const selectedCount = permissions.filter((p) =>
        selectedPermissions.has(p.permission_string)
    ).length;

    const toggleGroup = useCallback((platform: string) => {
        setCollapsed((prev) => {
            const next = new Set(prev);
            if (next.has(platform)) {
                next.delete(platform);
            } else {
                next.add(platform);
            }
            return next;
        });
    }, []);

    return (
        <div className={cn(
            'flex flex-col h-full border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-900 backdrop-blur-xl overflow-hidden',
            className
        )}>
            {/* Header */}
            <div className="shrink-0 p-3 border-b border-gray-200 dark:border-slate-700 bg-white dark:bg-white/[0.04]">
                <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                        <h3 className="text-xs font-semibold tracking-wide text-foreground/80">
                            {title}
                        </h3>
                        <Badge variant="secondary" className="text-[10px] h-5">
                            {filteredPerms.length}
                        </Badge>
                        {selectedCount > 0 && (
                            <Badge variant="outline" className="text-[10px] h-5 border-cyan-500/30 text-cyan-400">
                                {selectedCount} sel
                            </Badge>
                        )}
                    </div>

                    <div className="flex gap-1">
                        {onCreate && (
                            <Button
                                size="sm"
                                variant="ghost"
                                onClick={onCreate}
                                className="h-6 w-6 p-0 text-emerald-400 hover:text-emerald-300 hover:bg-emerald-500/10"
                            >
                                <Plus className="w-3.5 h-3.5" />
                            </Button>
                        )}
                        <Button size="sm" variant="ghost" onClick={onSelectAll} className="h-6 px-2 text-[10px]">
                            All
                        </Button>
                        <Button size="sm" variant="ghost" onClick={onSelectNone} className="h-6 px-2 text-[10px]">
                            None
                        </Button>
                    </div>
                </div>

                <div className="relative">
                    <Search className="absolute left-2.5 top-2 h-3.5 w-3.5 text-muted-foreground/50" />
                    <Input
                        placeholder="Filter permissions..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="h-8 pl-8 pr-8 text-xs bg-black/20 border-gray-200 dark:border-slate-700"
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

            {/* Permission List — Grouped */}
            <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-white/10">
                {filteredPerms.length === 0 ? (
                    <div className="flex items-center justify-center h-full text-center">
                        <div className="text-muted-foreground text-sm">
                            <div className="mb-2 opacity-50">&empty;</div>
                            <div>{emptyMessage}</div>
                        </div>
                    </div>
                ) : (
                    Object.entries(grouped).map(([platform, perms]) => {
                        const isCollapsed = collapsed.has(platform);
                        const color = getPlatformColorClass(platform);
                        const selInGroup = perms.filter((p) =>
                            selectedPermissions.has(p.permission_string)
                        ).length;

                        return (
                            <div key={platform}>
                                {/* Sticky group header */}
                                <div
                                    className={cn(
                                        'sticky top-0 z-10 flex items-center gap-2 px-3 py-1.5',
                                        'bg-white dark:bg-slate-900 backdrop-blur border-b border-gray-200 dark:border-slate-700 cursor-pointer select-none',
                                        'hover:bg-gray-100 dark:hover:bg-white/5 transition-colors'
                                    )}
                                    onClick={() => toggleGroup(platform)}
                                >
                                    {isCollapsed
                                        ? <ChevronRight className="w-3 h-3 text-muted-foreground" />
                                        : <ChevronDown className="w-3 h-3 text-muted-foreground" />
                                    }
                                    <span className={cn('w-1.5 h-1.5 rounded-full', color.replace('text-', 'bg-'))} />
                                    <span className={cn('text-[11px] font-bold tracking-wider', color)}>
                                        {getPlatformDisplayName(platform)}
                                    </span>
                                    <span className="text-[10px] text-muted-foreground/60">
                                        {perms.length}
                                    </span>
                                    {selInGroup > 0 && (
                                        <span className="text-[10px] text-cyan-400/80">
                                            ({selInGroup})
                                        </span>
                                    )}
                                </div>

                                {!isCollapsed && perms.map((perm) => (
                                    <PermissionItem
                                        key={perm.id}
                                        permission={perm}
                                        isSelected={selectedPermissions.has(perm.permission_string)}
                                        onToggle={onTogglePermission}
                                        onEdit={onEdit}
                                        onDelete={onDelete}
                                        onDoubleClick={onDoubleClickItem}
                                        stripPlatform
                                    />
                                ))}
                            </div>
                        );
                    })
                )}
            </div>
        </div>
    );
}
