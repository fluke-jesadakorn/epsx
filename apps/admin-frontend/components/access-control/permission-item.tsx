'use client';

import { Pencil, Trash2 } from 'lucide-react';

import type { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { getPlatformColorClass, getPlatformFromPermission } from '@/lib/utils/permission-utils';

interface PermissionItemProps {
    permission: PermissionDefinition;
    isSelected: boolean;
    onToggle: (permissionString: string) => void;
    showCheckbox?: boolean;
    onEdit?: (perm: PermissionDefinition) => void;
    onDelete?: (perm: PermissionDefinition) => void;
    onDoubleClick?: (permissionString: string) => void;
    stripPlatform?: boolean;
}

export function PermissionItem({
    permission,
    isSelected,
    onToggle,
    showCheckbox = true,
    onEdit,
    onDelete,
    onDoubleClick,
    stripPlatform,
}: PermissionItemProps) {
    const platform = getPlatformFromPermission(permission.permission_string);
    const platformColor = getPlatformColorClass(platform);

    const displayString = stripPlatform
        ? permission.permission_string.replace(`${platform}:`, '')
        : permission.permission_string;

    return (
        <div
            className={cn(
                'group flex items-start gap-3 px-3 py-2 border-b border-gray-200 dark:border-slate-700 last:border-b-0',
                'terminal-hover cursor-pointer select-none',
                isSelected && 'bg-cyan-500/10'
            )}
            title={permission.permission_string}
            onClick={() => showCheckbox && onToggle(permission.permission_string)}
            onDoubleClick={() => onDoubleClick?.(permission.permission_string)}
        >
            {showCheckbox && (
                <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => onToggle(permission.permission_string)}
                    className="terminal-checkbox mt-0.5 shrink-0"
                    onClick={(e) => e.stopPropagation()}
                />
            )}

            <div className="flex-1 min-w-0">
                <div className={cn('perm-string font-semibold truncate', platformColor)}>
                    {displayString}
                </div>
                {permission.name && (
                    <div className="text-[10px] text-muted-foreground truncate">
                        {permission.name}
                    </div>
                )}
                {permission.description && (
                    <div className="text-[10px] text-muted-foreground/70 line-clamp-1 mt-0.5">
                        {permission.description}
                    </div>
                )}
            </div>

            {(onEdit ?? onDelete) && (
                <div className="hidden group-hover:flex items-center gap-1 shrink-0 mt-0.5">
                    {onEdit && (
                        <button
                            onClick={(e) => { e.stopPropagation(); onEdit(permission); }}
                            className="p-1 rounded hover:bg-black/[0.05] dark:hover:bg-white/10 text-muted-foreground hover:text-cyan-400 transition-colors"
                        >
                            <Pencil className="w-3 h-3" />
                        </button>
                    )}
                    {onDelete && !permission.is_system && (
                        <button
                            onClick={(e) => { e.stopPropagation(); onDelete(permission); }}
                            className="p-1 rounded hover:bg-black/[0.05] dark:hover:bg-white/10 text-muted-foreground hover:text-red-400 transition-colors"
                        >
                            <Trash2 className="w-3 h-3" />
                        </button>
                    )}
                </div>
            )}
        </div>
    );
}
