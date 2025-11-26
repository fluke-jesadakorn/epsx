import { getCurrentUser } from '@/lib/actions/auth';
import { Alert, AlertDescription } from '@/components/ui/alert';

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
              <p className="text-sm text-muted-foreground">{user?.email || 'Not signed in'}</p>
            </div>
          </div>
          <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">
            {user ? 'Connected' : 'Disconnected'}
          </span>
        </div>

        <div className="flex items-center justify-between p-4 border rounded-lg opacity-50">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-red-100 rounded-full flex items-center justify-center">
              🔴
            </div>
            <div>
              <p className="font-medium">Google</p>
              <p className="text-sm text-muted-foreground">Connect your Google account</p>
            </div>
          </div>
          <button 
            className="px-3 py-1 border border-gray-300 rounded-md text-sm text-gray-500"
            disabled
            aria-disabled="true"
          >
            Coming Soon
          </button>
        </div>

        <div className="flex items-center justify-between p-4 border rounded-lg opacity-50">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
              🍎
            </div>
            <div>
              <p className="font-medium">Apple</p>
              <p className="text-sm text-muted-foreground">Connect your Apple ID</p>
            </div>
          </div>
          <button 
            className="px-3 py-1 border border-gray-300 rounded-md text-sm text-gray-500"
            disabled
            aria-disabled="true"
          >
            Coming Soon
          </button>
        </div>
      </div>
    </div>
  );
}