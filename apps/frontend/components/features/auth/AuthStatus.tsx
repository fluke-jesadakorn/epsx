"use client";

import { useEffect, memo } from "react";
import { useAuth } from "@/context/auth-context";

function AuthStatusComponent() {
  const { checkStatus } = useAuth();

  useEffect(() => {
    // Update auth status when component mounts
    checkStatus();

    // Set up interval to periodically check auth status (every 5 minutes)
    const interval = setInterval(checkStatus, 5 * 60 * 1000);
    
    return () => clearInterval(interval);
  }, [checkStatus]);

  // Component doesn't need to render anything visible
  return null;
}

// Memoize the component to prevent unnecessary re-renders
export default memo(AuthStatusComponent);
