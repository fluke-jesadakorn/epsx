import { AdminLayout } from '@/components/layout/AdminLayout';
import { AdminDashboard } from '@/components/admin/AdminDashboard';
import { getAdminUsers, getUserStats } from '@epsx/server-actions';

export default async function DashboardPage() {
  // Fetch data server-side
  const [statsResult, usersResult] = await Promise.allSettled([
    getUserStats(),
    getAdminUsers({ limit: 10 })
  ]);

  const stats = statsResult.status === 'fulfilled' ? statsResult.value : null;
  const users = usersResult.status === 'fulfilled' ? usersResult.value : { users: [] };

  return (
    <AdminLayout>
      <AdminDashboard 
        initialStats={stats}
        initialUsers={users.users}
      />
    </AdminLayout>
  );
}
