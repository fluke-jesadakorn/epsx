"use client";

import { AuthForm } from "@/components/auth/AuthForm";
import { handleOAuthCallback } from "../actions/auth-server";
import { signInWithEmail, signInWithOAuth } from "@/app/actions/auth-server";
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
          const response = await handleOAuthCallback(code, state);
          // Check for redirect URL from the callback
          if (response?.redirect && !response.redirect.includes('/login')) {
            // Prevent redirect loops by checking the URL
            window.location.href = response.redirect;
          }
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
      const response = await signInWithEmail(formData);
      // Check for redirect URL from the response
      if (response?.redirect && !response.redirect.includes('/login')) {
        // Prevent redirect loops by checking the URL
        window.location.href = response.redirect;
      }
    } catch (error) {
      console.error("Login error:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleOAuthLogin = async (provider: "google" | "github") => {
    try {
      setIsSubmitting(true);
      const providerId = provider === "google" ? "google.com" : "github.com";
      const redirectUrl = window.location.origin + "/login";
      const oauthRedirectUri = window.location.origin + "/home";
      const result = await signInWithOAuth(
        providerId,
        redirectUrl,
        oauthRedirectUri
      );
      if (result?.redirectUrl && !result.redirectUrl.includes('/login')) {
        // Use window.location for consistent cookie handling
        window.location.href = result.redirectUrl;
      }
    } catch (error) {
      console.error("OAuth login error:", error);
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
