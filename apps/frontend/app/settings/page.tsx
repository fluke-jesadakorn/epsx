"use client";

import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { PaymentForm } from "@/components/features/payment/PaymentForm";
import { ProfileSettings } from "@/components/features/settings/ProfileSettings";
import { AuthProviders } from "@/components/features/settings/AuthProviders";
import { Suspense } from "react";
import { SkeletonLoader } from "@/components/common/Skeleton";

export default function SettingsPage() {
  return (
    <Suspense fallback={<SkeletonLoader />}>
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
