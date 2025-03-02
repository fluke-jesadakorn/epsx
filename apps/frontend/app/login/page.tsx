"use client";

import { AuthForm } from "@/components/auth/AuthForm";
import { signInWithOAuth, signInWithEmailPassword } from "@/utils/auth";
import { useRouter, useSearchParams } from "next/navigation";
import { useState } from "react";
import { Suspense } from "react";
import { LoadingForm } from "@/components/common/LoadingForm";
import { GoogleAuthProvider, GithubAuthProvider } from "firebase/auth";

interface AuthFormValues {
  email: string;
  password: string;
}

function LoginContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [isSubmitting, setIsSubmitting] = useState(false);
  
  const redirectTo = searchParams.get("redirectTo") || "/home";

  const handleEmailPasswordLogin = async ({
    email,
    password,
  }: AuthFormValues) => {
    try {
      setIsSubmitting(true);
      await signInWithEmailPassword({ email, password });
      router.push(redirectTo);
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
      router.push(redirectTo);
    } catch (error) {
      console.error("OAuth login error:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center">
      <AuthForm
        mode="login"
        onSubmit={handleEmailPasswordLogin}
        onOAuthClick={handleOAuthLogin}
        isSubmitting={isSubmitting}
      />
    </div>
  );
}

export default function Login() {
  return (
    <Suspense fallback={<LoadingForm>Loading...</LoadingForm>}>
      <LoginContent />
    </Suspense>
  );
}
