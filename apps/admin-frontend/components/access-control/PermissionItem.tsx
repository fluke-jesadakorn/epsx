'use client';

import { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { getPlatformColorClass, getPlatformFromPermission } from '@/lib/utils/permission-utils';

interface PermissionItemProps {
    permission: PermissionDefinition;
    isSelected: boolean;
    onToggle: (permissionString: string) => void;
    showCheckbox?: boolean;
}

export function PermissionItem({
    permission,
    isSelected,
    onToggle,
    showCheckbox = true,
}: PermissionItemProps) {
    const platform = getPlatformFromPermission(permission.permission_string);
    const platformColor = getPlatformColorClass(platform);

    return (
        <div
            className={cn(
                'flex items-start gap-3 px-3 py-2 border-b border-white/5 last:border-b-0',
                'terminal-hover cursor-pointer select-none',
                isSelected && 'bg-cyan-500/10'
            )}
            onClick={() => showCheckbox && onToggle(permission.permission_string)}
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
                    {permission.permission_string}
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
        </div>
    );
}
