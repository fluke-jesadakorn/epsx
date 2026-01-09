/**
 * Group Hub Component
 * Main unified hub for group management - matches WalletHub design
 */
'use client';

import { Clock, Key, Plus, RefreshCw, Search, Shield, Trash2, Wallet } from 'lucide-react';
import Link from 'next/link';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { GroupCard } from './GroupCard';
import { GroupStatsBar } from './GroupStatsBar';
import type { GroupData, GroupFilters, GroupStats, GroupType } from './types';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';

interface GroupHubProps {
    className?: string;
}

// Transform API data to our GroupData format
function transformGroup(group: PermissionGroup): GroupData {
    return {
        id: group.id,
        name: group.name,
        slug: group.slug,
        description: group.description,
        groupType: (group.group_type as GroupType) || 'manual',
        permissions: group.permissions,
        memberCount: group.member_count ?? 0,
        priorityLevel: group.priority_level ?? 0,
        defaultExpiryDays: group.default_expiry_days,
        isActive: group.is_active,
        isSystemGroup: group.is_system_group || group.group_type === 'system',
        createdAt: group.created_at,
        updatedAt: group.updated_at,
    };
}

// Default empty stats for loading state
const DEFAULT_STATS: GroupStats = {
    totalGroups: 0,
    activeMemberships: 0,
    expiringSoon: 0,
    largestGroup: { name: '', memberCount: 0 },
};

/**
 * GroupHub component - main container for group management
 */
