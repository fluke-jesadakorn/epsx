import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { PermissionManagementDashboard } from '@/components/admin/PermissionManagementDashboard';

export default function UserPermissionsPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <div className="space-y-6">
          <PermissionManagementDashboard />
        </div>
      </AdminLayout>
    </AdminGuard>
  );
}