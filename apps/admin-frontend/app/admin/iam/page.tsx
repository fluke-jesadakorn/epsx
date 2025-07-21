'use client';

import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { IAMDashboardNew } from '@/components/admin/IAMDashboardNew';

export default function IAMPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <IAMDashboardNew />
      </AdminLayout>
    </AdminGuard>
  );
}
