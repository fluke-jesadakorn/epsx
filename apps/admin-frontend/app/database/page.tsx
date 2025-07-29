import { DatabaseDashboard } from '@/components/admin/DatabaseDashboard';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default async function DatabasePage() {
  return (
    <AdminLayout>
      <DatabaseDashboard />
    </AdminLayout>
  );
}
