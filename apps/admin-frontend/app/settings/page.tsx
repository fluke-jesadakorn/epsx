import { SettingsDashboard } from '@/components/admin/SettingsDashboard';
import { SSRAdminGuard } from '@epsx/auth-shared/server';
import { AdminLayout } from '@/components/layout/AdminLayout';

export default async function SettingsPage() {
  return (
    <SSRAdminGuard>
      <AdminLayout>
        <SettingsDashboard />
      </AdminLayout>
    </SSRAdminGuard>
  );
}
