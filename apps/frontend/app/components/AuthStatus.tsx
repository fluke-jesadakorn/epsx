"use client";

import { useEffect } from "react";
import { usePathname, useRouter } from "next/navigation";
import { useAuth } from "@/context/auth-context";

const publicRoutes = ["/login", "/register", "/privacy", "/terms"];

export default function AuthStatus() {
  const { user } = useAuth();
  const pathname = usePathname() || '';
  const router = useRouter();

  // Redirect to login if authentication is lost on protected routes
  useEffect(() => {
    if (!user && !publicRoutes.includes(pathname)) {
      router.push("/login");
    }
  }, [user, pathname, router]);

  return null;
}
