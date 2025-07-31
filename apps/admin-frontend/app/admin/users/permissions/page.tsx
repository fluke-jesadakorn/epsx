import { AdminLayout } from '@/components/layout/AdminLayout';
import { PermissionManagementDashboard } from '@/components/admin/PermissionManagementDashboard';

export default async function UserPermissionsPage() {
  return (
    <AdminLayout>
      <div className="space-y-6">
        <PermissionManagementDashboard />
      </div>
    </AdminLayout>
  );
}