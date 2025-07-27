import { DatabaseDashboard } from '@/components/admin/DatabaseDashboard';
import { SSRAdminGuard } from '@epsx/auth-shared/server';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default async function DatabasePage() {
  return (
    <SSRAdminGuard>
      <AdminLayout>
        <DatabaseDashboard />
      </AdminLayout>
    </SSRAdminGuard>
  );
}
