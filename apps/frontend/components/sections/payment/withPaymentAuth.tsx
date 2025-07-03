'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { auth } from '@/lib/firebase';
import { useRouter } from 'next/navigation';

export function withPaymentAuth<P extends object>(
  WrappedComponent: React.ComponentType<P>,
) {
  return function WithPaymentAuthComponent(props: P) {
    const [loading, setLoading] = useState(true);
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const router = useRouter();

    useEffect(() => {
      const unsubscribe = auth.onAuthStateChanged((user) => {
        setIsAuthenticated(!!user);
        setLoading(false);
      });

      return () => unsubscribe();
    }, []);

    if (loading) {
      return (
        <Card className="w-full animate-pulse">
          <CardContent className="py-6">
            <div className="space-y-3">
              <div className="h-4 bg-gray-200 rounded w-1/2 dark:bg-gray-700"></div>
              <div className="h-4 bg-gray-200 rounded w-1/4 dark:bg-gray-700"></div>
            </div>
          </CardContent>
        </Card>
      );
    }

    if (!isAuthenticated) {
      return (
        <Card className="w-full transition-shadow hover:shadow-lg border-warning border bg-warning/10">
          <CardContent className="py-6">
            <div className="text-center space-y-4">
              <div className="text-warning flex items-center justify-center gap-2">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
                <span className="font-medium">
                  Please log in to view payment information
                </span>
              </div>
              <Button
                onClick={() => router.push('/login')}
                className="bg-warning text-white hover:bg-warning/90"
              >
                Log In
              </Button>
            </div>
          </CardContent>
        </Card>
      );
    }

    return <WrappedComponent {...props} />;
  };
}
