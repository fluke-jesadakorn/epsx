'use client'

import { PermissionTransferList } from '@/components/plans/Permissiontransfer-list'

interface PlanAdvancedPermissionsProps {
    availablePermissions: string[]
    customPermissions: string[]
    setCustomPermissions: (permissions: string[]) => void
    loadingPermissions: boolean
}

export function PlanAdvancedPermissions({
    availablePermissions,
    customPermissions,
    setCustomPermissions,
    loadingPermissions,
}: PlanAdvancedPermissionsProps) {
    const filteredAvailable = (availablePermissions || []).filter(
        (p) =>
            !p.startsWith('epsx:api:calls:') &&
            !p.startsWith('epsx:rankings:offset:') &&
            !p.startsWith('epsx:analytics:queries:') &&
            !p.startsWith('epsx:export:limit:')
    )

    const systemPermissions = new Set(
        (availablePermissions || []).filter(
            (p) => p.startsWith('system:') || p.startsWith('admin:')
        )
    )

    return (
        <div className="bg-gradient-to-r from-gray-50 to-slate-50 dark:from-gray-900/30 dark:to-slate-900/30 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">
                Advanced Permissions
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                Manage specific permissions directly. API limits configured above are
                automatically handled.
            </p>
            <PermissionTransferList
                available={filteredAvailable}
                selected={customPermissions}
                onChange={setCustomPermissions}
                isLoading={loadingPermissions}
                systemPermissions={systemPermissions}
            />
        </div>
    )
}
