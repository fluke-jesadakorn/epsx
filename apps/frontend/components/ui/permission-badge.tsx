/* eslint-disable @typescript-eslint/no-unnecessary-condition */
'use client';

import { Badge } from '@/components/ui/badge';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';
import type {
    PermissionDefinition} from '@/shared/api/permission-definitions';
import {
    getPermissionMeta,
    getPermissionNote,
    getPermissionTitle,
    loadPermissionDefinitions
} from '@/shared/api/permission-definitions';
import { useApiClient } from '@/shared/hooks/use-api-client';
import {
    BarChart3,
    Crown,
    Eye,
    Settings,
    Shield,
    Users,
    Zap,
} from 'lucide-react';
import { useEffect, useState } from 'react';

// ============================================================================
// TYPES
// ============================================================================

export interface PermissionBadgeProps {
    /** The permission string (e.g., "epsx:analytics:view") */
    permission: string;
    /** Whether to show the note in a tooltip */
    showNote?: boolean;
    /** Whether to show the platform badge */
    showPlatform?: boolean;
    /** Additional class names */
    className?: string;
    /** Size variant */
    size?: 'sm' | 'md' | 'lg';
    /** Whether to show the technical code */
    showCode?: boolean;
}

// ============================================================================
// CONSTANTS
// ============================================================================

// Platform colors for visual categorization
const PLATFORM_COLORS: Record<string, string> = {
    epsx: 'bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:border-blue-700',
    'epsx-pay': 'bg-green-100 text-green-800 border-green-200 dark:bg-green-900/30 dark:text-green-300 dark:border-green-700',
    'epsx-token': 'bg-purple-100 text-purple-800 border-purple-200 dark:bg-purple-900/30 dark:text-purple-300 dark:border-purple-700',
    admin: 'bg-orange-100 text-orange-800 border-orange-200 dark:bg-orange-900/30 dark:text-orange-300 dark:border-orange-700',
    default: 'bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:border-gray-700',
};

// Permission icons by category/action
const PERMISSION_ICONS: Record<string, React.ElementType> = {
    view: Eye,
    read: Eye,
    manage: Settings,
    admin: Crown,
    users: Users,
    analytics: BarChart3,
    rankings: BarChart3,
    realtime: Zap,
    default: Shield,
};

// ============================================================================
// HELPERS
// ============================================================================

function getPermissionIcon(permission: string): React.ElementType {
    const parts = permission.toLowerCase().split(':');
    for (const part of parts) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (PERMISSION_ICONS[part]) {
            return PERMISSION_ICONS[part];
        }
    }
    return PERMISSION_ICONS.default;
}

// ============================================================================
// COMPONENT
// ============================================================================

/**
 * PermissionBadge - Displays a permission with human-readable title and optional tooltip
 *
 * @example
 * <PermissionBadge permission="epsx:analytics:view" showNote />
 * // Displays: "View Analytics" with tooltip showing "View basic analytics..."
 */
// eslint-disable-next-line complexity
export function PermissionBadge({
    permission,
    showNote = true,
    showPlatform = false,
    className,
    size = 'sm',
    showCode = false,
}: PermissionBadgeProps) {
    const { base } = useApiClient({ platform: 'frontend' });
    const [definitions, setDefinitions] = useState<Map<string, PermissionDefinition>>(new Map());
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const loadDefs = async () => {
            try {
                const defs = await loadPermissionDefinitions(base);
                setDefinitions(defs);
            } finally {
                setLoading(false);
            }
        };
        void loadDefs();
    }, [base]);

    const title = getPermissionTitle(permission, definitions);
    const note = getPermissionNote(permission, definitions);
    const { platform, category } = getPermissionMeta(permission, definitions);
    const Icon = getPermissionIcon(permission);

    const sizeClasses = {
        sm: 'text-xs px-2 py-0.5',
        md: 'text-sm px-2.5 py-1',
        lg: 'text-base px-3 py-1.5',
    };

    const platformColor = PLATFORM_COLORS[platform] || PLATFORM_COLORS.default;

    const badgeContent = (
        <Badge
            variant="outline"
            className={cn(
                'inline-flex items-center gap-1.5 font-medium transition-colors',
                sizeClasses[size],
                platformColor,
                className
            )}
        >
            <Icon className={cn('flex-shrink-0', size === 'sm' ? 'h-3 w-3' : size === 'md' ? 'h-3.5 w-3.5' : 'h-4 w-4')} />
            <span className="truncate">
                {loading ? '...' : title}
            </span>
            {showPlatform && (
                <span className="text-[9px] uppercase opacity-60 ml-1">
                    {platform}
                </span>
            )}
        </Badge>
    );

    // If we have a note and showNote is true, wrap in tooltip
    if (showNote === true && note !== null && note !== undefined && note.length > 0) {
        return (
            <TooltipProvider>
                <Tooltip>
                    <TooltipTrigger asChild>
                        {badgeContent}
                    </TooltipTrigger>
                    <TooltipContent className="max-w-xs">
                        <div className="space-y-1">
                            <p className="font-medium">{title}</p>
                            <p className="text-sm text-muted-foreground">{note}</p>
                            {showCode === true && (
                                <code className="text-xs text-muted-foreground/70 font-mono block mt-1">
                                    {permission}
                                </code>
                            )}
                            {category !== null && category !== undefined && category.length > 0 && (
                                <span className="text-[10px] uppercase text-muted-foreground/60">
                                    {category}
                                </span>
                            )}
                        </div>
                    </TooltipContent>
                </Tooltip>
            </TooltipProvider>
        );
    }

    return badgeContent;
}

/**
 * PermissionList - Displays a list of permissions with titles
 */
export interface PermissionListProps {
    permissions: string[];
    maxDisplay?: number;
    className?: string;
    size?: 'sm' | 'md' | 'lg';
}

export function PermissionList({
    permissions,
    maxDisplay = 3,
    className,
    size = 'sm',
}: PermissionListProps) {
    const displayed = permissions.slice(0, maxDisplay);
    const remaining = permissions.length - maxDisplay;

    return (
        <div className={cn('flex flex-wrap gap-1.5', className)}>
            {displayed.map((perm) => (
                <PermissionBadge key={perm} permission={perm} size={size} />
            ))}
            {remaining > 0 && (
                <Badge variant="secondary" className={cn('text-xs', size === 'sm' ? 'px-1.5 py-0' : 'px-2 py-0.5')}>
                    +{remaining} more
                </Badge>
            )}
        </div>
    );
}

export default PermissionBadge;
