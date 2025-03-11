"use client";

import { AuthForm } from "@/components/auth/AuthForm";
import { handleOAuthCallback } from "../actions/auth-server";
import { signInWithEmail } from "@/app/actions/auth-server";
import { signInWithOAuth } from "@/utils/auth";
import { GoogleAuthProvider, GithubAuthProvider } from "firebase/auth";
import { LoadingForm } from "@/components/common/LoadingForm";
import { useRouter, useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";

interface AuthFormValues {
  email: string;
  password: string;
}

export default function Page() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const code = searchParams.get('code');
  const state = searchParams.get('state');
  const error = searchParams.get('error');

  useEffect(() => {
    const handleCallback = async () => {
      try {
        if (code && state) {
          const urlParams = new URLSearchParams();
          urlParams.set("code", code);
          urlParams.set("state", state);
          await handleOAuthCallback(urlParams);
          router.push("/home");
        }
      } catch (error) {
        console.error("Auth error:", error);
      }
    };

    handleCallback();
  }, [code, state, router]);

  // Handle errors from OAuth callback
  const authError = error 
    ? decodeURIComponent(error) 
    : undefined;

  // Show loading state during callback processing
  if (code && state) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingForm>Processing authentication...</LoadingForm>
      </div>
    );
  }

  const handleEmailPasswordLogin = async ({ email, password }: AuthFormValues) => {
    try {
      setIsSubmitting(true);
      const formData = new FormData();
      formData.append('email', email);
      formData.append('password', password);
      await signInWithEmail(formData);
      router.push("/home");
    } catch (error) {
      console.error("Login error:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleOAuthLogin = async (provider: "google" | "github") => {
    try {
      setIsSubmitting(true);
      const authProvider =
        provider === "google"
          ? new GoogleAuthProvider()
          : new GithubAuthProvider();
      await signInWithOAuth(authProvider);
      router.push("/home");
    } catch (error) {
      console.error("OAuth login error:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Show auth form
  return (
    <div className="min-h-screen flex items-center justify-center">
      <AuthForm
        mode="login"
        onSubmit={handleEmailPasswordLogin}
        onOAuthClick={handleOAuthLogin}
        isSubmitting={isSubmitting}
        error={authError}
      />
    </div>
  );
}
