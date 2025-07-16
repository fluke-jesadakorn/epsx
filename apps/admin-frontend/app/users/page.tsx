import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { AdminUserManagement } from '@/components/admin/AdminUserManagement';

export default function UsersPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <AdminUserManagement />
      </AdminLayout>
    </AdminGuard>
  );
}
