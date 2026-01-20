'use client';

import { Settings } from 'lucide-react';

import { SettingsDashboard } from '@/components/admin/SettingsDashboard';
import { PageHeader, PageLayout } from '@/components/shared';

export const dynamic = 'force-dynamic';

/**
 * Settings Page
 * Uses unified page components for consistent design
 */
export default function SettingsPage() {
  return (
    <PageLayout maxWidth="6xl">
      <PageHeader
        title="Settings"
        subtitle="Configure system settings, notifications, and security options"
        icon={Settings}
        gradient="default"
      />

      <SettingsDashboard />
    </PageLayout>
  );
}
