import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { PermissionProfileAssignmentDashboard } from '@/components/admin/PermissionProfileAssignmentDashboard';

export default function PermissionProfileAssignmentPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <PermissionProfileAssignmentDashboard />
      </AdminLayout>
    </AdminGuard>
  );
}