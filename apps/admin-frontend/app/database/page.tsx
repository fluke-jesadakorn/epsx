import { DatabaseDashboard } from '@/components/admin/DatabaseDashboard';
import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default function DatabasePage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <DatabaseDashboard />
      </AdminLayout>
    </AdminGuard>
  );
}
