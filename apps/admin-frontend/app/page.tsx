import { SSRAdminGuard } from '@epsx/auth-shared/server';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { AdminDashboard } from '@/components/admin/AdminDashboard';

export default async function DashboardPage() {
  return (
    <SSRAdminGuard>
      <AdminLayout>
        <AdminDashboard />
      </AdminLayout>
    </SSRAdminGuard>
  );
}
