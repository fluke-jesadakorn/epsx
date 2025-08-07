import { PermissionManagementDashboard } from '@/components/admin/PermissionManagementDashboard';

export default async function UserPermissionsPage() {
  return (
    <div className="space-y-6">
      <PermissionManagementDashboard />
    </div>
  );
}