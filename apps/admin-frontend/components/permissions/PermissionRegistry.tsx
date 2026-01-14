'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { AlertTriangle, Copy, Key, Loader2, Plus, Search, Shield, Trash2, X } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { groupMgmt, type PermissionDefinitionDto } from '@/lib/api/group-management-client';
import { copyToClipboard as copyToClipboardUtil } from '@/lib/utils';

/**
 *
 */
export function PermissionRegistry() {
    const [search, setSearch] = useState('');
    const [newPermission, setNewPermission] = useState('');
    const [isCreating, setIsCreating] = useState(false);
    const queryClient = useQueryClient();

    // Fetch permissions
    const { data: permissions = [], isLoading, error } = useQuery({
        queryKey: ['permission-definitions'],
        queryFn: () => groupMgmt.getPermissionDefinitions(),
    });

    // Create permission mutation
    const createMutation = useMutation({
        mutationFn: async (permission: string) => {
            return groupMgmt.createPermissionDefinition({
                permission,
                platform: permission.split(':')[0],
                category: permission.split(':')[1],
            });
        },
        onSuccess: () => {
            toast.success('Permission created successfully');
            setNewPermission('');
            setIsCreating(false);
            queryClient.invalidateQueries({ queryKey: ['permission-definitions'] });
        },
        onError: (err: any) => {
            toast.error(err.message || 'Failed to create permission');
        },
    });

    // Delete permission mutation
    const deleteMutation = useMutation({
        mutationFn: (id: string) => groupMgmt.deletePermissionDefinition(id),
        onSuccess: () => {
            toast.success('Permission deleted successfully');
            queryClient.invalidateQueries({ queryKey: ['permission-definitions'] });
        },
        onError: (err: any) => {
            toast.error(err.message || 'Failed to delete permission');
        },
    });

    const handleCreate = (e: React.FormEvent) => {
        e.preventDefault();
        const trimmed = newPermission.trim();
        if (!trimmed) { return; }

        if (!/^[\w-]+:[\w-]+:[\w-*]+$/.test(trimmed)) {
            toast.error('Invalid format. Use: platform:resource:action');
            return;
        }

        createMutation.mutate(trimmed);
    };

    const copyToClipboard = async (text: string) => {
        const success = await copyToClipboardUtil(text);
        if (success) {
            toast.success('Copied to clipboard');
        }
    };

    const filteredPermissions = permissions.filter(p =>
        (p.permission || '').toLowerCase().includes(search.toLowerCase()) ||
        p.description?.toLowerCase().includes(search.toLowerCase())
    );

    // Group by platform
    const groupedPermissions = filteredPermissions.reduce((acc, curr) => {
        const platform = curr.platform || 'other';
        if (!acc[platform]) { acc[platform] = []; }
        acc[platform].push(curr);
        return acc;
    }, {} as Record<string, PermissionDefinitionDto[]>);

    return (
        <div className="space-y-6">
            <div className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center">
                <div>
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-2">
                        <Key className="h-6 w-6 text-blue-600" />
                        Permission Registry
                    </h2>
                    <p className="text-gray-500 dark:text-gray-400 text-sm mt-1">
                        Define global permissions available in the system
                    </p>
                </div>
                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <div className="relative flex-1 sm:w-64">
                        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                        <Input
                            placeholder="Search permissions..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            className="pl-9 h-9"
                        />
                    </div>
                    <Button
                        onClick={() => setIsCreating(!isCreating)}
                        variant={isCreating ? "secondary" : "default"}
                        size="sm"
                        className="h-9"
                    >
                        {isCreating ? <X className="h-4 w-4 mr-2" /> : <Plus className="h-4 w-4 mr-2" />}
                        {isCreating ? 'Cancel' : 'New Permission'}
                    </Button>
                </div>
            </div>

            {/* Creation Form */}
            {isCreating && (
                <form onSubmit={handleCreate} className="bg-blue-50/50 dark:bg-blue-900/10 border border-blue-200 dark:border-blue-800 rounded-xl p-4 animate-in fade-in slide-in-from-top-2">
                    <div className="flex flex-col gap-3">
                        <Label>Permission String</Label>
                        <div className="flex gap-2">
                            <Input
                                value={newPermission}
                                onChange={(e) => setNewPermission(e.target.value)}
                                placeholder="platform:resource:action (e.g. epsx:analytics:view)"
                                className="flex-1 font-mono text-sm"
                                autoFocus
                            />
                            <Button type="submit" disabled={createMutation.isPending}>
                                {createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                                Create Definition
                            </Button>
                        </div>
                        <p className="text-xs text-muted-foreground flex items-center gap-1">
                            <AlertTriangle className="h-3 w-3" />
                            Format: <code className="text-blue-600 dark:text-blue-400">platform:resource:action</code>. Wildcards (*) allowed.
                        </p>
                    </div>
                </form>
            )}

            {/* Permission List */}
            {isLoading ? (
                <div className="text-center py-12">
                    <Loader2 className="h-8 w-8 animate-spin mx-auto text-gray-400" />
                    <p className="text-gray-500 mt-2 text-sm">Loading definitions...</p>
                </div>
            ) : error ? (
                <div className="bg-red-50 dark:bg-red-900/10 p-4 rounded-xl text-red-600 dark:text-red-400 text-sm text-center">
                    Failed to load permissions
                </div>
            ) : Object.keys(groupedPermissions).length === 0 ? (
                <div className="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-xl border border-dashed border-gray-200 dark:border-gray-700">
                    <Shield className="h-12 w-12 mx-auto text-gray-300 mb-3" />
                    <p className="text-gray-500 font-medium">No permissions found</p>
                    {search && <p className="text-gray-400 text-sm mt-1">Try adjusting your search</p>}
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {Object.entries(groupedPermissions).map(([platform, items]) => (
                        <div key={platform} className="bg-white/80 dark:bg-gray-800/80 border border-gray-200 dark:border-gray-700 rounded-xl overflow-hidden shadow-sm">
                            <div className="bg-gray-50/80 dark:bg-gray-900/50 px-4 py-2 border-b border-gray-100 dark:border-gray-800 flex justify-between items-center">
                                <h3 className="font-semibold text-sm uppercase tracking-wider text-gray-500 dark:text-gray-400">
                                    {platform}
                                </h3>
                                <Badge variant="secondary" className="text-[10px] h-5 px-1.5">
                                    {items.length}
                                </Badge>
                            </div>
                            <div className="divide-y divide-gray-100 dark:divide-gray-800">
                                {items.map((def) => (
                                    <div key={def.id} className="p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group relative">
                                        <div className="flex items-start justify-between gap-2">
                                            <div className="min-w-0 flex-1">
                                                <div className="flex items-center gap-1.5 font-mono text-sm font-medium text-gray-900 dark:text-gray-100 break-all">
                                                    {def.permission}
                                                    <button
                                                        onClick={() => copyToClipboard(def.permission)}
                                                        className="opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                                    >
                                                        <Copy className="h-3 w-3" />
                                                    </button>
                                                </div>
                                                {def.description && (
                                                    <p className="text-xs text-gray-500 mt-0.5 line-clamp-1">
                                                        {def.description}
                                                    </p>
                                                )}
                                                <div className="flex items-center gap-2 mt-2">
                                                    <Badge variant="outline" className="text-[10px] py-0 h-4 border-gray-200 dark:border-gray-700 text-gray-500">
                                                        {def.category || 'general'}
                                                    </Badge>
                                                    {def.is_system && (
                                                        <Badge variant="secondary" className="text-[10px] py-0 h-4 bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300 border-transparent">
                                                            System
                                                        </Badge>
                                                    )}
                                                </div>
                                            </div>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="h-7 w-7 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg opacity-0 group-hover:opacity-100 transition-all"
                                                onClick={() => {
                                                    if (confirm(`Delete permission definition "${def.permission}"? Note: This does not revoke it from users/groups.`)) {
                                                        deleteMutation.mutate(def.id);
                                                    }
                                                }}
                                                disabled={def.is_system}
                                            >
                                                <Trash2 className="h-3.5 w-3.5" />
                                            </Button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
