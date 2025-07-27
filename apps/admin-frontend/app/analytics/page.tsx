import { AnalyticsDashboard } from '@/components/admin/AnalyticsDashboard';
import { SSRAdminGuard } from '@epsx/auth-shared/server';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default async function AnalyticsPage() {
  return (
    <SSRAdminGuard>
      <AdminLayout>
        <AnalyticsDashboard />
      </AdminLayout>
    </SSRAdminGuard>
  );
}
