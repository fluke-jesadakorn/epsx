import { AdminLayout } from '@/components/layout/AdminLayout';
import { UserManagementList } from '@/components/admin/UserManagementList';
import { getAdminUsers } from '@epsx/server-actions';

export default async function UsersPage() {
  // Fetch users server-side
  const users = await getAdminUsers();

  return (
    <AdminLayout>
      <div className="space-y-6">
        <UserManagementList initialUsers={users} />
      </div>
    </AdminLayout>
  );
}
