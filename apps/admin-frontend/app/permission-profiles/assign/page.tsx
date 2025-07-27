import { SSRAdminGuard } from '@epsx/auth-shared/server';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { PermissionProfileAssignmentDashboard } from '@/components/admin/PermissionProfileAssignmentDashboard';

export default async function PermissionProfileAssignmentPage() {
  return (
    <SSRAdminGuard>
      <AdminLayout>
        <PermissionProfileAssignmentDashboard />
      </AdminLayout>
    </SSRAdminGuard>
  );
}