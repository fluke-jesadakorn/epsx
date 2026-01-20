/**
 * Group Card Component
 * Premium compact group display for list view with action buttons
 * Matches WalletCard design pattern
 */
'use client';

import { Edit, Eye, MoreHorizontal, Trash2, Users } from 'lucide-react';
import Link from 'next/link';

import type { GroupData, GroupType } from './types';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';

interface GroupCardProps {
    group: GroupData;
    isSelected?: boolean;
    onSelect?: (selected: boolean) => void;
    onView?: () => void;
    onEdit?: () => void;
    onDelete?: () => void;
    onManageMembers?: () => void;
    className?: string;
}

const GROUP_TYPE_CONFIG: Record<GroupType, { label: string; className: string }> = {
    manual: {
        label: 'Manual',
        className: 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:border-blue-800',
    },
    subscription: {
        label: 'Subscription',
        className: 'bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800',
    },
    web3_asset: {
        label: 'Web3 Asset',
        className: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-900/30 dark:text-amber-400 dark:border-amber-800',
    },
    dao_membership: {
        label: 'DAO',
        className: 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800',
    },
    admin: {
        label: 'Admin',
        className: 'bg-red-50 text-red-700 border-red-200 dark:bg-red-900/30 dark:text-red-400 dark:border-red-800',
    },
    system: {
        label: 'System',
        className: 'bg-gray-50 text-gray-700 border-gray-200 dark:bg-gray-900/30 dark:text-gray-400 dark:border-gray-700',
    },
};

