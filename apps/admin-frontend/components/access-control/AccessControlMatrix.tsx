'use client';

import { AccessPolicy, POLICY_TYPE_CONFIG } from '@/components/access-control/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { useToast } from '@/hooks/use-toast';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import { PermissionDefinitionDto, planMgmt } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { Check, Info, Plus, Search, Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';

interface AccessControlMatrixProps {
    className?: string;
}

export function AccessControlMatrix({ className }: AccessControlMatrixProps) {
    const router = useRouter();
    const { toast } = useToast();
    const [policies, setPolicies] = useState<AccessPolicy[]>([]);
    const [permissions, setPermissions] = useState<PermissionDefinitionDto[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [search, setSearch] = useState('');
    const [isUpdating, setIsUpdating] = useState<Record<string, boolean>>({});

    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            // Load both datasets in parallel
            const [policiesData, permissionsData] = await Promise.all([
                accessPolicyClient.getPolicies(),
                planMgmt.getPermissionDefinitions()
            ]);

            setPolicies(policiesData);
            setPermissions(permissionsData);
        } catch (err) {
            console.error('Failed to load matrix data:', err);
            toast({
                title: 'Error',
                description: 'Failed to load access control data',
                variant: 'destructive'
            });
        } finally {
            setIsLoading(false);
        }
    }, [toast]);

    useEffect(() => {
        loadData();
    }, [loadData]);

    // Group permissions by platform
    const groupedPermissions = useMemo(() => {
        const filtered = permissions.filter(p =>
            (p.permission && p.permission.toLowerCase().includes(search.toLowerCase())) ||
            (p.description && p.description.toLowerCase().includes(search.toLowerCase()))
        );

        const groups: Record<string, PermissionDefinitionDto[]> = {};

        filtered.forEach(p => {
            const platform = p.platform || 'Other';
            if (!groups[platform]) groups[platform] = [];
            groups[platform].push(p);
        });

        // specific sort order for platforms
        const sortOrder = ['epsx', 'admin', 'pay', 'token', 'other'];
        return Object.entries(groups).sort((a, b) => {
            const indexA = sortOrder.indexOf(a[0].toLowerCase());
            const indexB = sortOrder.indexOf(b[0].toLowerCase());
            if (indexA !== -1 && indexB !== -1) return indexA - indexB;
            if (indexA !== -1) return -1;
            if (indexB !== -1) return 1;
            return a[0].localeCompare(b[0]);
        });
    }, [permissions, search]);

    // Handle Permission Toggle
    const togglePermission = async (policy: AccessPolicy, permissionKey: string) => {
        // Optimistic update ID
        const updateKey = `${policy.id}-${permissionKey}`;
        if (isUpdating[updateKey]) return;

        setIsUpdating(prev => ({ ...prev, [updateKey]: true }));

        const hasPermission = policy.permissions.includes(permissionKey);
        const newPermissions = hasPermission
            ? policy.permissions.filter(p => p !== permissionKey)
            : [...policy.permissions, permissionKey];

        // Optimistically update local state
        setPolicies(current => current.map(p => {
            if (p.id === policy.id) {
                return { ...p, permissions: newPermissions };
            }
            return p;
        }));

        try {
            await accessPolicyClient.updatePolicy(policy.id, {
                permissions: newPermissions
            });
        } catch (err) {
            console.error('Failed to update policy:', err);
            toast({
                title: 'Update failed',
                description: 'Could not update permission assignment',
                variant: 'destructive'
            });
            // Revert on failure
            loadData();
        } finally {
            setIsUpdating(prev => ({ ...prev, [updateKey]: false }));
        }
    };

    if (isLoading) {
        return (
            <div className="space-y-4">
                <div className="flex gap-4">
                    <Skeleton className="h-10 w-64" />
                    <Skeleton className="h-10 w-32" />
                </div>
                <Skeleton className="h-[600px] w-full rounded-xl" />
            </div>
        );
    }

    return (
        <div className={cn("space-y-4", className)}>
            {/* Header / Actions */}
            <div className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-card/50 p-1 rounded-xl">
                <div className="relative w-full sm:max-w-md">
                    <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="Search permissions..."
                        className="pl-9 h-10 bg-background/50 border-border/50 focus:bg-background transition-colors"
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                    />
                </div>

                <div className="flex gap-2 w-full sm:w-auto">
                    <Button variant="outline" size="sm" onClick={() => router.push('/subscriptions/plans/new')}>
                        <Plus className="h-4 w-4 mr-2" />
                        New Policy
                    </Button>
                    <Button size="sm" onClick={() => {
                        // Ideally open a modal, but for now specific route or placeholder
                        toast({ title: "Coming Soon", description: "Standard permission creation modal coming next." });
                    }}>
                        <Shield className="h-4 w-4 mr-2" />
                        New Permission
                    </Button>
                </div>
            </div>

            {/* Matrix Container */}
            <div className="border border-border rounded-xl bg-card overflow-hidden flex flex-col h-[700px]">
                <div className="flex-1 w-full h-full overflow-auto scrollbar-thin scrollbar-thumb-muted-foreground/20 scrollbar-track-transparent">
                    <div className="min-w-max relative pb-4">

                        {/* Sticky Header Row (Policies) */}
                        <div className="flex sticky top-0 z-20 bg-card border-b border-border shadow-sm min-w-max">
                            {/* Empty strings/spacer for permission column */}
                            <div className="sticky left-0 z-30 w-[350px] bg-card p-4 border-r border-border/50 flex items-end font-medium text-muted-foreground text-sm">
                                Permission Key
                            </div>

                            {/* Policy Columns */}
                            {policies.map(policy => {
                                const config = POLICY_TYPE_CONFIG[policy.type] || POLICY_TYPE_CONFIG.manual;
                                return (
                                    <div key={policy.id} className="w-[140px] p-3 text-center border-r border-border/50 flex flex-col items-center justify-end gap-2 hover:bg-muted/30 transition-colors group relative">
                                        <Badge variant="outline" className={cn("text-[10px] px-1.5 h-5 font-normal capitalize whitespace-nowrap", config.badgeClass)}>
                                            {config.label}
                                        </Badge>
                                        <div className="font-semibold text-sm truncate w-full" title={policy.name}>
                                            {policy.name}
                                        </div>
                                        {policy.pricing && (
                                            <div className="text-xs text-muted-foreground">
                                                ${policy.pricing.amount}/{policy.pricing.cycle.slice(0, 2)}
                                            </div>
                                        )}

                                        {/* Actions overlay */}
                                        <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="h-6 w-6"
                                                onClick={() => router.push(`/wallet-management/groups/${policy.sourceId}`)}
                                            >
                                                <Info className="h-3 w-3" />
                                            </Button>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>

                        {/* Grid Body */}
                        <div className="divide-y divide-border/40 min-w-max">
                            {groupedPermissions.map(([platform, perms]) => (
                                <div key={platform}>
                                    {/* Platform Section Header */}
                                    <div className="sticky left-0 right-0 z-10 bg-muted/30 px-4 py-1.5 text-xs font-bold uppercase tracking-wider text-muted-foreground border-y border-border/50">
                                        {platform} Platform
                                    </div>

                                    {/* Permission Rows */}
                                    {perms.map(perm => (
                                        <div key={perm.id} className="flex hover:bg-muted/10 transition-colors group/row">
                                            {/* Permission Name Column (Sticky Left) */}
                                            <div className="sticky left-0 z-10 w-[350px] bg-card group-hover/row:bg-muted/10 p-3 border-r border-border/50 flex flex-col justify-center">
                                                <div className="flex items-center gap-2 mb-1">
                                                    <code className="text-xs font-mono font-semibold text-primary/80 bg-primary/5 px-1.5 py-0.5 rounded break-all">
                                                        {perm.permission}
                                                    </code>
                                                    {perm.is_system && (
                                                        <Badge variant="secondary" className="text-[9px] h-4 px-1">Sys</Badge>
                                                    )}
                                                </div>
                                                <div className="text-xs text-muted-foreground line-clamp-1" title={perm.description || ''}>
                                                    {perm.description || 'No description'}
                                                </div>
                                            </div>

                                            {/* Checkbox Cells */}
                                            {policies.map(policy => {
                                                const hasPermission = policy.permissions.includes(perm.permission);
                                                const isUpdatingCell = isUpdating[`${policy.id}-${perm.permission}`];
                                                const isSystemProtected = policy.isSystemGroup && perm.is_system; // Example lock logic

                                                return (
                                                    <div
                                                        key={`${policy.id}-${perm.id}`}
                                                        className={cn(
                                                            "w-[140px] border-r border-border/50 flex items-center justify-center p-2 relative",
                                                            hasPermission ? "bg-primary/5" : ""
                                                        )}
                                                    >
                                                        <div
                                                            className={cn(
                                                                "h-8 w-8 rounded-md flex items-center justify-center transition-all cursor-pointer hover:bg-primary/10",
                                                                hasPermission ? "text-primary" : "text-muted-foreground/20 hover:text-muted-foreground/50",
                                                                isUpdatingCell && "opacity-50 cursor-wait",
                                                                isSystemProtected && "opacity-50 cursor-not-allowed"
                                                            )}
                                                            onClick={() => !isUpdatingCell && !isSystemProtected && togglePermission(policy, perm.permission)}
                                                        >
                                                            {hasPermission ? (
                                                                <Check className="h-5 w-5" />
                                                            ) : (
                                                                <div className="h-1.5 w-1.5 rounded-full bg-current" />
                                                            )}
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    ))}
                                </div>
                            ))}

                            {groupedPermissions.length === 0 && (
                                <div className="p-8 text-center text-muted-foreground">
                                    No permissions found matching "{search}"
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            <div className="flex justify-between items-center text-xs text-muted-foreground px-2">
                <span>Showing {permissions.length} permissions across {policies.length} policies</span>
                <div className="flex gap-4">
                    <div className="flex items-center gap-1.5">
                        <div className="h-2 w-2 rounded-full bg-primary" />
                        Active
                    </div>
                    <div className="flex items-center gap-1.5">
                        <div className="h-2 w-2 rounded-full bg-muted-foreground/30" />
                        Inactive
                    </div>
                </div>
            </div>
        </div>
    );
}
