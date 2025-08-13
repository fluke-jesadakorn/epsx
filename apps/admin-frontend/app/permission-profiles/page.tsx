/**
 * Permission Profiles Management Page
 */

import { requireAdminAuth } from '@/lib/auth/server-auth'
import { PermissionProfileManager } from '@/components/permission-profiles/PermissionProfileManager'

export default async function PermissionProfilesPage() {
  await requireAdminAuth()

  return (
    <div className="container mx-auto py-6">
      <PermissionProfileManager />
    </div>
  )
}