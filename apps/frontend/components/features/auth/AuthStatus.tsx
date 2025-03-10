"use client";

import { useEffect, memo } from "react";
import { useAuth } from "@/context/auth-context";

function AuthStatusComponent() {
  const { updateAuthStatus } = useAuth();

  useEffect(() => {
    // Update auth status when component mounts
    updateAuthStatus();

    // Set up interval to periodically check auth status (every 5 minutes)
    const interval = setInterval(updateAuthStatus, 5 * 60 * 1000);
    
    return () => clearInterval(interval);
  }, [updateAuthStatus]);

  // Component doesn't need to render anything visible
  return null;
}

// Memoize the component to prevent unnecessary re-renders
export default memo(AuthStatusComponent);
