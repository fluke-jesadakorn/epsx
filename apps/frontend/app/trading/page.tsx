'use client';

import { useAuth } from '@/context/auth-context';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function TradingPage() {
  const { user } = useAuth();
  const router = useRouter();

  if (!user) {
    router.push('/login');
    return null;
  }

  return (
    <div className="container mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Trading Dashboard</h1>
      
      <div className="grid gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Welcome to Trading</CardTitle>
          </CardHeader>
          <CardContent>
            <p>Trading features are coming soon. Check back later!</p>
            <p className="text-sm text-muted-foreground mt-2">
              Logged in as: {user.email}
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
