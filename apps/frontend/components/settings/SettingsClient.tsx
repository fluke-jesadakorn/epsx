'use client';

import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';

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