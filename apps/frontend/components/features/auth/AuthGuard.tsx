"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/context/auth-context";

interface AuthGuardProps {
  children: React.ReactNode;
  requireAdmin?: boolean;
}

export function AuthGuard({ children, requireAdmin = false }: AuthGuardProps) {
  const { isLoggedIn, isAdmin } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoggedIn) {
      router.push("/login");
      return;
    }

    if (requireAdmin && !isAdmin) {
      router.push("/unauthorized");
      return;
    }
  }, [isLoggedIn, isAdmin, requireAdmin, router]);

  if (!isLoggedIn || (requireAdmin && !isAdmin)) {
    return null;
  }

  return <>{children}</>;
}

export default AuthGuard;
