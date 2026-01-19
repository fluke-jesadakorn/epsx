'use client'


import { GroupHub } from '@/components/group/GroupHub'

export const dynamic = 'force-dynamic'

/**
 * Group and Permission Management Page
 * Uses the same card-based UX/UI as the Wallet Management page
 */
export default function GroupAndPermissionPage() {
  return (
    <div className="p-3 sm:p-6 space-y-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 to-pink-600 bg-clip-text text-transparent flex items-center gap-2">
          <span>🔑</span> Group & Permission Hub
        </h1>
        <p className="text-muted-foreground mt-2">
          Manage permission groups, assignments, and wallet memberships
        </p>
      </div>

      {/* Group Hub */}
      <GroupHub />
    </div>
  )
}
