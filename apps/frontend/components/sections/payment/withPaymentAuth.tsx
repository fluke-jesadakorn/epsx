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
    // BYPASS AUTH FOR LOCAL DEBUGGING
    const isAuthenticated = true;
    const loading = false;
    // const [loading, setLoading] = useState(true);
    // const [isAuthenticated, setIsAuthenticated] = useState(false);
    // const router = useRouter();

    // useEffect(() => {
    //   const unsubscribe = auth.onAuthStateChanged((user) => {
    //     setIsAuthenticated(!!user);
    //     setLoading(false);
    //   });
    //   return () => unsubscribe();
    // }, []);

    // if (loading) {
    //   return (
    //     <Card className="w-full animate-pulse">
    //       <CardContent className="py-6">
    //         <div className="space-y-3">
    //           <div className="h-4 bg-gray-200 rounded w-1/2 dark:bg-gray-700"></div>
    //           <div className="h-4 bg-gray-200 rounded w-1/4 dark:bg-gray-700"></div>
    //         </div>
    //       </CardContent>
    //     </Card>
    //   );
    // }

    // if (!isAuthenticated) {
    //   return (
    //     <Card className="w-full transition-shadow hover:shadow-lg border-warning border bg-warning/10">
    //       <CardContent className="py-6">
    //         <div className="text-center space-y-4">
    //           <div className="text-warning flex items-center justify-center gap-2">
    //             <svg ... />
    //             <span className="font-medium">
    //               Please log in to view payment information
    //             </span>
    //           </div>
    //           <Button ...>Log In</Button>
    //         </div>
    //       </CardContent>
    //     </Card>
    //   );
    // }

    return <WrappedComponent {...props} />;
  };
}
