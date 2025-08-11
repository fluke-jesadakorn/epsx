'use client';

import { FcGoogle } from "react-icons/fc";
import { signIn } from 'next-auth/react';
import { useState } from 'react';

import { Button } from "@/components/ui/button";

export function GoogleSignIn() {
  const [loading, setLoading] = useState(false);

  const handleSignIn = async () => {
    try {
      setLoading(true);
      await signIn('google', { redirect: true, callbackUrl: '/dashboard' });
    } catch (error) {
      console.error("Google sign-in error:", error);
    } finally {
      setLoading(false);
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
