import { getCurrentUser } from '@/lib/server-actions';
import { SettingsClient } from '@/components/settings/SettingsClient';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';
export default async function SettingsPage() {
  // Fetch user data server-side
  const user = await getCurrentUser();
  // Allow access even without authentication for demo purposes
  if (!user) {
    return (
      <div className="container mx-auto p-6">
        <h1 className="text-3xl font-bold mb-6">Settings</h1>
        <Card>
          <CardHeader>
            <CardTitle>Guest Access</CardTitle>
          </CardHeader>
          <CardContent>
            <p>You are viewing settings as a guest user. Please log in for full functionality.</p>
            <Link href="/login">
              <Button className="mt-4">
                Log In
              </Button>
            </Link>
          </CardContent>
        </Card>
      </div>
    );
  }
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
              <p><strong>Email:</strong> {user.email}</p>
              <p><strong>User ID:</strong> {user.user_id || user.id || user.uid}</p>
              <p><strong>Email Verified:</strong> {user.emailVerified ? 'Yes' : 'No'}</p>
            </div>
          </CardContent>
        </Card>
        <SettingsClient />
      </div>
    </div>
  );
}