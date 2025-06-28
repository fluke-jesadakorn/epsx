"use client";

import { Suspense } from "react";

import { SkeletonLoader } from "@/components/common/Skeleton";
import { PaymentForm } from "@/components/features/payment/PaymentForm";
import { AuthProviders } from "@/components/features/settings/AuthProviders";
import { ProfileSettings } from "@/components/features/settings/ProfileSettings";
import { Card, CardHeader, CardContent } from "@/components/ui/card";

export default function SettingsPage() {
  const SettingsSkeleton = () => (
    <div className="max-w-2xl mx-auto p-6">
      <div className="flex flex-col gap-6">
        {/* Profile Settings Card Skeleton */}
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
      <div className="max-w-2xl mx-auto p-6">
        <div className="flex flex-col gap-6">
        <Card>
          <CardHeader>
            <h2 className="text-2xl font-semibold">Profile Settings</h2>
          </CardHeader>
          <CardContent>
            <ProfileSettings />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <h2 className="text-2xl font-semibold">Authentication Providers</h2>
          </CardHeader>
          <CardContent>
            <AuthProviders />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <h2 className="text-2xl font-semibold">Payment Settings</h2>
          </CardHeader>
          <CardContent>
            <PaymentForm />
          </CardContent>
        </Card>
        </div>
      </div>
    </Suspense>
  );
}
