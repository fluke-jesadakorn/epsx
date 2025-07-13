'use client';

import { Suspense } from 'react';

import { AuthProviders } from '@/components/features/settings/AuthProviders';
import { ProfileSettings } from '@/components/features/settings/ProfileSettings';
import { UserLevelDisplay } from '@/components/features/settings/UserLevelDisplay';
import { LevelBenefitsComparison } from '@/components/features/settings/LevelBenefitsComparison';
import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/auth-context';
import { status } from '@/services/pay';
import { useState, useEffect } from 'react';
import type { UserLevelType } from '@/app/constants/packages';

const SettingsSkeleton = () => (
  <div className="max-w-4xl mx-auto p-4 sm:p-6">
    {/* Header Skeleton */}
    <div className="mb-6 sm:mb-8 text-center">
      <div className="h-8 sm:h-10 md:h-12 bg-gray-200 rounded-md animate-pulse w-48 mx-auto mb-2 dark:bg-gray-700"></div>
      <div className="h-4 bg-gray-200 rounded-md animate-pulse w-64 mx-auto dark:bg-gray-700"></div>
    </div>

    <div className="flex flex-col gap-6 sm:gap-8">
      {/* User Level Display Skeleton */}
      <div className="border rounded-lg shadow-sm p-4 sm:p-6">
        <div className="flex flex-col sm:flex-row items-center gap-4 mb-4">
          <div className="w-16 h-16 sm:w-20 sm:h-20 bg-gray-200 rounded-full animate-pulse dark:bg-gray-700"></div>
          <div className="flex-1 space-y-2 text-center sm:text-left">
            <div className="h-6 bg-gray-200 rounded-md animate-pulse w-32 mx-auto sm:mx-0 dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse w-24 mx-auto sm:mx-0 dark:bg-gray-700"></div>
          </div>
        </div>
        <div className="space-y-3">
          <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
          <div className="h-4 bg-gray-200 rounded-md animate-pulse w-3/4 dark:bg-gray-700"></div>
          <div className="h-2 bg-gray-200 rounded-full animate-pulse dark:bg-gray-700"></div>
        </div>
      </div>

      {/* Profile Settings Skeleton */}
      <div className="border rounded-lg shadow-sm">
        <div className="p-3 sm:p-4 border-b">
          <div className="h-6 bg-gray-200 rounded-md animate-pulse w-1/4 dark:bg-gray-700"></div>
        </div>
        <div className="p-3 sm:p-4 space-y-4">
          <div className="flex flex-col sm:flex-row items-center gap-3 sm:gap-4 p-3 sm:p-4 bg-gray-50 rounded-lg dark:bg-gray-800">
            <div className="w-16 h-16 sm:w-20 sm:h-20 bg-gray-200 rounded-full animate-pulse dark:bg-gray-700"></div>
            <div className="flex-1 space-y-2 text-center sm:text-left">
              <div className="h-5 bg-gray-200 rounded-md animate-pulse w-32 mx-auto sm:mx-0 dark:bg-gray-700"></div>
              <div className="h-4 bg-gray-200 rounded-md animate-pulse w-40 mx-auto sm:mx-0 dark:bg-gray-700"></div>
            </div>
          </div>
          <div className="h-10 bg-gray-200 rounded-md animate-pulse w-full sm:w-32 dark:bg-gray-700"></div>
        </div>
      </div>

      {/* Authentication Providers Card Skeleton */}
      <div className="border rounded-lg shadow-sm">
        <div className="p-3 sm:p-4 border-b">
          <div className="h-6 bg-gray-200 rounded-md animate-pulse w-1/3 dark:bg-gray-700"></div>
        </div>
        <div className="p-3 sm:p-4 space-y-4">
          <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
          <div className="h-4 bg-gray-200 rounded-md animate-pulse w-5/6 dark:bg-gray-700"></div>
          <div className="h-10 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
        </div>
      </div>

      {/* Payment Settings Card Skeleton */}
      <div className="border rounded-lg shadow-sm">
        <div className="p-3 sm:p-4 border-b">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div className="flex items-center gap-2 flex-1">
              <div className="w-8 h-8 bg-gray-200 rounded-full animate-pulse dark:bg-gray-700"></div>
              <div className="space-y-2 flex-1">
                <div className="h-5 bg-gray-200 rounded-md animate-pulse w-32 dark:bg-gray-700"></div>
                <div className="h-4 bg-gray-200 rounded-md animate-pulse w-48 dark:bg-gray-700"></div>
              </div>
            </div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse w-full sm:w-32 dark:bg-gray-700"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
);

