'use client';

import { SettingsDashboard } from '@/components/admin/settings-dashboard';
import { PageHeader, PageLayout } from '@/components/shared';

export const dynamic = 'force-dynamic';

/**
 * Settings Page
 * Uses unified page components for consistent design
 */
export default function SettingsPage() {
  return (
    <PageLayout maxWidth="7xl">
      <PageHeader
        title="Settings Nexus"
        subtitle="Universal configuration interface for security, appearance, and system protocols"
        icon="Settings"
        gradient="warning"
        centered
      />

      <SettingsDashboard />
    </PageLayout>
  );
}
