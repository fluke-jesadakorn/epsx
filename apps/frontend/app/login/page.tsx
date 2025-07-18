'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function LoginPage() {
  const { user, loading } = useAuth();
  const searchParams = useSearchParams();
  const returnUrl = searchParams.get('returnUrl') || '/my-data';
  const [redirecting, setRedirecting] = useState(false);

  useEffect(() => {
    if (user && !loading && !redirecting) {
      setRedirecting(true);
      // Redirect to the return URL
      window.location.href = returnUrl;
    }
  }, [user, loading, returnUrl, redirecting]);

  // Show loading state while checking authentication
  if (loading) {
    return (
      <main className="min-h-screen flex items-center justify-center">
        <Card className="w-full max-w-md">
          <CardContent className="p-8 text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500 mx-auto mb-4"></div>
            <p>Loading...</p>
          </CardContent>
        </Card>
      </main>
    );
  }

  if (redirecting) {
    return (
      <main className="min-h-screen flex items-center justify-center">
        <Card className="w-full max-w-md">
          <CardContent className="p-8 text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500 mx-auto mb-4"></div>
            <p>Redirecting...</p>
          </CardContent>
        </Card>
      </main>
    );
  }

  // If user is authenticated, don't show login form
  if (user) {
    return null;
  }

  return (
    <main className="min-h-screen flex items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl text-center">Welcome Back</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="text-center">
              <p className="text-gray-600 mb-4">Sign in to continue</p>
              <Button className="w-full" asChild>
                <Link href="/">Go to Home</Link>
              </Button>
            </div>
            <div className="text-center">
              <Button variant="link" asChild>
                <Link href="/signup">Don't have an account? Sign up</Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </main>
  );
}
