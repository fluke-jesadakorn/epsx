import { getCurrentUser } from '@/lib/server-actions';
import { SettingsClient } from '@/components/settings/SettingsClient';
import { RequireProfileManagement } from '@/lib/permissions/guards';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui';

export const dynamic = 'force-dynamic';
export default async function SettingsPage() {
  // Fetch user data server-side
  const user = await getCurrentUser();
  
  return (
    <RequireProfileManagement
      fallback={
        <div className="container mx-auto p-6 text-center">
          <div className="max-w-md mx-auto mt-20 p-8 bg-white rounded-2xl shadow-lg border border-orange-200/50">
            <div className="w-16 h-16 mx-auto mb-4 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-full flex items-center justify-center">
              <span className="text-2xl">⚙️</span>
            </div>
            <h1 className="text-xl font-semibold text-gray-900 mb-2">Settings Access Required</h1>
            <p className="text-gray-600 mb-6">You need profile management permissions to access settings.</p>
            <button 
              className="px-6 py-2 bg-gradient-to-r from-orange-500 to-yellow-500 text-white rounded-xl font-medium hover:shadow-lg transition-all"
              onClick={() => window.location.href = '/billing'}
            >
              Upgrade Plan
            </button>
          </div>
        </div>
      }
    >
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
    </RequireProfileManagement>
  );
}