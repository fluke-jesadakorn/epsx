'use client';

// import { GoogleSignIn } from "@/components/auth/GoogleSignIn";
import { useEffect, useState } from 'react';
import Link from 'next/link';
import { useSearchParams, useRouter } from 'next/navigation';

import { EmailPasswordForm } from '@/components/auth/EmailPasswordForm';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/context/auth-context';

export default function LoginPage() {
  const { user, loading } = useAuth();
  const searchParams = useSearchParams();
  const returnUrl = searchParams.get('returnUrl') || '/dashboard';
  const [redirecting, setRedirecting] = useState(false);
  const router = useRouter();

  useEffect(() => {
    // Only redirect if we have a confirmed authenticated user and we're not already redirecting
    if (user && !loading && !redirecting) {
      setRedirecting(true);
      // Add a small delay to ensure session cookie is set and recognized
      setTimeout(() => {
        // Use window.location instead of redirect to ensure proper navigation
        // Append a query parameter to indicate this is a post-login redirect
        const redirectUrl = `${returnUrl}${returnUrl.includes('?') ? '&' : '?'}postLogin=true`;
        router.push(redirectUrl);
      }, 500);
    }
  }, [user, loading, returnUrl, redirecting]);

  // Show loading state while checking authentication or redirecting
  if (loading || redirecting) {
    return (
      <main className="flex items-center justify-center min-h-screen w-full p-4 md:p-6 lg:p-8 bg-gradient-to-br from-gray-50 to-gray-100">
        <Card className="w-full sm:max-w-md md:max-w-lg">
          <CardContent className="flex items-center justify-center p-8">
            <div className="text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900 mx-auto mb-4"></div>
              <p className="text-gray-600">
                {redirecting ? 'Redirecting...' : 'Checking authentication...'}
              </p>
            </div>
          </CardContent>
        </Card>
      </main>
    );
  }

  // Only show login form if user is definitely not authenticated
  if (user) {
    return null; // Will redirect in useEffect
  }

  return (
    <main className="flex items-center justify-center min-h-screen w-full p-4 md:p-6 lg:p-8 bg-gradient-to-br from-gray-50 to-gray-100">
      <Card className="w-full sm:max-w-md md:max-w-lg hover:scale-105 transition-transform duration-300">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl md:text-3xl font-bold mb-4">
            Welcome
          </CardTitle>
          {/* <CardDescription>
            Choose your preferred sign in method
          </CardDescription> */}
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-background px-2 text-muted-foreground">
                  Or continue with
                </span>
              </div>
            </div> */}

            <EmailPasswordForm isSignUp={false} />

            <div className="text-center mt-4">
              <Button
                variant="link"
                type="button"
                className="min-h-[48px]"
                asChild
              >
                <Link href="/signup">Don&apos;t have an account? Sign up</Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </main>
  );
}