export function GroupHub({ className }: GroupHubProps) {
    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // State
    const [groups, setGroups] = useState<GroupData[]>([]);
    const [stats, setStats] = useState<GroupStats>(DEFAULT_STATS);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Delete state
    const [deleteConfirm, setDeleteConfirm] = useState<{ groupId: string; groupName: string } | null>(null);
    const [isDeleting, setIsDeleting] = useState(false);

    // Filters
    const [filters, setFilters] = useState<GroupFilters>({
        search: '',
        groupType: 'all',
        sortBy: 'name',
        sortOrder: 'asc',
    });

    // Selection
    const [selectedGroups, setSelectedGroups] = useState<Set<string>>(new Set());

    // Load data from API
    const loadData = useCallback(async () => {
        setIsLoading(true);
        setError(null);

        try {
            const [groupsData, analyticsData] = await Promise.all([
                groupMgmt.getPermissionGroups(),
                groupMgmt.getGroupAnalytics(),
            ]);

            setGroups(groupsData.map(transformGroup));

            // Transform analytics to our stats format
            setStats({
                totalGroups: analyticsData.total_groups,
                activeMemberships: analyticsData.total_active_memberships,
                expiringSoon: analyticsData.expiring_soon_count,
                largestGroup: analyticsData.most_popular_groups?.[0] || { name: '', memberCount: 0 },
            });
        } catch (err) {
            console.error('Failed to load group data:', err);
            setError(err instanceof Error ? err.message : 'Failed to load data');
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            loadData();
        }
    }, [isAuthenticated, authLoading, loadData]);

    // Filter and sort groups
    const filteredGroups = useMemo(() => {
        let result = [...groups];

        // Search filter
        if (filters.search) {
            const searchLower = filters.search.toLowerCase();
            result = result.filter((group) =>
                group.name.toLowerCase().includes(searchLower) ||
                group.description.toLowerCase().includes(searchLower)
            );
        }

        // Type filter
        if (filters.groupType !== 'all') {
            result = result.filter((group) => group.groupType === filters.groupType);
        }

        // Sort
        result.sort((a, b) => {
            let comparison = 0;
            switch (filters.sortBy) {
                case 'name':
                    comparison = a.name.localeCompare(b.name);
                    break;
                case 'members':
                    comparison = a.memberCount - b.memberCount;
                    break;
                case 'permissions':
                    comparison = a.permissions.length - b.permissions.length;
                    break;
                case 'created_at':
                    comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
                    break;
            }
            return filters.sortOrder === 'asc' ? comparison : -comparison;
        });

        return result;
    }, [groups, filters]);

    // Selection handlers
    const handleSelectGroup = (groupId: string, selected: boolean) => {
        setSelectedGroups((prev) => {
            const next = new Set(prev);
            if (selected) {
                next.add(groupId);
            } else {
                next.delete(groupId);
            }
            return next;
        });
    };

    // Delete handler
    const handleDeleteGroup = async () => {
        if (!deleteConfirm) return;

        setIsDeleting(true);
        try {
            await groupMgmt.deletePermissionGroup(deleteConfirm.groupId);
            toast.success('Permission group deleted successfully');
            setDeleteConfirm(null);
            await loadData();
        } catch (err) {
            console.error('Failed to delete group:', err);
            toast.error('Failed to delete permission group');
        } finally {
            setIsDeleting(false);
        }
    };

    // Auth check
    if (authLoading) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="text-center">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4" />
                    <p className="text-gray-600 dark:text-gray-400">Checking authentication...</p>
                </div>
            </div>
        );
    }

    if (!isAuthenticated) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="text-center max-w-md">
                    <div className="text-4xl mb-4">🔐</div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                        Authentication Required
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400">
                        Please connect your wallet to access the group management hub.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className={cn('space-y-6', className)}>
            {/* Stats Dashboard */}
            <GroupStatsBar stats={stats} isLoading={isLoading && stats.totalGroups === 0} />

            {/* Quick Actions */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                {/* Permission Registry */}
                <Link href="/group-and-permission" className="block group">
                    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-purple-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl h-full">
                            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-purple-400 to-pink-500 rounded-full blur-sm opacity-60"></div>
                            <div className="p-4 sm:p-6">
                                <div className="flex items-center gap-2 mb-2">
                                    <Key className="w-5 h-5 text-purple-500" />
                                    <h3 className="text-lg font-bold bg-gradient-to-r from-purple-400 to-pink-500 bg-clip-text text-transparent">
                                        Permissions
                                    </h3>
                                </div>
                                <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                                    Define global permissions
                                </p>
                                <div className="flex items-center justify-between mt-auto">
                                    <div className="px-3 py-1 bg-gradient-to-r from-purple-400 to-pink-500 text-white rounded-full text-xs font-medium">
                                        Registry
                                    </div>
                                    <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Link>

                {/* Create Group */}
                <Link href="/group-and-permission/create-group" className="block group">
                    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-cyan-400/20 to-blue-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl h-full">
                            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full blur-sm opacity-60"></div>
                            <div className="p-4 sm:p-6">
                                <div className="flex items-center gap-2 mb-2">
                                    <Plus className="w-5 h-5 text-blue-500" />
                                    <h3 className="text-lg font-bold bg-gradient-to-r from-blue-400 to-cyan-500 bg-clip-text text-transparent">
                                        Create Group
                                    </h3>
                                </div>
                                <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                                    Create a new permission group
                                </p>
                                <div className="flex items-center justify-between mt-auto">
                                    <div className="px-3 py-1 bg-gradient-to-r from-blue-400 to-cyan-500 text-white rounded-full text-xs font-medium">
                                        Open
                                    </div>
                                    <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Link>

                {/* Assign Wallet */}
                <Link href="/group-and-permission/assign-wallet" className="block group">
                    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-green-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl h-full">
                            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full blur-sm opacity-60"></div>
                            <div className="p-4 sm:p-6">
                                <div className="flex items-center gap-2 mb-2">
                                    <Wallet className="w-5 h-5 text-green-500" />
                                    <h3 className="text-lg font-bold bg-gradient-to-r from-green-400 to-emerald-500 bg-clip-text text-transparent">
                                        Assign Wallet
                                    </h3>
                                </div>
                                <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                                    Assign wallet to a group
                                </p>
                                <div className="flex items-center justify-between mt-auto">
                                    <div className="px-3 py-1 bg-gradient-to-r from-green-400 to-emerald-500 text-white rounded-full text-xs font-medium">
                                        Open
                                    </div>
                                    <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Link>

                {/* Expiring Soon */}
                <Link href="/group-and-permission/expiring" className="block group">
                    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-orange-400/20 via-pink-400/20 to-orange-400/20 p-0.5 hover:scale-105 transition-all duration-300">
                        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl h-full">
                            <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-orange-400 to-pink-500 rounded-full blur-sm opacity-60"></div>
                            <div className="p-4 sm:p-6">
                                <div className="flex items-center gap-2 mb-2">
                                    <Clock className="w-5 h-5 text-orange-500" />
                                    <h3 className="text-lg font-bold bg-gradient-to-r from-orange-400 to-pink-500 bg-clip-text text-transparent">
                                        Expiring Soon
                                    </h3>
                                </div>
                                <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                                    View expiring assignments
                                </p>
                                <div className="flex items-center justify-between mt-auto">
                                    <div className="px-3 py-1 bg-gradient-to-r from-orange-400 to-pink-500 text-white rounded-full text-xs font-medium">
                                        Open
                                    </div>
                                    <div className="text-gray-500 dark:text-gray-400 group-hover:translate-x-1 transition-transform duration-200">→</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Link>
            </div>

            {/* Search & Filters */}
            <div className="rounded-2xl bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700 p-4">
                <div className="flex flex-col sm:flex-row gap-4">
                    {/* Search */}
                    <div className="flex-1">
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                            <Input
                                placeholder="Search groups..."
                                value={filters.search}
                                onChange={(e) => setFilters((prev) => ({ ...prev, search: e.target.value }))}
                                className="pl-10"
                            />
                        </div>
                    </div>

                    {/* Type Filter */}
                    <Select
                        value={filters.groupType}
                        onValueChange={(v) => setFilters((prev) => ({ ...prev, groupType: v as any }))}
                    >
                        <SelectTrigger className="w-40">
                            <SelectValue placeholder="Type" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Types</SelectItem>
                            <SelectItem value="manual">Manual</SelectItem>
                            <SelectItem value="subscription">Subscription</SelectItem>
                            <SelectItem value="system">System</SelectItem>
                        </SelectContent>
                    </Select>

                    {/* Sort */}
                    <Select
                        value={filters.sortBy}
                        onValueChange={(v) => setFilters((prev) => ({ ...prev, sortBy: v as any }))}
                    >
                        <SelectTrigger className="w-40">
                            <SelectValue placeholder="Sort by" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="name">Name</SelectItem>
                            <SelectItem value="members">Members</SelectItem>
                            <SelectItem value="permissions">Permissions</SelectItem>
                            <SelectItem value="created_at">Date Created</SelectItem>
                        </SelectContent>
                    </Select>

                    {/* Refresh */}
                    <Button
                        variant="outline"
                        onClick={loadData}
                        disabled={isLoading}
                        className="gap-2"
                    >
                        <RefreshCw className={cn('h-4 w-4', isLoading && 'animate-spin')} />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Error */}
            {error && (
                <div className="p-4 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400">
                    ⚠️ {error}
                </div>
            )}

            {/* Group List */}
            <div className="space-y-3">
                {isLoading ? (
                    // Loading skeleton
                    Array.from({ length: 5 }).map((_, i) => (
                        <div key={i} className="rounded-2xl bg-gray-100 dark:bg-gray-800 p-6 animate-pulse">
                            <div className="flex items-center gap-4">
                                <Skeleton className="h-12 w-12 rounded-xl" />
                                <div className="space-y-2 flex-1">
                                    <Skeleton className="h-4 w-40" />
                                    <Skeleton className="h-3 w-32" />
                                </div>
                                <Skeleton className="h-9 w-20" />
                            </div>
                        </div>
                    ))
                ) : filteredGroups.length === 0 ? (
                    <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                        <Shield className="h-12 w-12 mx-auto mb-4 opacity-30" />
                        <p className="font-medium">No groups found</p>
                        <p className="text-sm mt-1">Try adjusting your filters or create a new group</p>
                    </div>
                ) : (
                    filteredGroups.map((group) => (
                        <GroupCard
                            key={group.id}
                            group={group}
                            isSelected={selectedGroups.has(group.id)}
                            onSelect={(selected) => handleSelectGroup(group.id, selected)}
                            onDelete={() => setDeleteConfirm({ groupId: group.id, groupName: group.name })}
                        />
                    ))
                )}
            </div>

            {/* Delete Confirmation Modal */}
            {deleteConfirm && (
                <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
                    <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl border border-gray-200 dark:border-gray-700">
                        <div className="flex items-center gap-3 mb-4">
                            <div className="p-3 bg-red-100 dark:bg-red-900/30 rounded-full">
                                <Trash2 className="w-6 h-6 text-red-600 dark:text-red-400" />
                            </div>
                            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                Delete Permission Group
                            </h3>
                        </div>
                        <p className="text-gray-600 dark:text-gray-400 mb-6">
                            Are you sure you want to delete <strong>"{deleteConfirm.groupName}"</strong>?
                            This action cannot be undone.
                        </p>
                        <div className="flex gap-3">
                            <Button
                                variant="outline"
                                onClick={() => setDeleteConfirm(null)}
                                className="flex-1"
                            >
                                Cancel
                            </Button>
                            <Button
                                variant="destructive"
                                onClick={handleDeleteGroup}
                                disabled={isDeleting}
                                className="flex-1 bg-red-600 hover:bg-red-700 text-white"
                            >
                                {isDeleting ? 'Deleting...' : 'Delete'}
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
