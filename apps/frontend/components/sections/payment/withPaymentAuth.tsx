'use client';

import React from 'react';
import { useSession } from 'next-auth/react';
import { Card, CardContent } from '@/components/ui/card';
import { Lock, User } from 'lucide-react';
import { Button } from '@/components/ui/button';
import Link from 'next/link';

/**
 * Higher-order component that adds payment/authentication guard to components
 * Shows auth prompt if user is not authenticated
 */
export function withPaymentAuth<P extends object>(
  WrappedComponent: React.ComponentType<P>
) {
  const AuthGuardedComponent = (props: P) => {
    const { data: session, status } = useSession();
    const user = session?.user;
    const loading = status === 'loading';

    if (loading) {
      return (
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-orange-500"></div>
        </div>
      );
    }

    if (!user) {
      return (
        <div className="min-h-[400px] flex items-center justify-center p-4">
          <Card className="max-w-md w-full">
            <CardContent className="p-8 text-center">
              <div className="mb-6">
                <div className="w-16 h-16 mx-auto bg-gradient-to-br from-orange-500 to-yellow-500 rounded-2xl flex items-center justify-center mb-4">
                  <Lock className="w-8 h-8 text-white" />
                </div>
              </div>
              <h2 className="text-2xl font-bold mb-4">
                Authentication Required
              </h2>
              <p className="text-gray-600 dark:text-gray-300 mb-6">
                Please log in to access payment features and manage your subscription.
              </p>
              <Link href="/login">
                <Button className="w-full" size="lg">
                  <User className="mr-2 h-5 w-5" />
                  Log In
                </Button>
              </Link>
            </CardContent>
          </Card>
        </div>
      );
    }

    // User is authenticated, render the wrapped component
    return <WrappedComponent {...props} />;
  };

  // Set display name for debugging
  AuthGuardedComponent.displayName = `withPaymentAuth(${
    WrappedComponent.displayName || WrappedComponent.name || 'Component'
  })`;

  return AuthGuardedComponent;
}