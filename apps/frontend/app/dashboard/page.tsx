import { checkAuth } from '@/lib/auth';

export default async function DashboardPage() {
  const user = await checkAuth();

  return (
    <div className="container mx-auto py-8">
      <div className="max-w-4xl mx-auto">
        <div className="bg-card rounded-lg shadow-sm p-6">
          <h1 className="text-2xl font-bold mb-4">Dashboard</h1>

          <div className="space-y-4">
            <div className="p-4 bg-muted rounded-md">
              <h2 className="font-semibold mb-2">User Information</h2>
              <div className="space-y-2 text-sm">
                <p>User ID: {user.uid}</p>
                {user.email && <p>Email: {user.email}</p>}
                {user.email_verified !== undefined && (
                  <p>Email Verified: {user.email_verified ? 'Yes' : 'No'}</p>
                )}
              </div>
            </div>

            <div className="flex items-center justify-between">
              <p className="text-muted-foreground text-sm">
                This is a protected route that requires authentication.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
