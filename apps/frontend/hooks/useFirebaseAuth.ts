'use client';

import { useEffect, useState } from 'react';
import { watchAuthState, type FirebaseUser } from '@/lib/firebase';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/auth-context';

export function useFirebaseAuth() {
  const [fbUser, setFbUser] = useState<FirebaseUser | null>(null);
  const [loading, setLoading] = useState(true);
  const router = useRouter();
  const { user: authUser } = useAuth();

  useEffect(() => {
    // Subscribe to Firebase auth state changes
    const unsubscribe = watchAuthState((user) => {
      setFbUser(user);
      setLoading(false);

      // If user is logged out in Firebase but we have an auth session, clear it
      if (!user && authUser) {
        router.push('/login');
      }
    });

    // Cleanup subscription
    return () => unsubscribe();
  }, [router, authUser]);

  return {
    user: fbUser,
    loading,
    isAuth: !!fbUser && !!authUser,
  };
}

export type { FirebaseUser };
