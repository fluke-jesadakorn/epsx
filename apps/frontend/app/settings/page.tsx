"use client";

import { Card, CardHeader, CardContent } from "@/components/ui/card";
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
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <h2 className="text-2xl font-semibold">Authentication Providers</h2>
          </CardHeader>
          <CardContent>
          </CardContent>
        </Card>
        </div>
      </div>
    </Suspense>
  );
}
