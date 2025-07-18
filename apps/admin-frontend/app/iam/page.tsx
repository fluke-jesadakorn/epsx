'use client';

import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { PermissionManagement } from '@/components/admin/PermissionManagement';

export default function IAMPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <PermissionManagement />
      </AdminLayout>
    </AdminGuard>
  );
}
