'use client';

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Badge } from '@/components/ui/badge';
import type { AuthUser } from '@/lib/server-actions';
import { createPlansClient } from '@/shared/api/plans';
import { UnifiedApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';
import { APIDocumentation } from './APIDocumentation';
import { APIKeyManager } from './APIKeyManager';
import { UsageMonitor } from './UsageMonitor';

interface UserGroupData {
  groups: Array<{
    id: string;
    name: string;
    slug: string;
    description: string;
    group_type: string;
    permissions: string[];
    expires_at: string | null;
    rate_limit_per_minute: number | null;
    rate_limit_per_day: number | null;
    assigned_at: string;
  }>;
  total_api_keys: number;
  total_requests: number;
}

interface DeveloperAPIClientProps {
  currentUser: AuthUser;
}

export function DeveloperAPIClient({ currentUser }: DeveloperAPIClientProps) {
  const [activeTab, setActiveTab] = useState('keys');
  const [userGroupData, setUserGroupData] = useState<UserGroupData | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Reusable function to fetch user groups/stats
  const fetchUserGroups = async () => {
    try {
      const client = new UnifiedApiClient({
        baseURL: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
        platform: 'frontend',
      });
      const plansClient = createPlansClient(client);
      const response = await plansClient.getMyGroups();
      if (response.success && response.data) {
        setUserGroupData(response.data as UserGroupData);
      }
    } catch (error) {
      console.error('Failed to fetch user groups:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // Callback for child components to trigger stats refresh
  const refreshStats = () => {
    fetchUserGroups();
  };

  // Fetch on mount
  useEffect(() => {
    fetchUserGroups();
  }, []);

  // Derive display values from user data (real data only, no hardcoded fallbacks)
  const hasGroups = userGroupData?.groups && userGroupData.groups.length > 0;
  const primaryGroup = userGroupData?.groups?.[0];
  const hasApiKeys = (userGroupData?.total_api_keys ?? 0) > 0;

  // Access level based on actual API keys/groups, not hardcoded role
  const accessLevel = hasApiKeys || hasGroups ? 'Active' : 'No API Keys';

  // Get rate limits from user's actual groups only
  const rateLimit = primaryGroup?.rate_limit_per_minute
    ? `${primaryGroup.rate_limit_per_minute}/min`
    : (hasApiKeys ? 'Default' : 'N/A');

  // Get earliest expiry from groups
  const getEarliestExpiry = () => {
    if (!userGroupData?.groups?.length) return null;
    const expiringGroups = userGroupData.groups.filter(g => g.expires_at);
    if (!expiringGroups.length) return null;
    const dates = expiringGroups.map(g => new Date(g.expires_at!));
    return new Date(Math.min(...dates.map(d => d.getTime())));
  };
  const earliestExpiry = getEarliestExpiry();

  return (
    <div className="space-y-8">
      {/* Quick Stats */}
      <div className="grid md:grid-cols-4 gap-4">
        {/* API Access Status */}
        <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
          <div className="absolute inset-0 bg-gradient-to-br from-emerald-500/10 to-teal-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
          <div className="relative p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-500 flex items-center justify-center shadow-lg shadow-emerald-500/25">
                <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">API Access</span>
            </div>
            <div className={`text-2xl font-bold mb-2 ${accessLevel === 'Active' ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'}`}>
              {accessLevel}
            </div>
            {hasGroups && (
              <div className="flex flex-wrap gap-1">
                {userGroupData.groups.slice(0, 2).map(g => (
                  <Badge key={g.id} variant="outline" className="text-xs bg-white/50 dark:bg-gray-900/50">
                    {g.name}
                  </Badge>
                ))}
                {userGroupData.groups.length > 2 && (
                  <Badge variant="outline" className="text-xs bg-white/50 dark:bg-gray-900/50">
                    +{userGroupData.groups.length - 2}
                  </Badge>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Rate Limit */}
        <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-indigo-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
          <div className="relative p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-500 flex items-center justify-center shadow-lg shadow-blue-500/25">
                <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Rate Limit</span>
            </div>
            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400 mb-2">{rateLimit}</div>
            {primaryGroup?.rate_limit_per_day && (
              <div className="text-xs text-gray-500 dark:text-gray-400">{primaryGroup.rate_limit_per_day.toLocaleString()}/day</div>
            )}
          </div>
        </div>

        {/* Usage Stats */}
        <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
          <div className="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
          <div className="relative p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center shadow-lg shadow-purple-500/25">
                <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Total Usage</span>
            </div>
            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400 mb-2">
              {isLoading ? '...' : ((userGroupData?.total_requests ?? 0).toLocaleString())}
            </div>
            <div className="text-xs text-gray-500 dark:text-gray-400">
              {userGroupData?.total_api_keys || 0} API Key{(userGroupData?.total_api_keys || 0) !== 1 ? 's' : ''}
            </div>
          </div>
        </div>

        {/* Expiry */}
        <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
          <div className="absolute inset-0 bg-gradient-to-br from-amber-500/10 to-orange-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
          <div className="relative p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-amber-500 to-orange-500 flex items-center justify-center shadow-lg shadow-amber-500/25">
                <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Expires</span>
            </div>
            {earliestExpiry ? (
              <>
                <div className={`text-2xl font-bold mb-2 ${earliestExpiry.getTime() - Date.now() < 7 * 24 * 60 * 60 * 1000
                  ? 'text-red-600 dark:text-red-400'
                  : 'text-gray-700 dark:text-gray-200'
                  }`}>
                  {earliestExpiry.toLocaleDateString()}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  {Math.ceil((earliestExpiry.getTime() - Date.now()) / (1000 * 60 * 60 * 24))} days left
                </div>
              </>
            ) : (
              <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400 mb-2">Never</div>
            )}
          </div>
        </div>
      </div>

      {/* Main Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl">
          {/* Tab Header */}
          <div className="px-4 pt-4 pb-0 border-b border-gray-200/50 dark:border-gray-700/50">
            <TabsList className="inline-flex gap-2 p-1 bg-gray-100/80 dark:bg-gray-800/80 rounded-xl backdrop-blur-sm">
              <TabsTrigger
                value="keys"
                className="relative px-5 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 data-[state=active]:bg-white data-[state=active]:dark:bg-gray-700 data-[state=active]:shadow-lg data-[state=active]:text-amber-600 data-[state=active]:dark:text-amber-400 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
              >
                <div className="flex items-center gap-2">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                  </svg>
                  <span>API Keys</span>
                </div>
              </TabsTrigger>
              <TabsTrigger
                value="docs"
                className="relative px-5 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 data-[state=active]:bg-white data-[state=active]:dark:bg-gray-700 data-[state=active]:shadow-lg data-[state=active]:text-amber-600 data-[state=active]:dark:text-amber-400 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
              >
                <div className="flex items-center gap-2">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                  </svg>
                  <span>Documentation</span>
                </div>
              </TabsTrigger>
              <TabsTrigger
                value="usage"
                className="relative px-5 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 data-[state=active]:bg-white data-[state=active]:dark:bg-gray-700 data-[state=active]:shadow-lg data-[state=active]:text-amber-600 data-[state=active]:dark:text-amber-400 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
              >
                <div className="flex items-center gap-2">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                  <span>Usage & Monitoring</span>
                </div>
              </TabsTrigger>
            </TabsList>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            <TabsContent value="keys" className="mt-0 focus-visible:outline-none">
              <APIKeyManager currentUser={currentUser} onStatsChange={refreshStats} />
            </TabsContent>
            <TabsContent value="docs" className="mt-0 focus-visible:outline-none">
              <APIDocumentation currentUser={currentUser} />
            </TabsContent>
            <TabsContent value="usage" className="mt-0 focus-visible:outline-none">
              <UsageMonitor currentUser={currentUser} />
            </TabsContent>
          </div>
        </div>
      </Tabs>
    </div>
  );
}