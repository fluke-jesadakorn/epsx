import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { UserManagementList } from '@/components/admin/UserManagementList';

export default function UsersPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <div className="space-y-6">
          <UserManagementList />
        </div>
      </AdminLayout>
    </AdminGuard>
  );
}
