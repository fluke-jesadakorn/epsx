"use client";

import { useEffect } from "react";
import { useAuth } from "@/context/auth-context";

export default function AuthStatus() {
  const { isLoggedIn, userEmail, isAdmin, updateAuthStatus } = useAuth();

  useEffect(() => {
    // Update auth status when component mounts
    updateAuthStatus();

    // Set up interval to periodically check auth status
    const interval = setInterval(updateAuthStatus, 60000); // Check every minute
    
    return () => clearInterval(interval);
  }, [updateAuthStatus]);

  // Component doesn't need to render anything visible
  return null;
}
