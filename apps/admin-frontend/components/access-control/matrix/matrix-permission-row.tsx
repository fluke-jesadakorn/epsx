import { Check } from 'lucide-react';

import { type AccessPolicy } from '@/components/access-control/types';
import { Badge } from '@/components/ui/badge';
import { type PermissionDefinitionDto } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';

interface MatrixPermissionRowProps {
    platform: string;
    perms: PermissionDefinitionDto[];
    policies: AccessPolicy[];
    isUpdating: Record<string, boolean>;
    togglePermission: (policy: AccessPolicy, permissionKey: string) => void;
}

export function MatrixPermissionRow({
    platform,
    perms,
    policies,
    isUpdating,
    togglePermission,
}: MatrixPermissionRowProps) {
    return (
        <div>
            {/* Platform Section Header */}
            <div className="sticky left-0 right-0 z-10 bg-muted/30 px-4 py-1.5 text-xs font-bold uppercase tracking-wider text-muted-foreground border-y border-border/50">
                {platform} Platform
            </div>

            {/* Permission Rows */}
            {perms.map((perm) => (
                <div
                    key={perm.id}
                    className="flex hover:bg-muted/10 transition-colors group/row"
                >
                    {/* Permission Name Column (Sticky Left) */}
                    <div className="sticky left-0 z-10 w-[350px] bg-card group-hover/row:bg-muted/10 p-3 border-r border-border/50 flex flex-col justify-center">
                        <div className="flex items-center gap-2 mb-1">
                            <code className="text-xs font-mono font-semibold text-primary/80 bg-primary/5 px-1.5 py-0.5 rounded break-all">
                                {perm.permission}
                            </code>
                            {perm.is_system && (
                                <Badge variant="secondary" className="text-[9px] h-4 px-1">
                                    Sys
                                </Badge>
                            )}
                        </div>
                        <div
                            className="text-xs text-muted-foreground line-clamp-1"
                            title={perm.description ?? ''}
                        >
                            {perm.description ?? 'No description'}
                        </div>
                    </div>

                    {/* Checkbox Cells */}
                    {policies.map((policy) => {
                        const hasPermission = policy.permissions.includes(perm.permission);
                        const isUpdatingCell =
                            isUpdating[`${policy.id}-${perm.permission}`];
                        const isSystemProtected =
                            Boolean(policy.isSystemGroup) && perm.is_system;

                        return (
                            <div
                                key={`${policy.id}-${perm.id}`}
                                className={cn(
                                    'w-[140px] border-r border-border/50 flex items-center justify-center p-2 relative',
                                    hasPermission ? 'bg-primary/5' : ''
                                )}
                            >
                                <div
                                    className={cn(
                                        'h-8 w-8 rounded-md flex items-center justify-center transition-all cursor-pointer hover:bg-primary/10',
                                        hasPermission
                                            ? 'text-primary'
                                            : 'text-muted-foreground/20 hover:text-muted-foreground/50',
                                        isUpdatingCell === true ? 'opacity-50 cursor-wait' : '',
                                        isSystemProtected === true
                                            ? 'opacity-50 cursor-not-allowed'
                                            : ''
                                    )}
                                    onClick={() => {
                                        if (isUpdatingCell !== true && isSystemProtected !== true) {
                                            togglePermission(policy, perm.permission);
                                        }
                                    }}
                                    onKeyDown={(e) => {
                                        if (
                                            (e.key === 'Enter' || e.key === ' ') &&
                                            isUpdatingCell !== true &&
                                            isSystemProtected !== true
                                        ) {
                                            togglePermission(policy, perm.permission);
                                        }
                                    }}
                                    role="button"
                                    tabIndex={0}
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
    );
}
