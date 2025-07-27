'use client';

import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { IAMDashboard } from '@/components/admin/IAMDashboard';

export default function IAMPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <IAMDashboard />
      </AdminLayout>
    </AdminGuard>
  );
}
