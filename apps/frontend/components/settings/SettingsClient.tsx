'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { useRouter } from 'next/navigation';

export function SettingsClient() {
  const router = useRouter();

  const handleSignOut = async () => {
    // Redirect to OIDC logout endpoint
    router.push('/auth/logout');
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Actions</CardTitle>
      </CardHeader>
      <CardContent>
        <Button onClick={handleSignOut} variant="destructive">
          Sign Out
        </Button>
      </CardContent>
    </Card>
  );
}