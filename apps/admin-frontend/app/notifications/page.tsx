import React from 'react';
import type { Metadata } from 'next';
import { NotificationDashboard } from '@/components/admin/NotificationDashboard';

export const metadata: Metadata = {
  title: 'Notification Management | EPSX Admin',
  description: 'Manage and monitor system notifications across the platform.',
};

export default function NotificationsPage() {
  return (
    <div className="container max-w-7xl mx-auto py-8 px-4">
      <NotificationDashboard />
    </div>
  );
}