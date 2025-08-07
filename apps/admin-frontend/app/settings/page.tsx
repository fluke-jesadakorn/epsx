import { SettingsDashboard } from '@/components/admin/SettingsDashboard';
import {
  getSystemConfig,
  getSettingsByCategory,
  getFeatureFlags,
  getEnvironmentConfig
} from '@epsx/server-actions';

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
