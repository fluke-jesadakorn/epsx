import { AnalyticsDashboard } from '@/components/admin/AnalyticsDashboard';
import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default function AnalyticsPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <AnalyticsDashboard />
      </AdminLayout>
    </AdminGuard>
  );
}