export default function SettingsPage() {
  const router = useRouter();
  const { user, loading } = useAuth();
  const [userLevel, setUserLevel] = useState<UserLevelType>('BASIC');
  const [isLoadingUserData, setIsLoadingUserData] = useState(true);

  useEffect(() => {
    const fetchUserData = async () => {
      if (!user) {
        setIsLoadingUserData(false);
        return;
      }

      try {
        const userStatus = await status();
        setUserLevel(userStatus.level as UserLevelType);
        // In a real app, you'd fetch payment count from transaction history
      } catch (error) {
        console.error('Failed to fetch user data:', error);
        setUserLevel('BASIC');
      } finally {
        setIsLoadingUserData(false);
      }
    };

    fetchUserData();
  }, [user]);

  if (loading) {
    return <SettingsSkeleton />;
  }

  return (
    <Suspense fallback={<SettingsSkeleton />}>
      <div className="max-w-4xl mx-auto p-4 sm:p-6">
        {/* Page Header */}
        <div className="mb-6 sm:mb-8 text-center">
          <h1 className="text-2xl sm:text-3xl md:text-4xl font-bold tracking-tight text-primary mb-2 flex flex-col sm:flex-row items-center justify-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6 sm:h-8 sm:w-8 text-primary"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            Settings
          </h1>
          <div className="overflow-hidden whitespace-nowrap max-w-xl mx-auto px-4">
            <span className="inline-block animate-marquee text-muted-foreground text-sm sm:text-base">
              Manage your profile, track your progress, and unlock premium
              features.
            </span>
          </div>
        </div>
        <div className="flex flex-col gap-6 sm:gap-8">
          {/* User Level Display - New Enhanced Section */}
          <UserLevelDisplay className="animate-fade-in" />

          {/* Level Benefits Comparison */}
          {!isLoadingUserData && (
            <LevelBenefitsComparison
              currentLevel={userLevel}
              className="animate-fade-in-delayed-2"
            />
          )}
          <ProfileSettings />

          {/* Authentication Providers */}
          <Card className="transition-shadow hover:shadow-lg border bg-background/80">
            <CardHeader className="flex flex-col sm:flex-row items-start sm:items-center gap-2 pb-2">
              <span className="bg-primary/10 rounded-full p-2">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-5 w-5 sm:h-6 sm:w-6 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M16 17v1a3 3 0 01-6 0v-1M12 7a4 4 0 00-4 4v1a4 4 0 008 0v-1a4 4 0 00-4-4z"
                  />
                </svg>
              </span>
              <h2 className="text-lg sm:text-xl font-semibold">
                Authentication Providers
              </h2>
            </CardHeader>
            <CardContent>
              <AuthProviders />
            </CardContent>
          </Card>

          {/* Payment Settings Link */}
          <Card className="transition-shadow hover:shadow-lg border border-primary bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10">
            <CardHeader className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 pb-2">
              <div className="flex items-start sm:items-center gap-2 flex-1">
                <span className="bg-primary/10 rounded-full p-2 flex-shrink-0">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5 sm:h-6 sm:w-6 text-primary"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                </span>
                <div className="min-w-0">
                  <h2 className="text-lg sm:text-xl font-semibold">
                    Payment Settings
                  </h2>
                  <p className="text-sm text-muted-foreground">
                    View payment status and manage subscription
                  </p>
                </div>
              </div>
              <Button
                onClick={() => router.push('/payment')}
                className="bg-primary text-primary-foreground hover:bg-primary/90 w-full sm:w-auto"
                size="sm"
              >
                Manage Payment
              </Button>
            </CardHeader>
          </Card>
        </div>
      </div>
    </Suspense>
  );
}
