'use client';

import { ChevronDown, ChevronRight } from 'lucide-react';
import { useState } from 'react';

import { PermissionDefinition } from '@/lib/api/permissions-client';
import { cn } from '@/lib/utils';
import { getPlatformColorClass, getPlatformDisplayName } from '@/lib/utils/permission-utils';

import { PermissionItem } from './PermissionItem';

interface PermissionCategoryGroupProps {
    platform: string;
    permissions: PermissionDefinition[];
    selectedPermissions: Set<string>;
    onTogglePermission: (permissionString: string) => void;
    onToggleAll: (permissionStrings: string[]) => void;
    showCheckboxes?: boolean;
}

export function PermissionCategoryGroup({
    platform,
    permissions,
    selectedPermissions,
    onTogglePermission,
    onToggleAll,
    showCheckboxes = true,
}: PermissionCategoryGroupProps) {
    const [isExpanded, setIsExpanded] = useState(true);

    const platformColor = getPlatformColorClass(platform);
    const displayName = getPlatformDisplayName(platform);

    const allSelected = permissions.every((p) =>
        selectedPermissions.has(p.permission_string)
    );
    const someSelected = permissions.some((p) =>
        selectedPermissions.has(p.permission_string)
    );

    const handleToggleAll = () => {
        onToggleAll(permissions.map((p) => p.permission_string));
    };

    if (permissions.length === 0) {return null;}

    return (
        <div className="mb-3">
            {/* Category Header with ASCII border */}
            <div
                className={cn(
                    'flex items-center justify-between px-3 py-2 rounded-t-lg border border-white/10 bg-white/5',
                    'cursor-pointer select-none terminal-hover',
                    platformColor
                )}
                onClick={() => setIsExpanded(!isExpanded)}
            >
                <div className="flex items-center gap-2">
                    {isExpanded ? (
                        <ChevronDown className="w-3 h-3" />
                    ) : (
                        <ChevronRight className="w-3 h-3" />
                    )}
                    <span className="font-bold text-xs tracking-wider ascii-box">
                        {displayName}
                    </span>
                    <span className="text-[10px] opacity-60">({permissions.length})</span>
                </div>

                {showCheckboxes && (
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            handleToggleAll();
                        }}
                        className="text-[10px] px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 transition-colors"
                    >
                        {allSelected ? 'Deselect All' : someSelected ? 'Select All' : 'Select All'}
                    </button>
                )}
            </div>

            {/* Permission List */}
            {isExpanded && (
                <div className="border-x border-b border-white/10 rounded-b-lg bg-black/20">
                    {permissions.map((perm) => (
                        <PermissionItem
                            key={perm.id}
                            permission={perm}
                            isSelected={selectedPermissions.has(perm.permission_string)}
                            onToggle={onTogglePermission}
                            showCheckbox={showCheckboxes}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}
