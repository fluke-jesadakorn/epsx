import { getCurrentUser } from '@/lib/server-actions';
import { SettingsClient } from '@/components/settings/SettingsClient';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui';

export const dynamic = 'force-dynamic';
export default async function SettingsPage() {
  // Fetch user data server-side
  const user = await getCurrentUser();
  
  return (
    <div className="container mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Settings</h1>
      <div className="grid gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Account Information</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <p><strong>Email:</strong> {user?.email}</p>
              <p><strong>User ID:</strong> {user?.user_id || user?.id || user?.uid}</p>
              <p><strong>Email Verified:</strong> {user?.emailVerified ? 'Yes' : 'No'}</p>
              <p><strong>Auth Method:</strong> {(user as any)?.walletAddress || (user as any)?.wallet_address ? 'Web3 Wallet' : 'OIDC'}</p>
              {((user as any)?.walletAddress || (user as any)?.wallet_address) && (
                <p><strong>Wallet:</strong> {((user as any)?.walletAddress || (user as any)?.wallet_address)?.slice(0, 6)}...{((user as any)?.walletAddress || (user as any)?.wallet_address)?.slice(-4)}</p>
              )}
            </div>
          </CardContent>
        </Card>
        <SettingsClient />
      </div>
    </div>
  );
}