// Generate a deterministic gradient based on group name
function getAvatarGradient(name: string): string {
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue1 = Math.abs(hash % 360);
    const hue2 = (hue1 + 40) % 360;
    return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%) 0%, hsl(${hue2}, 80%, 50%) 100%)`;
}

// Get initials from group name
function getInitials(name: string): string {
    return name
        .split(' ')
        .map(word => word[0])
        .join('')
        .substring(0, 2)
        .toUpperCase();
}

/**
 * GroupCard displays a single permission group in a card format
 */
export function GroupCard({
    group,
    isSelected = false,
    onSelect,
    onView,
    onEdit,
    onDelete,
    onManageMembers,
    className,
}: GroupCardProps) {
    const typeConfig = GROUP_TYPE_CONFIG[group.groupType] || GROUP_TYPE_CONFIG.manual;

    return (
        <div
            className={cn(
                // Base card styles with premium glassmorphism
                'group relative rounded-2xl overflow-hidden',
                'bg-card text-card-foreground',
                'border border-border',
                // Smooth transitions
                'transition-all duration-300 ease-out',
                // Hover effects
                'hover:shadow-xl hover:shadow-primary/5',
                'hover:border-border',
                'hover:scale-[1.01] hover:-translate-y-0.5',
                // Selected state
                isSelected && 'ring-2 ring-primary ring-offset-2 dark:ring-offset-background border-primary/50',
                className
            )}
        >
            {/* Subtle gradient overlay on hover */}
            <div className="absolute inset-0 bg-gradient-to-br from-blue-500/0 via-purple-500/0 to-pink-500/0 group-hover:from-blue-500/3 group-hover:via-purple-500/3 group-hover:to-pink-500/3 transition-all duration-500 pointer-events-none" />

            {/* Shine effect on hover */}
            <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none overflow-hidden">
                <div className="absolute -inset-full top-0 w-1/2 h-full bg-gradient-to-r from-transparent via-white/10 to-transparent skew-x-12 group-hover:animate-[shimmer_2s_ease-in-out_infinite]" />
            </div>

            <div className="relative flex flex-col p-5 gap-4">
                {/* Header: Checkbox + Avatar + Name + Type Badge */}
                <div className="flex items-center gap-4">
                    {/* Checkbox */}
                    {onSelect && (
                        <div className="flex items-center">
                            <input
                                type="checkbox"
                                checked={isSelected}
                                onChange={(e) => onSelect(e.target.checked)}
                                className="h-4 w-4 rounded-md border-border text-primary focus:ring-primary focus:ring-offset-0 transition-colors cursor-pointer bg-card"
                            />
                        </div>
                    )}

                    {/* Premium Avatar */}
                    <div className="relative group/avatar">
                        <div
                            className={cn(
                                'relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl text-sm font-bold shadow-lg',
                                'text-white transition-all duration-300',
                                'group-hover/avatar:scale-105 group-hover/avatar:shadow-xl',
                            )}
                            style={{ background: getAvatarGradient(group.name) }}
                        >
                            {getInitials(group.name)}

                            {/* System indicator */}
                            {group.isSystemGroup && (
                                <div className={cn(
                                    'absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white dark:border-gray-900',
                                    'bg-purple-500',
                                )}>
                                    <span className={cn(
                                        'absolute inset-0 rounded-full bg-purple-500',
                                        'animate-ping opacity-40'
                                    )} />
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Name & Description */}
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                            <span className="font-semibold text-gray-900 dark:text-gray-100 truncate">
                                {group.name}
                            </span>
                        </div>
                        {group.description && (
                            <p className="text-xs text-gray-500 dark:text-gray-400 line-clamp-1 mt-0.5">
                                {group.description}
                            </p>
                        )}
                    </div>

                    {/* Status Badge */}
                    <div className="hidden sm:flex items-center gap-2">
                        {group.isSystemGroup && (
                            <Badge className="text-xs px-2 py-0.5 font-medium border rounded-full bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800">
                                System
                            </Badge>
                        )}
                        <Badge className={cn(
                            'text-xs px-3 py-1 font-semibold border rounded-full',
                            'transition-all duration-200 hover:scale-105',
                            'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800'
                        )}>
                            Active
                        </Badge>
                    </div>

                    {/* Actions Menu */}
                    <div className="flex items-center gap-1.5">
                        <Link href={`/group-and-permission/groups/${group.id}/edit`}>
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn(
                                    'hidden sm:flex h-9 px-4 gap-2 text-sm font-medium rounded-xl',
                                    'bg-gradient-to-r from-blue-50 to-indigo-50 text-blue-700',
                                    'hover:from-blue-100 hover:to-indigo-100 hover:text-blue-800',
                                    'dark:from-blue-900/30 dark:to-indigo-900/30 dark:text-blue-400',
                                    'dark:hover:from-blue-900/50 dark:hover:to-indigo-900/50',
                                    'transition-all duration-200 hover:scale-105 hover:shadow-md'
                                )}
                            >
                                <Eye className="h-4 w-4" />
                                View
                            </Button>
                        </Link>

                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className="h-9 w-9 p-0 rounded-xl hover:bg-muted transition-all duration-200 hover:scale-110"
                                >
                                    <MoreHorizontal className="h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" className="w-48 rounded-xl p-1">
                                <Link href={`/group-and-permission/groups/${group.id}/edit`}>
                                    <DropdownMenuItem className="rounded-lg">
                                        <Edit className="h-4 w-4 mr-2" />
                                        <span className="text-sm">Edit Group</span>
                                    </DropdownMenuItem>
                                </Link>
                                <Link href={`/group-and-permission/groups/${group.id}/members`}>
                                    <DropdownMenuItem className="rounded-lg">
                                        <Users className="h-4 w-4 mr-2" />
                                        <span className="text-sm">Manage Members</span>
                                    </DropdownMenuItem>
                                </Link>
                                <DropdownMenuSeparator />
                                {!group.isSystemGroup && (
                                    <DropdownMenuItem
                                        onClick={onDelete}
                                        className="text-red-700 dark:text-red-400 rounded-lg"
                                    >
                                        <Trash2 className="h-4 w-4 mr-2" />
                                        <span className="text-sm">Delete Group</span>
                                    </DropdownMenuItem>
                                )}
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>
                </div>

                {/* Key Metrics Grid - Enhanced */}
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    {/* Type */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Type</span>
                        <Badge className={cn('text-xs px-2 py-0.5 font-medium border rounded w-fit', typeConfig.className)}>
                            {typeConfig.label}
                        </Badge>
                    </div>

                    {/* Members */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Members</span>
                        <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                            {group.memberCount} <span className="text-gray-500 font-normal text-xs">wallets</span>
                        </span>
                    </div>

                    {/* Permissions */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Perms</span>
                        <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                            {group.permissions.length} <span className="text-gray-500 font-normal text-xs">active</span>
                        </span>
                    </div>

                    {/* Priority */}
                    <div className="flex flex-col p-3 rounded-xl bg-gradient-to-br from-gray-50 to-gray-100/50 dark:from-gray-800/50 dark:to-gray-900/50 border border-gray-100 dark:border-gray-800 transition-all duration-200 hover:border-gray-200 dark:hover:border-gray-700">
                        <span className="text-[10px] uppercase text-gray-500 font-semibold tracking-wider mb-1">Priority</span>
                        <div className="flex items-center gap-1.5">
                            <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                                Level {group.priorityLevel}
                            </span>
                        </div>
                    </div>
                </div>

                {/* Permissions Preview */}
                {group.permissions.length > 0 && (
                    <div className="flex flex-wrap gap-1.5">
                        {group.permissions.slice(0, 4).map((perm) => (
                            <span
                                key={perm}
                                className="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 rounded-lg"
                            >
                                {perm.split(':').pop()}
                            </span>
                        ))}
                        {group.permissions.length > 4 && (
                            <span className="text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-lg font-medium">
                                +{group.permissions.length - 4} more
                            </span>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
