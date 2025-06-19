'use client';

import { useEffect, useState } from 'react';
import { watchAuthState, type FirebaseUser } from '@/lib/firebase';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/auth-context';

export function useFirebaseAuth() {
  const [firebaseUser, setFirebaseUser] = useState<FirebaseUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();
  const { user: authUser } = useAuth();

  useEffect(() => {
    // Subscribe to Firebase auth state changes
    const unsubscribe = watchAuthState((user) => {
      setFirebaseUser(user);
      setIsLoading(false);

      // If user is logged out in Firebase but we have an auth session, clear it
      if (!user && authUser) {
        router.push('/login');
      }
    });

    // Cleanup subscription
    return () => unsubscribe();
  }, [router, authUser]);

  return {
    user: firebaseUser,
    isLoading,
    isAuthenticated: !!firebaseUser && !!authUser,
  };
}

export type { FirebaseUser };
