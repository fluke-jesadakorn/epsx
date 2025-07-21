'use client';

import { IAMGuard } from '@/components/auth/IAMGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { ToastProvider } from '@/components/ui/toast';
import { IAMDashboardContent } from '../../components/iam/IAMDashboardContent';

export default function IAMPage() {
  return (
    <IAMGuard>
      <AdminLayout>
        <ToastProvider>
          <IAMDashboardContent />
        </ToastProvider>
      </AdminLayout>
    </IAMGuard>
  );
}
