'use client';

import {
  createContext,
  useContext,
  useEffect,
  useState,
} from "react";
import type { ReactNode } from "react";
import {
  signInWithPopup,
  signInWithEmailAndPassword as signInWithEmailPwd,
  createUserWithEmailAndPassword,
  GoogleAuthProvider,
  signOut as firebaseSignOut,
  onIdTokenChanged,
} from "firebase/auth";
import type { User } from "firebase/auth";
import { auth } from "@/lib/firebase";
import { handleSignIn, handleSignOut } from "@/app/actions/auth";

interface AuthContextType {
  user: User | null;
  loading: boolean;
  error: string | null;
  signInWithGoogle: () => Promise<void>;
  signInWithEmailPassword: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Listen for token changes
  useEffect(() => {
    const unsubscribe = onIdTokenChanged(auth, async (user) => {
      if (user) {
        try {
          const idToken = await user.getIdToken();
          // Use server action to create session
          await handleSignIn(idToken);
          setUser(user);
        } catch (error) {
          console.error("Session sync error:", error);
          setError("Failed to sync session");
        }
      } else {
        setUser(null);
      }
      setLoading(false);
    });

    return () => unsubscribe();
  }, []);

  const signInWithGoogle = async () => {
    try {
      setError(null);
      const provider = new GoogleAuthProvider();
      await signInWithPopup(auth, provider);
    } catch (error) {
      console.error("Google sign-in error:", error);
      setError("Failed to sign in with Google");
      throw error;
    }
  };

  const signInWithEmailPassword = async (email: string, password: string) => {
    try {
      setError(null);
      await signInWithEmailPwd(auth, email, password);
    } catch (error) {
      console.error("Email/Password sign-in error:", error);
      setError("Failed to sign in with email/password");
      throw error;
    }
  };

  const signUp = async (email: string, password: string) => {
    try {
      setError(null);
      await createUserWithEmailAndPassword(auth, email, password);
    } catch (error) {
      console.error("Sign-up error:", error);
      setError("Failed to create account");
      throw error;
    }
  };

  const signOut = async () => {
    try {
      setError(null);
      await firebaseSignOut(auth);
      // Use server action to clear session
      await handleSignOut();
    } catch (error) {
      console.error("Sign-out error:", error);
      setError("Failed to sign out");
      throw error;
    }
  };

  const value = {
    user,
    loading,
    error,
    signInWithGoogle,
    signInWithEmailPassword,
    signUp,
    signOut,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
