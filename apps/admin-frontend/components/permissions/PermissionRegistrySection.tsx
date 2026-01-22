'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { PermissionDefinitionDto, planMgmt } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { Key, Plus, Search } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

interface PermissionRegistrySectionProps {
    className?: string;
}

export function PermissionRegistrySection({ className }: PermissionRegistrySectionProps) {
    const router = useRouter();
    const [permissions, setPermissions] = useState<PermissionDefinitionDto[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [search, setSearch] = useState('');
    const [platformFilter, setPlatformFilter] = useState<string>('all');

    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            const data = await planMgmt.getPermissionDefinitions();

            let result = data;
            if (search) {
                result = result.filter(p => p.permission.toLowerCase().includes(search.toLowerCase()));
            }
            if (platformFilter !== 'all') {
                result = result.filter(p => p.platform === platformFilter);
            }

            setPermissions(result.slice(0, 50)); // Increased limit
        } catch (err) {
            console.error('Failed to load permissions:', err);
        } finally {
            setIsLoading(false);
        }
    }, [search, platformFilter]);

    useEffect(() => {
        loadData();
    }, [loadData]);

    return (
        <div className={cn("space-y-4", className)}>
            {/* Filter Bar */}
            <div className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-card/50 p-1 rounded-xl">
                <div className="flex items-center gap-2 w-full sm:w-auto flex-1">
                    <div className="relative w-full sm:max-w-md">
                        <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="Search permissions..."
                            className="pl-9 h-10 bg-background/50 border-border/50 focus:bg-background transition-colors"
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                        />
                    </div>
                    <Select value={platformFilter} onValueChange={setPlatformFilter}>
                        <SelectTrigger className="w-[130px] h-10 bg-background/50 border-border/50">
                            <SelectValue placeholder="Platform" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Platforms</SelectItem>
                            <SelectItem value="epsx">EPSX</SelectItem>
                            <SelectItem value="admin">Admin</SelectItem>
                            <SelectItem value="pay">Pay</SelectItem>
                            <SelectItem value="token">Token</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <Button size="sm" variant="default" className="w-full sm:w-auto">
                        <Plus className="h-4 w-4 mr-2" />
                        New Permission
                    </Button>
                </div>
            </div>

            {/* Table Content */}
            <div className="border border-border rounded-xl bg-card overflow-hidden">
                {/* Table Header */}
                <div className="grid grid-cols-12 gap-4 p-4 border-b border-border bg-muted/40 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    <div className="col-span-12 sm:col-span-5">Permission Key</div>
                    <div className="hidden sm:block sm:col-span-5">Description</div>
                    <div className="hidden sm:block sm:col-span-2 text-right">Platform</div>
                </div>

                {isLoading ? (
                    <div className="divide-y divide-border/40">
                        {Array.from({ length: 5 }).map((_, i) => (
                            <div key={i} className="p-4 grid grid-cols-12 gap-4">
                                <div className="col-span-12 sm:col-span-5"><Skeleton className="h-5 w-3/4" /></div>
                                <div className="hidden sm:block sm:col-span-5"><Skeleton className="h-4 w-full" /></div>
                                <div className="hidden sm:block sm:col-span-2"><Skeleton className="h-4 w-1/2 ml-auto" /></div>
                            </div>
                        ))}
                    </div>
                ) : permissions.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-16 text-muted-foreground bg-muted/5">
                        <Key className="h-8 w-8 mb-2 opacity-50" />
                        <p>No permissions found matching your filters</p>
                    </div>
                ) : (
                    <div className="divide-y divide-border/40 max-h-[600px] overflow-y-auto">
                        {permissions.map(perm => (
                            <div key={perm.id} className="grid grid-cols-12 gap-4 p-4 hover:bg-muted/30 transition-colors items-center group">
                                <div className="col-span-12 sm:col-span-5">
                                    <div className="flex items-center gap-2">
                                        <code className="text-sm font-mono font-semibold text-primary/80 bg-primary/5 px-2 py-0.5 rounded border border-primary/10 group-hover:border-primary/20 transition-colors break-all">
                                            {perm.permission}
                                        </code>
                                        {perm.is_system && (
                                            <Badge variant="secondary" className="text-[10px] h-5 px-1.5 bg-zinc-100 dark:bg-zinc-800 text-zinc-500 border-zinc-200">System</Badge>
                                        )}
                                    </div>
                                    {/* Mobile only description */}
                                    <div className="sm:hidden mt-2 text-xs text-muted-foreground line-clamp-2">
                                        {perm.description}
                                    </div>
                                </div>

                                <div className="hidden sm:block sm:col-span-5 text-sm text-foreground/70">
                                    {perm.description || <span className="text-muted-foreground/50 italic">No description provided</span>}
                                </div>

                                <div className="hidden sm:flex sm:col-span-2 justify-end gap-2">
                                    <Badge variant="outline" className="capitalize text-xs font-normal text-muted-foreground">
                                        {perm.platform}
                                    </Badge>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <div className="flex justify-between items-center text-xs text-muted-foreground px-2">
                <span>Showing {permissions.length} records</span>
                <span>Registry v1.2</span>
            </div>
        </div>
    );
}
