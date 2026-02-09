import { Info } from 'lucide-react';
import { useRouter } from 'next/navigation';

import { type AccessPolicy, POLICY_TYPE_CONFIG } from '@/components/access-control/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { type PermissionDefinitionDto } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';

import { MatrixPermissionRow } from './matrix-permission-row';

export interface MatrixGridProps {
    policies: AccessPolicy[];
    groupedPermissions: [string, PermissionDefinitionDto[]][];
    isUpdating: Record<string, boolean>;
    togglePermission: (policy: AccessPolicy, permissionKey: string) => void;
    search: string;
}

export function MatrixGrid({
    policies,
    groupedPermissions,
    isUpdating,
    togglePermission,
    search,
}: MatrixGridProps) {
    const router = useRouter();

    return (
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
                        {policies.map((policy) => {
                            const config =
                                POLICY_TYPE_CONFIG[policy.type] ?? POLICY_TYPE_CONFIG.manual;
                            return (
                                <div
                                    key={policy.id}
                                    className="w-[140px] p-3 text-center border-r border-border/50 flex flex-col items-center justify-end gap-2 hover:bg-muted/30 transition-colors group relative"
                                >
                                    <Badge
                                        variant="outline"
                                        className={cn(
                                            'text-[10px] px-1.5 h-5 font-normal capitalize whitespace-nowrap',
                                            config.badgeClass
                                        )}
                                    >
                                        {config.label}
                                    </Badge>
                                    <div
                                        className="font-semibold text-sm truncate w-full"
                                        title={policy.name}
                                    >
                                        {policy.name}
                                    </div>
                                    {policy.pricing && (
                                        <div className="text-xs text-muted-foreground">
                                            ${policy.pricing.amount}/
                                            {policy.pricing.cycle.slice(0, 2)}
                                        </div>
                                    )}

                                    {/* Actions overlay */}
                                    <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            className="h-6 w-6"
                                            onClick={() => {
                                                router.push(
                                                    `/wallet-management/groups/${policy.sourceId}`
                                                );
                                            }}
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
                            <MatrixPermissionRow
                                key={platform}
                                platform={platform}
                                perms={perms}
                                policies={policies}
                                isUpdating={isUpdating}
                                togglePermission={togglePermission}
                            />
                        ))}

                        {groupedPermissions.length === 0 && (
                            <div className="p-8 text-center text-muted-foreground">
                                No permissions found matching &quot;{search}&quot;
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
