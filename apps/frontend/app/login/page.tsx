"use client";

import { AuthForm } from "@/components/auth/AuthForm";
import { signInWithOAuth, signInWithEmailPassword } from "@/utils/auth";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Suspense } from "react";
import { LoadingForm } from "@/components/common/LoadingForm";
import { GoogleAuthProvider, GithubAuthProvider } from "firebase/auth";

interface AuthFormValues {
  email: string;
  password: string;
}

export default function Login() {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleEmailPasswordLogin = async ({
    email,
    password,
  }: AuthFormValues) => {
    try {
      setIsSubmitting(true);
      await signInWithEmailPassword({ email, password });
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

  return (
    <div className="min-h-screen flex items-center justify-center">
      <Suspense fallback={<LoadingForm>Loading...</LoadingForm>}>
        <AuthForm
          mode="login"
          onSubmit={handleEmailPasswordLogin}
          onOAuthClick={handleOAuthLogin}
          isSubmitting={isSubmitting}
        />
      </Suspense>
    </div>
  );
}
