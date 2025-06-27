"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";

import { useAuth } from "@/context/auth-context";

interface AuthGuardProps {
  children: React.ReactNode;
}

export function AuthGuard({ 
  children
}: AuthGuardProps) {
  const { user } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!user) {
      router.push("/login");
      return;
    }
    // Check if user is authenticated
  }, [user, router]);

  if (!user) {
    return null;
  }

  return <>{children}</>;
}

export default AuthGuard;
