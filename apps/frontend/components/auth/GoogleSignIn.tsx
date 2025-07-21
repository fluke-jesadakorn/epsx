'use client';

import { FcGoogle } from "react-icons/fc";

import { Button } from "@/components/ui/button";
import { useAuth } from "@/context/auth-context";

export function GoogleSignIn() {
  const { signInWithGoogle, loading } = useAuth();

  const handleSignIn = async () => {
    try {
      await signInWithGoogle();
    } catch (error) {
      console.error("Google sign-in error:", error);
    }
  };

  return (
    <Button
      variant="outline"
      onClick={handleSignIn}
      disabled={loading}
      className="w-full flex items-center justify-center gap-2"
    >
      <FcGoogle className="h-5 w-5" />
      Sign in with Google
    </Button>
  );
}
