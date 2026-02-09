'use client';

import { Shield, Trash2 } from 'lucide-react';

import { PolicyCard } from '@/components/access-control/policy-card';
import type { AccessPolicy } from '@/components/access-control/types';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';

interface PolicyListProps {
    isRefreshing: boolean;
    policiesCount: number;
    filteredPolicies: AccessPolicy[];
    selectedPolicies: Set<string>;
    onSelect: (policyId: string, selected: boolean) => void;
    onDeleteConfirm: (policy: AccessPolicy) => void;
}

export function PolicyList({
    isRefreshing,
    policiesCount,
    filteredPolicies,
    selectedPolicies,
    onSelect,
    onDeleteConfirm,
}: PolicyListProps) {
    if (isRefreshing && policiesCount === 0) {
        return (
            <div className="space-y-3">
                {Array.from({ length: 5 }).map((_, i) => (
                    <div key={`skeleton-${i}`} className="rounded-2xl bg-card border border-border p-6 animate-pulse">
                        <div className="flex items-center gap-4">
                            <Skeleton className="h-12 w-12 rounded-xl" />
                            <div className="space-y-2 flex-1">
                                <Skeleton className="h-4 w-40" />
                                <Skeleton className="h-3 w-32" />
                            </div>
                            <Skeleton className="h-9 w-20" />
                        </div>
                    </div>
                ))}
            </div>
        );
    }

    if (filteredPolicies.length === 0) {
        return (
            <div className="text-center py-12 text-muted-foreground">
                <Shield className="h-12 w-12 mx-auto mb-4 opacity-30" />
                <p className="font-medium">No policies found</p>
                <p className="text-sm mt-1">Try adjusting your filters or create a new policy</p>
            </div>
        );
    }

    return (
        <div className="space-y-3">
            {filteredPolicies.map((policy) => (
                <PolicyCard
                    key={policy.id}
                    policy={policy}
                    isSelected={selectedPolicies.has(policy.id)}
                    onSelect={(selected) => onSelect(policy.id, selected)}
                    onDelete={policy.isSystemGroup ? undefined : () => onDeleteConfirm(policy)}
                />
            ))}
        </div>
    );
}

interface PolicyDeleteModalProps {
    policy: AccessPolicy;
    onClose: () => void;
    onDelete: () => Promise<void>;
    isDeleting: boolean;
}

export function PolicyDeleteModal({ policy, onClose, onDelete, isDeleting }: PolicyDeleteModalProps) {
    return (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
            <div className="bg-card rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl border border-border">
                <div className="flex items-center gap-3 mb-4">
                    <div className="p-3 bg-destructive/10 rounded-full">
                        <Trash2 className="w-6 h-6 text-destructive" />
                    </div>
                    <h3 className="text-lg font-semibold text-foreground">
                        Delete Policy
                    </h3>
                </div>
                <p className="text-muted-foreground mb-6">
                    Are you sure you want to delete <strong>&quot;{policy.name}&quot;</strong>?
                    This action cannot be undone.
                    {policy.memberCount > 0 && (
                        <span className="block mt-2 text-amber-600 dark:text-amber-400">
                            ⚠️ This policy has {policy.memberCount} members who will lose access.
                        </span>
                    )}
                </p>
                <div className="flex gap-3">
                    <Button
                        variant="outline"
                        onClick={onClose}
                        className="flex-1"
                    >
                        Cancel
                    </Button>
                    <Button
                        variant="destructive"
                        onClick={() => { void onDelete(); }}
                        disabled={isDeleting}
                        className="flex-1"
                    >
                        {isDeleting ? 'Deleting...' : 'Delete'}
                    </Button>
                </div>
            </div>
        </div>
    );
}
