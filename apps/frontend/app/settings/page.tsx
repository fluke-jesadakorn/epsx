"use client";

import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { Suspense } from "react";

export default function SettingsPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
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
