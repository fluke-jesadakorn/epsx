"use client";

import { Card, CardHeader, CardContent } from "@/components/ui/card";

export default function SettingsPage() {
  return (
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
  );
}
