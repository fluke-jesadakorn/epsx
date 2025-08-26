// ============================================================================
// SIMPLE PERMISSION PROFILE MANAGER STUB
// ============================================================================
// Stub - using simple roles now

'use client';

import { AdminOnly } from '@/components/guards/FeatureGuard';

export function PermissionProfileManager() {
  return (
    <AdminOnly>
      <div className="p-6">
        <h2 className="text-xl font-semibold mb-4">Role Management</h2>
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <p className="text-blue-800">
            System now uses simple roles: <strong>Admin</strong>, <strong>User</strong>, <strong>Guest</strong>
          </p>
          <p className="text-blue-600 text-sm mt-2">
            Permission profiles have been replaced with the unified role system.
          </p>
        </div>
      </div>
    </AdminOnly>
  );
}

export default PermissionProfileManager;