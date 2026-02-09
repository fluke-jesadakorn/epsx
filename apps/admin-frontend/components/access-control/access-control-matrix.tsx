'use client';

import { Skeleton } from '@/components/ui/skeleton';
import { cn } from '@/lib/utils';

import { MatrixGrid } from './matrix/matrix-grid';
import { MatrixHeader } from './matrix/matrix-header';
import { type AccessControlMatrixProps } from './matrix/types';
import { useAccessControlMatrix } from './matrix/use-matrix-logic';

export function AccessControlMatrix({ className }: AccessControlMatrixProps) {
    const {
        policies,
        permissions,
        isLoading,
        search,
        setSearch,
        isUpdating,
        groupedPermissions,
        togglePermission,
    } = useAccessControlMatrix();

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
        <div className={cn('space-y-4', className)}>
            <MatrixHeader search={search} setSearch={setSearch} />

            <MatrixGrid
                policies={policies}
                groupedPermissions={groupedPermissions}
                isUpdating={isUpdating}
                togglePermission={togglePermission}
                search={search}
            />

            <div className="flex justify-between items-center text-xs text-muted-foreground px-2">
                <span>
                    Showing {permissions.length} permissions across {policies.length}{' '}
                    policies
                </span>
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
