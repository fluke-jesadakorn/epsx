import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/server-actions';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';

export const dynamic = 'force-dynamic';
export default async function VerifyEmailPage() {
  // Server-side authentication check
  let user = null;
  try {
    user = await getCurrentUser();
  } catch (error) {
    // Handle error silently
  }
  if (!user) {
    const { redirectToBackendLogin } = await import('@/lib/server/auth');
    redirectToBackendLogin('/verify-email');
    return;
  }
  if (user.emailVerified) {
    redirect('/dashboard');
  }
  return (
    <div className="flex items-center justify-center min-h-screen">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Verify Your Email</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="mb-4">
            Please check your email ({user.email}) for a verification link.
          </p>
          <Button asChild className="w-full">
            <Link href="/dashboard">
              Continue to Dashboard
            </Link>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}