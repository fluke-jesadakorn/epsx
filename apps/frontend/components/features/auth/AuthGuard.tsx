"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/context/auth-context";

interface AuthGuardProps {
  children: React.ReactNode;
  requireAdmin?: boolean;
}

export function AuthGuard({ 
  children, 
  // @ts-ignore - Admin check temporarily disabled, will be implemented later
  requireAdmin = false 
}: AuthGuardProps) {
  const { user } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!user) {
      router.push("/login");
      return;
    }
    // TODO: Implement proper admin check when admin functionality is added
    // For now, we'll skip the admin check
  }, [user, router]);

  if (!user) {
    return null;
  }

  return <>{children}</>;
}

export default AuthGuard;
