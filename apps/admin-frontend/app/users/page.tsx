import { SSRAdminGuard } from '@epsx/auth-shared/server';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { UserManagementList } from '@/components/admin/UserManagementList';

export default async function UsersPage() {
  return (
    <SSRAdminGuard>
      <AdminLayout>
        <div className="space-y-6">
          <UserManagementList />
        </div>
      </AdminLayout>
    </SSRAdminGuard>
  );
}
