import { SettingsDashboard } from '@/components/admin/SettingsDashboard';
import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default function SettingsPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <SettingsDashboard />
      </AdminLayout>
    </AdminGuard>
  );
}
