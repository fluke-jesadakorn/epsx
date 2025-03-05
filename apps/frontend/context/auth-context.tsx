"use client";

import { createContext, useContext, useCallback, useState, useEffect, ReactNode } from "react";
import { getAuthStatus } from "@/app/actions/getAuthStatus";

interface AuthContextType {
  isLoggedIn: boolean;
  userEmail: string | null;
  isAdmin: boolean;
  updateAuthStatus: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType>({
  isLoggedIn: false,
  userEmail: null,
  isAdmin: false,
  updateAuthStatus: async () => {},
});

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [userEmail, setUserEmail] = useState<string | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);

  const updateAuthStatus = useCallback(async () => {
    try {
      const status = await getAuthStatus();
      setIsLoggedIn(status.isAuthenticated);
      setUserEmail(status.email);
      setIsAdmin(status.role === "admin");
    } catch (error) {
      console.error("Failed to update auth status:", error);
      setIsLoggedIn(false);
      setUserEmail(null);
      setIsAdmin(false);
    }
  }, []);

  useEffect(() => {
    updateAuthStatus();
  }, [updateAuthStatus]);

  return (
    <AuthContext.Provider
      value={{
        isLoggedIn,
        userEmail,
        isAdmin,
        updateAuthStatus,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
