/**
 * Permission Profiles Management Page
 */

import { PermissionProfileManager } from '@/components/permission-profiles/PermissionProfileManager'

export default async function PermissionProfilesPage() {
  // Authentication is handled at the layout level by AdminAuthWrapper

  return (
    <div className="container mx-auto py-6">
      <PermissionProfileManager />
    </div>
  )
}