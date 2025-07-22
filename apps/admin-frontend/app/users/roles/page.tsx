import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { RoleManagementDashboard } from '@/components/admin/RoleManagementDashboard';

export default function UserRolesPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <div className="space-y-6">
          <RoleManagementDashboard />
        </div>
      </AdminLayout>
    </AdminGuard>
  );
}