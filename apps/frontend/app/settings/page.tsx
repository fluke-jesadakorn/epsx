'use client';

import { Suspense } from 'react';

import { AuthProviders } from '@/components/features/settings/AuthProviders';
import { ProfileSettings } from '@/components/features/settings/ProfileSettings';
import { UserLevelDisplay } from '@/components/features/settings/UserLevelDisplay';
import { AchievementBadges } from '@/components/features/settings/AchievementBadges';
import { LevelBenefitsComparison } from '@/components/features/settings/LevelBenefitsComparison';
import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/auth-context';
import { status } from '@/services/pay';
import { useState, useEffect } from 'react';
import type { UserLevelType } from '@/app/constants/packages';

export default function SettingsPage() {
  const router = useRouter();
  const { user } = useAuth();
  const [userLevel, setUserLevel] = useState<UserLevelType>('BASIC');
  const [paymentCount, setPaymentCount] = useState(0);
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
        setPaymentCount(0); // Placeholder
      } catch (error) {
        console.error('Failed to fetch user data:', error);
        setUserLevel('BASIC');
        setPaymentCount(0);
      } finally {
        setIsLoadingUserData(false);
      }
    };

    fetchUserData();
  }, [user]);

  const SettingsSkeleton = () => (
    <div className="max-w-2xl mx-auto p-6">
      <div className="flex flex-col gap-6">
        <div className="border rounded-lg shadow-sm">
          <div className="p-4 border-b">
            <div className="h-6 bg-gray-200 rounded-md animate-pulse w-1/4 dark:bg-gray-700"></div>
          </div>
          <div className="p-4 space-y-4">
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse w-3/4 dark:bg-gray-700"></div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
          </div>
        </div>

        {/* Authentication Providers Card Skeleton */}
        <div className="border rounded-lg shadow-sm">
          <div className="p-4 border-b">
            <div className="h-6 bg-gray-200 rounded-md animate-pulse w-1/3 dark:bg-gray-700"></div>
          </div>
          <div className="p-4 space-y-4">
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse w-5/6 dark:bg-gray-700"></div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
          </div>
        </div>

        {/* Payment Settings Card Skeleton */}
        <div className="border rounded-lg shadow-sm">
          <div className="p-4 border-b">
            <div className="h-6 bg-gray-200 rounded-md animate-pulse w-1/4 dark:bg-gray-700"></div>
          </div>
          <div className="p-4 space-y-4">
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded-md animate-pulse w-2/3 dark:bg-gray-700"></div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700"></div>
            <div className="h-10 bg-gray-200 rounded-md animate-pulse w-1/3 dark:bg-gray-700"></div>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <Suspense fallback={<SettingsSkeleton />}>
      <div className="max-w-3xl mx-auto p-6">
        {/* Page Header */}
        <div className="mb-8 text-center">
          <h1 className="text-3xl md:text-4xl font-bold tracking-tight text-primary mb-2 flex items-center justify-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-8 w-8 text-primary"
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
          <p className="text-muted-foreground text-base max-w-xl mx-auto">
            Manage your profile, track your progress, and unlock premium
            features.
          </p>
        </div>
        <div className="flex flex-col gap-8">
          {/* User Level Display - New Enhanced Section */}
          <UserLevelDisplay className="animate-fade-in" />

          {/* Achievements Section */}
          {!isLoadingUserData && (
            <AchievementBadges
              currentLevel={userLevel}
              paymentCount={paymentCount}
              className="animate-fade-in-delayed"
            />
          )}

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
            <CardHeader className="flex flex-row items-center gap-2 pb-2">
              <span className="bg-primary/10 rounded-full p-2">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-6 w-6 text-primary"
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
              <h2 className="text-xl font-semibold">
                Authentication Providers
              </h2>
            </CardHeader>
            <CardContent>
              <AuthProviders />
            </CardContent>
          </Card>

          {/* Payment Settings Link */}
          <Card className="transition-shadow hover:shadow-lg border border-primary bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <div className="flex items-center gap-2">
                <span className="bg-primary/10 rounded-full p-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-6 w-6 text-primary"
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
                <div>
                  <h2 className="text-xl font-semibold">Payment Settings</h2>
                  <p className="text-sm text-muted-foreground">
                    View payment status and manage subscription
                  </p>
                </div>
              </div>
              <Button
                onClick={() => router.push('/payment')}
                className="bg-primary text-primary-foreground hover:bg-primary/90"
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
