import { AdminLayout } from '@/components/layout/AdminLayout';
import { RoleManagementDashboard } from '@/components/admin/RoleManagementDashboard';

export default async function UserRolesPage() {
  return (
    <AdminLayout>
      <div className="space-y-6">
        <RoleManagementDashboard />
      </div>
    </AdminLayout>
  );
}