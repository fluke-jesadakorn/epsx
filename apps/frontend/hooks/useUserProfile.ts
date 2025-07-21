// apps/frontend/hooks/useUserProfile.ts
import { useEffect, useState } from 'react';
import { doc, getDoc } from 'firebase/firestore';
import { db } from '@/lib/firebase';
import { useAuth } from '@/context/auth-context';

interface UserProfile {
  userLevel?: string;
  numericLevel?: number;
  rankingLimit?: number;
  paymentStatus?: {
    expirationDate?: any;
    hasPaid?: boolean;
  };
  [key: string]: any;
}

export function useUserProfile() {
  const { user } = useAuth();
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!user) {
      setProfile(null);
      setLoading(false);
      return;
    }
    setLoading(true);
    const fetchProfile = async () => {
      const ref = doc(db, 'users', user.uid);
      const snap = await getDoc(ref);
      if (snap.exists()) {
        setProfile(snap.data() as UserProfile);
      } else {
        setProfile(null);
      }
      setLoading(false);
    };
    fetchProfile();
  }, [user]);

  return { profile, loading };
}
