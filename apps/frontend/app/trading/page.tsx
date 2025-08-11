import { getCurrentUser } from '@epsx/server-actions';
import { redirect } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default async function TradingPage() {
  // Server-side auth check with automatic redirect if not authenticated
  let user = null;
  try {
    const result = await getCurrentUser({});
    user = result?.success ? result.data : null;
  } catch (error) {
    console.error('TradingPage: Failed to get user:', error);
  }

  if (!user) {
    redirect('/login?callbackUrl=/trading');
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
