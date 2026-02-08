import { Alert, AlertDescription } from '@/components/ui/alert';
import { getCurrentUser } from '@/lib/server-actions';

export async function AuthProvidersServer() {
  const user = await getCurrentUser();

  return (
    <div>
      <h3 className="text-lg font-semibold mb-4">Connected Accounts</h3>

      <Alert>
        <AlertDescription>
          Third-party provider integration (Google, Apple, etc.) will be available in a future update.
          Currently using secure backend authentication.
        </AlertDescription>
      </Alert>

      <div className="space-y-3 mt-4">
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
              📧
            </div>
            <div>
              <p className="font-medium">Email Authentication</p>
              <p className="text-sm text-muted-foreground">{(user as any)?.email || 'Not signed in'}</p>
            </div>
          </div>
          <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">
            {user ? 'Connected' : 'Disconnected'}
          </span>
        </div>

      </div>
    </div>
  );
}