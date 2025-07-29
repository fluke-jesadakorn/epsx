import { AdminLayout } from '@/components/layout/AdminLayout';
import { PermissionProfileAssignmentDashboard } from '@/components/admin/PermissionProfileAssignmentDashboard';

export default async function PermissionProfileAssignmentPage() {
  return (
    <AdminLayout>
      <PermissionProfileAssignmentDashboard />
    </AdminLayout>
  );
}