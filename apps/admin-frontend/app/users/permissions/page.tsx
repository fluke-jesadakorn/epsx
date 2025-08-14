import { PermissionManagementDashboard } from '@/components/admin/PermissionMgmtDash';

export default async function UserPermissionsPage() {
  return (
    <div className="space-y-6">
      <PermissionManagementDashboard />
    </div>
  );
}