'use client';

import { PolicyCard } from '@/components/access-control/policy-card';
import type { AccessPolicy } from '@/components/access-control/types';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import { cn } from '@/lib/utils';
import { Plus, Search, Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';

interface AccessPolicySectionProps {
    className?: string;
}

export function AccessPolicySection({ className }: AccessPolicySectionProps) {
    const router = useRouter();
    const [policies, setPolicies] = useState<AccessPolicy[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [search, setSearch] = useState('');
    const [typeFilter, setTypeFilter] = useState<string>('all');

    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            const data = await accessPolicyClient.getPolicies();

            let result = data;
            // Client-side filter
            if (search) {
                result = result.filter(p => p.name.toLowerCase().includes(search.toLowerCase()));
            }
            if (typeFilter !== 'all') {
                result = result.filter(p => p.type === typeFilter);
            }

            setPolicies(result.slice(0, 12)); // Increase limit for full view
        } catch (err) {
            console.error('Failed to load policies:', err);
        } finally {
            setIsLoading(false);
        }
    }, [search, typeFilter]);

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
                            placeholder="Search policies..."
                            className="pl-9 h-10 bg-background/50 border-border/50 focus:bg-background transition-colors"
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                        />
                    </div>
                    <Select value={typeFilter} onValueChange={setTypeFilter}>
                        <SelectTrigger className="w-[130px] h-10 bg-background/50 border-border/50">
                            <SelectValue placeholder="Type" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Types</SelectItem>
                            <SelectItem value="subscription">Plan</SelectItem>
                            <SelectItem value="manual">Group</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <div className="flex items-center gap-2 w-full sm:w-auto">
                    <Button onClick={() => router.push('/subscriptions/plans/new')} className="w-full sm:w-auto">
                        <Plus className="h-4 w-4 mr-2" />
                        New Policy
                    </Button>
                </div>
            </div>

            {/* Content - Responsive Grid */}
            <div className="min-h-[400px]">
                {isLoading ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {[1, 2, 3, 4, 5, 6].map(i => <Skeleton key={i} className="h-64 rounded-xl" />)}
                    </div>
                ) : policies.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-20 text-center border border-dashed border-border rounded-xl bg-muted/5">
                        <Shield className="h-10 w-10 text-muted-foreground/50 mb-4" />
                        <h3 className="text-lg font-semibold">No policies found</h3>
                        <p className="text-sm text-muted-foreground mt-2 mb-4">
                            Create a new access policy to get started.
                        </p>
                        <Button variant="outline" onClick={() => router.push('/subscriptions/plans/new')}>
                            Create Policy
                        </Button>
                    </div>
                ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-in fade-in-50 duration-500">
                        {policies.map(policy => (
                            <PolicyCard
                                key={policy.id}
                                policy={policy}
                            />
                        ))}
                    </div>
                )}
            </div>

            {/* Footer */}
            {policies.length > 0 && (
                <div className="flex justify-center pt-4 border-t border-border/40">
                    <Button variant="ghost" className="text-sm text-muted-foreground hover:text-primary" onClick={() => router.push('/subscriptions')}>
                        View All Policies →
                    </Button>
                </div>
            )}
        </div>
    );
}
