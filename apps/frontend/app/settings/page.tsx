'use client';

import { Suspense } from 'react';

import { AuthProviders } from '@/components/features/settings/AuthProviders';
import { ProfileSettings } from '@/components/features/settings/ProfileSettings';
import { UserLevelDisplay } from '@/components/features/settings/UserLevelDisplay';
import { LevelBenefitsComparison } from '@/components/features/settings/LevelBenefitsComparison';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/auth-context-improved';
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
      <div className="relative min-h-screen overflow-hidden">
        {/* PancakeSwap-style vibrant background */}
        <div className="fixed inset-0 z-0">
          {/* Main gradient background */}
          <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

          {/* Floating gradient orbs - PancakeSwap style */}
          <div className="absolute -top-40 -left-40 w-96 h-96 bg-gradient-to-br from-orange-400/30 to-yellow-400/30 rounded-full blur-3xl animate-bounce-slow" />
          <div className="absolute top-20 -right-32 w-80 h-80 bg-gradient-to-br from-blue-400/25 to-cyan-400/25 rounded-full blur-3xl animate-float" />
          <div className="absolute bottom-20 left-20 w-72 h-72 bg-gradient-to-br from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse-gentle" />
          <div className="absolute top-1/2 right-1/4 w-64 h-64 bg-gradient-to-br from-green-400/15 to-emerald-400/15 rounded-full blur-3xl animate-float-reverse" />
          <div className="absolute bottom-1/3 left-1/3 w-56 h-56 bg-gradient-to-br from-yellow-400/20 to-orange-400/20 rounded-full blur-3xl animate-bounce-gentle" />

          {/* Mesh gradient overlays for depth */}
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)] animate-pulse-slow" />
          <div
            className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)] animate-pulse-slow"
            style={{ animationDelay: '1s' }}
          />
          <div
            className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)] animate-pulse-slow"
            style={{ animationDelay: '2s' }}
          />

          {/* Decorative geometric shapes */}
          <div className="absolute top-1/4 left-1/4 w-32 h-32 bg-gradient-to-br from-orange-300/10 to-yellow-300/10 rounded-2xl rotate-45 animate-spin-slow" />
          <div className="absolute bottom-1/3 right-1/3 w-24 h-24 bg-gradient-to-br from-blue-300/10 to-cyan-300/10 rounded-full animate-bounce-gentle" />
          <div className="absolute top-3/4 left-10 w-20 h-20 bg-gradient-to-br from-purple-300/10 to-pink-300/10 rounded-2xl rotate-12 animate-float-gentle" />

          {/* Floating PancakeSwap-style emojis */}
          <div className="absolute top-20 left-1/3 text-4xl opacity-10 animate-float">
            🥞
          </div>
          <div className="absolute bottom-32 right-1/4 text-3xl opacity-15 animate-bounce-gentle">
            ✨
          </div>
          <div className="absolute top-1/2 left-10 text-2xl opacity-20 animate-wiggle">
            ⚙️
          </div>
          <div className="absolute bottom-1/4 right-20 text-3xl opacity-12 animate-pulse-gentle">
            🚀
          </div>
        </div>

        <div className="relative z-10 container-responsive py-8 lg:py-12">
          <div className="max-w-6xl mx-auto">
            {/* Enhanced Page Header with PancakeSwap vibes */}
            <div className="mb-12 sm:mb-16 text-center relative">
              <div className="relative inline-block">
                {/* Floating decorative elements around title */}
                <div className="absolute -top-8 -left-8 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full blur-xl animate-float" />
                <div className="absolute -bottom-8 -right-8 w-20 h-20 bg-gradient-to-br from-blue-400/20 to-cyan-400/20 rounded-full blur-xl animate-bounce-gentle" />

                <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight mb-6 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x">
                  ⚙️ Settings
                  <span className="block mt-2 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent">
                    Dashboard
                  </span>
                  <span className="block mt-2 text-2xl sm:text-3xl md:text-4xl">
                    ✨
                  </span>
                </h1>
              </div>

              <div className="relative max-w-4xl mx-auto animate-slide-up-delayed">
                <p className="text-lg sm:text-xl md:text-2xl text-gray-600 dark:text-gray-300 max-w-4xl mx-auto leading-relaxed mb-8">
                  🚀 Customize your experience with our comprehensive settings!
                  <span className="block mt-2 bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent font-bold">
                    Manage your profile and unlock premium features 🥞
                  </span>
                </p>

                {/* Enhanced Decorative Divider */}
                <div className="flex items-center justify-center mb-8">
                  <div className="flex items-center gap-4">
                    <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
                    <div
                      className="w-3 h-3 bg-yellow-400 rounded-full animate-pulse"
                      style={{ animationDelay: '0.5s' }}
                    />
                    <div className="h-px bg-gradient-to-r from-transparent via-orange-400/50 to-transparent w-24 sm:w-32" />
                    <div className="p-3 bg-gradient-to-br from-orange-100/50 to-yellow-100/50 dark:from-orange-900/50 dark:to-yellow-900/50 rounded-full backdrop-blur-sm">
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        className="h-6 w-6 text-orange-500 animate-spin-slow"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        />
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                      </svg>
                    </div>
                    <div className="h-px bg-gradient-to-r from-orange-400/50 via-transparent to-transparent w-24 sm:w-32" />
                    <div
                      className="w-3 h-3 bg-blue-400 rounded-full animate-pulse"
                      style={{ animationDelay: '1s' }}
                    />
                    <div
                      className="w-2 h-2 bg-purple-400 rounded-full animate-pulse"
                      style={{ animationDelay: '1.5s' }}
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Enhanced Content Grid */}
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 sm:gap-6 lg:gap-8">
              {/* Main Content Area */}
              <div className="lg:col-span-8 space-y-4 sm:space-y-6 lg:space-y-8">
                {/* User Level Display - Enhanced */}
                <div className="relative">
                  <UserLevelDisplay className="animate-fade-in hover-lift" />
                  {/* Floating decoration */}
                  <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary/30 rounded-full animate-bounce-gentle hidden sm:block"></div>
                </div>

                {/* Level Benefits Comparison - Enhanced */}
                {!isLoadingUserData && (
                  <div className="relative">
                    <LevelBenefitsComparison
                      currentLevel={userLevel}
                      className="animate-fade-in-delayed-2 hover-lift"
                    />
                    {/* Floating decoration */}
                    <div className="absolute -bottom-2 -left-2 w-3 h-3 bg-secondary/30 rounded-full animate-pulse hidden sm:block"></div>
                  </div>
                )}

                {/* Profile Settings - Enhanced with PancakeSwap style */}
                <div className="relative animate-fade-in-delayed-3">
                  {/* Floating decorative elements */}
                  <div className="absolute -top-4 -left-4 w-12 h-12 bg-gradient-to-br from-purple-400/20 to-pink-400/20 rounded-full blur-xl animate-float-gentle" />
                  <div className="absolute -bottom-4 -right-4 w-8 h-8 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full blur-lg animate-bounce-gentle" />

                  <div className="relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8 shadow-2xl border border-purple-200/50 dark:border-purple-400/20 overflow-hidden hover-lift">
                    {/* Card background pattern */}
                    <div className="absolute inset-0 bg-gradient-to-br from-purple-50/50 via-transparent to-pink-50/50 dark:from-purple-900/10 dark:via-transparent dark:to-pink-900/10" />
                    <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full blur-2xl" />
                    <div className="absolute bottom-0 left-0 w-24 h-24 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full blur-2xl" />

                    <div className="relative z-10">
                      <div className="mb-4 sm:mb-6">
                        <h3 className="text-lg sm:text-2xl lg:text-3xl font-bold mb-2 sm:mb-3 flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3">
                          <div className="p-2 sm:p-3 bg-gradient-to-br from-purple-100 to-pink-100 dark:from-purple-900/50 dark:to-pink-900/50 rounded-xl sm:rounded-2xl shadow-lg">
                            <span className="text-lg sm:text-2xl animate-wiggle">👤</span>
                          </div>
                          <span className="bg-gradient-to-r from-purple-500 to-pink-500 bg-clip-text text-transparent">
                            🎨 Profile Management
                          </span>
                        </h3>
                        <p className="text-xs sm:text-sm text-muted-foreground sm:ml-14">
                          Update your personal information and customize your
                          experience ✨
                        </p>
                      </div>
                      <ProfileSettings />
                    </div>
                  </div>
                </div>
              </div>

              {/* Sidebar Content */}
              <div className="lg:col-span-4 space-y-4 sm:space-y-6 animate-slide-up-delayed">
                {/* Quick Actions Card - Enhanced with PancakeSwap style */}
                <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8 shadow-2xl border border-orange-200/50 dark:border-orange-400/20 hover-lift">
                  {/* Card background pattern */}
                  <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
                  <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full blur-2xl" />
                  <div className="absolute bottom-0 left-0 w-24 h-24 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full blur-2xl" />

                  <CardHeader className="relative z-10 pb-4">
                    <CardTitle className="flex items-center gap-3 text-xl font-bold">
                      <div className="p-3 bg-gradient-to-br from-orange-100 to-yellow-100 dark:from-orange-900/50 dark:to-yellow-900/50 rounded-2xl shadow-lg">
                        <span className="text-2xl animate-bounce-gentle">
                          ⚡
                        </span>
                      </div>
                      <span className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
                        🚀 Quick Actions
                      </span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="relative z-10 space-y-3 sm:space-y-4">
                    <Button
                      onClick={() => router.push('/payment')}
                      className="w-full h-10 sm:h-12 bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-xl sm:rounded-2xl shadow-2xl hover:shadow-orange-300/50 hover:scale-105 transition-all duration-300 group font-bold text-sm sm:text-base lg:text-lg"
                    >
                      <span className="mr-2 sm:mr-3 text-base sm:text-xl group-hover:animate-bounce-gentle">
                        💎
                      </span>
                      Upgrade Plan
                    </Button>
                    <Button
                      onClick={() => router.push('/dashboard')}
                      className="w-full h-10 sm:h-12 bg-white/10 backdrop-blur-sm border-2 border-orange-300/50 text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-orange-900/20 rounded-xl sm:rounded-2xl shadow-xl hover:shadow-orange-300/30 hover:scale-105 transition-all duration-300 group font-bold text-sm sm:text-base"
                    >
                      <span className="mr-2 sm:mr-3 text-base sm:text-xl group-hover:animate-wiggle">
                        📊
                      </span>
                      Go to Dashboard
                    </Button>
                  </CardContent>
                </Card>

                {/* Authentication Providers - Enhanced PancakeSwap style */}
                <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8 shadow-2xl border border-blue-200/50 dark:border-blue-400/20 hover-lift">
                  {/* Card background pattern */}
                  <div className="absolute inset-0 bg-gradient-to-br from-blue-50/50 via-transparent to-purple-50/50 dark:from-blue-900/10 dark:via-transparent dark:to-purple-900/10" />
                  <div className="absolute top-0 left-0 w-28 h-28 bg-gradient-to-br from-blue-400/10 to-purple-400/10 rounded-full blur-2xl" />
                  <div className="absolute bottom-0 right-0 w-20 h-20 bg-gradient-to-br from-cyan-400/10 to-blue-400/10 rounded-full blur-2xl" />

                  <CardHeader className="relative z-10 pb-4">
                    <CardTitle className="flex items-center gap-3 text-xl font-bold">
                      <div className="p-3 bg-gradient-to-br from-blue-100 to-purple-100 dark:from-blue-900/50 dark:to-purple-900/50 rounded-2xl shadow-lg">
                        <span className="text-2xl animate-pulse-gentle">
                          🔐
                        </span>
                      </div>
                      <span className="bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent">
                        🛡️ Security
                      </span>
                    </CardTitle>
                    <p className="text-sm text-muted-foreground ml-14">
                      Manage your authentication methods safely 🔒
                    </p>
                  </CardHeader>
                  <CardContent className="relative z-10">
                    <AuthProviders />
                  </CardContent>
                </Card>

                {/* Help & Support Card - Enhanced PancakeSwap style */}
                <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8 shadow-2xl border border-green-200/50 dark:border-green-400/20 hover-lift">
                  {/* Card background pattern */}
                  <div className="absolute inset-0 bg-gradient-to-br from-green-50/50 via-transparent to-emerald-50/50 dark:from-green-900/10 dark:via-transparent dark:to-emerald-900/10" />
                  <div className="absolute top-0 right-0 w-24 h-24 bg-gradient-to-br from-green-400/10 to-emerald-400/10 rounded-full blur-2xl" />
                  <div className="absolute bottom-0 left-0 w-20 h-20 bg-gradient-to-br from-emerald-400/10 to-green-400/10 rounded-full blur-2xl" />

                  <CardHeader className="relative z-10 pb-4">
                    <CardTitle className="flex items-center gap-3 text-xl font-bold">
                      <div className="p-3 bg-gradient-to-br from-green-100 to-emerald-100 dark:from-green-900/50 dark:to-emerald-900/50 rounded-2xl shadow-lg">
                        <span className="text-2xl animate-float-gentle">
                          💡
                        </span>
                      </div>
                      <span className="bg-gradient-to-r from-green-500 to-emerald-500 bg-clip-text text-transparent">
                        🤝 Help & Support
                      </span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="relative z-10 space-y-4">
                    <p className="text-sm text-muted-foreground">
                      Need assistance? We're here to help! 🥞✨
                    </p>
                    <div className="space-y-2 sm:space-y-3">
                      <Button
                        variant="outline"
                        className="w-full justify-start h-9 sm:h-11 hover:bg-green-50 dark:hover:bg-green-900/20 hover:border-green-300 rounded-lg sm:rounded-xl group transition-all duration-300 text-sm sm:text-base"
                      >
                        <span className="mr-2 sm:mr-3 text-sm sm:text-lg group-hover:animate-bounce-gentle">
                          📚
                        </span>
                        Documentation
                      </Button>
                      <Button
                        variant="outline"
                        className="w-full justify-start h-9 sm:h-11 hover:bg-green-50 dark:hover:bg-green-900/20 hover:border-green-300 rounded-lg sm:rounded-xl group transition-all duration-300 text-sm sm:text-base"
                      >
                        <span className="mr-2 sm:mr-3 text-sm sm:text-lg group-hover:animate-wiggle">
                          💬
                        </span>
                        Contact Support
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Suspense>
  );
}
