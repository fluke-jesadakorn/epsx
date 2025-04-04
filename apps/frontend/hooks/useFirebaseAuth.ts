'use client';

import { useEffect, useState } from 'react';
import { watchAuthState, type FirebaseUser } from '@/lib/firebase-client';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/providers/AuthProvider';

export function useFirebaseAuth() {
  const [firebaseUser, setFirebaseUser] = useState<FirebaseUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();
  const { session } = useAuth();

  useEffect(() => {
    // Subscribe to Firebase auth state changes
    const unsubscribe = watchAuthState((user) => {
      setFirebaseUser(user);
      setIsLoading(false);

      // If user is logged out in Firebase but we have a session, clear it
      if (!user && session) {
        router.push('/login');
      }
    });

    // Cleanup subscription
    return () => unsubscribe();
  }, [router, session]);

  return {
    user: firebaseUser,
    isLoading,
    isAuthenticated: !!firebaseUser && !!session,
  };
}

export type { FirebaseUser };
