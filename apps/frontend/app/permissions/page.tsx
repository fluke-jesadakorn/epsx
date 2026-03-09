'use client';

import { GlobalAuthGuard } from '@/components/auth/global-auth-guard';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/lib/auth';
import { useApiClient } from '@/shared/hooks/use-api-client';
import { Shield } from 'lucide-react';
import {
  AnalyticsContent,
  HistoryContent,
  PageHeader,
  PermissionList,
  PermissionStats,
  TabNavigation,
  UserInfoCard,
} from './permissions-sections';
import { usePermissionsPage } from './use-permissions-page';

export default function PermissionsPage() {
  const { user } = useAuth();
  const { base } = useApiClient({ platform: 'frontend' });

  const {
    activeTab,
    setActiveTab,
    timestampedPermissions,
    filteredPermissions,
    analytics,
    history,
    isExporting,
    permissionDefinitions,
    permissionStatus,
    handleExport,
  } = usePermissionsPage({ base });

  return (
    <div className="container mx-auto p-6">
      <GlobalAuthGuard title="My Permissions">
        <div className="max-w-6xl mx-auto">
          <PageHeader
            isExporting={isExporting}
            onExport={() => void handleExport('json')}
            onRefresh={() => window.location.reload()}
          />

          <PermissionStats permissions={timestampedPermissions} />

          <UserInfoCard
            walletAddress={user?.wallet_address}
            permissionsCount={permissionStatus?.total_permissions ?? 0}
          />

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Shield className="h-5 w-5" />
                <span>Permission Details</span>
              </CardTitle>
              <CardDescription>
                All permissions including embedded timestamp permissions with expiry
                information
              </CardDescription>

              <TabNavigation
                activeTab={activeTab}
                timestampedPermissions={timestampedPermissions}
                onTabChange={(tab) => setActiveTab(tab as typeof activeTab)}
              />
            </CardHeader>

            <CardContent>
              {activeTab === 'analytics' && (
                <AnalyticsContent analytics={analytics} />
              )}

              {activeTab === 'history' && <HistoryContent history={history} />}

              {activeTab !== 'analytics' && activeTab !== 'history' && (
                <PermissionList
                  permissions={filteredPermissions}
                  definitions={permissionDefinitions}
                  activeTab={activeTab}
                />
              )}
            </CardContent>
          </Card>
        </div>
      </GlobalAuthGuard>
    </div>
  );
}
