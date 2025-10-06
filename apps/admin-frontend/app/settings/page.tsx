import { SettingsDashboard } from '@/components/admin/SettingsDashboard';

// TODO: Replace with direct API calls

// Temporary placeholder functions for migration
const getSystemConfig = async () => ({ config: {} });
const getSettingsByCategory = async (category: string) => ({ settings: [] });
const getFeatureFlags = async () => ({ flags: {} });
const getEnvironmentConfig = async () => ({ env: {} });

export const dynamic = 'force-dynamic';

/**
 *
 */
export default async function SettingsPage() {
  // Fetch settings data server-side
  const [
    systemConfigResult,
    generalSettingsResult,
    notificationSettingsResult,
    securitySettingsResult,
    featureFlagsResult,
    environmentConfigResult
  ] = await Promise.allSettled([
    getSystemConfig(),
    getSettingsByCategory('general'),
    getSettingsByCategory('notifications'),
    getSettingsByCategory('security'),
    getFeatureFlags(),
    getEnvironmentConfig()
  ]);

  const systemConfig = systemConfigResult.status === 'fulfilled' ? systemConfigResult.value : {};
  const generalSettings = generalSettingsResult.status === 'fulfilled' ? generalSettingsResult.value : {};
  const notificationSettings = notificationSettingsResult.status === 'fulfilled' ? notificationSettingsResult.value : {};
  const securitySettings = securitySettingsResult.status === 'fulfilled' ? securitySettingsResult.value : {};
  const featureFlags = featureFlagsResult.status === 'fulfilled' ? featureFlagsResult.value : {};
  const environmentConfig = environmentConfigResult.status === 'fulfilled' ? environmentConfigResult.value : {};

  return (
    <SettingsDashboard 
      initialSystemConfig={systemConfig}
      initialGeneralSettings={generalSettings}
      initialNotificationSettings={notificationSettings}
      initialSecuritySettings={securitySettings}
      initialFeatureFlags={featureFlags}
      initialEnvironmentConfig={environmentConfig}
    />
  );
}
