"use client";

import { createContext, useContext, useEffect, useState } from "react";
import { auth } from "@/lib/firebase-client";
import { onAuthStateChanged, User as FirebaseUser } from "firebase/auth";
import type { Role, Permission } from "@/constants/roles";
import { useRouter } from "next/navigation";
import { signOut } from "@/utils/auth-client";
import { logout as serverLogout } from "@/app/actions/auth";

export type AuthUser = {
  firebaseUser: FirebaseUser | null;
  role?: Role;
  permissions?: Permission[];
  accessLevel?: number;
};

export type AuthContextType = {
  user: AuthUser | null;
  loading: boolean;
  logout: () => Promise<void>;
};

const AuthContext = createContext<AuthContextType>({
  user: null,
  loading: true,
  logout: async () => {},
});

export const AuthProvider = ({ children }: { children: React.ReactNode }) => {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [loading, setLoading] = useState(true);
  const router = useRouter();

  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (firebaseUser) => {
      if (firebaseUser) {
        // Get ID token to fetch session data
        const idToken = await firebaseUser.getIdToken();
        
        try {
          // Fetch session data including roles and permissions
          const response = await fetch('/api/auth/session', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify({ idToken }),
          });

          if (response.ok) {
            const { success, data } = await response.json();
            if (success && data?.user) {
              setUser({
                firebaseUser,
                role: data.user.role,
                permissions: data.user.permissions,
                accessLevel: data.user.accessLevel,
              });
            }
          } else {
            setUser({ firebaseUser });
          }
        } catch (error) {
          console.error('Error fetching session:', error);
          setUser({ firebaseUser });
        }
      } else {
        setUser(null);
      }
      
      setLoading(false);
      router.refresh();
    });

    return () => unsubscribe();
  }, [router]);

  const logout = async () => {
    // First sign out from Firebase
    await signOut();
    // Then call server action to clear session
    const result = await serverLogout();
    if (result.success && result.redirectUrl) {
      router.push(result.redirectUrl);
    }
  };

  return (
    <AuthContext.Provider value={{ user, loading, logout }}>
      {!loading && children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);
