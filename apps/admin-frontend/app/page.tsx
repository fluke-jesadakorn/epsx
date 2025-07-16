import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { AdminDashboard } from '@/components/admin/AdminDashboard';

export default function DashboardPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <AdminDashboard />
      </AdminLayout>
    </AdminGuard>
  );
}
