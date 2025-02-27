"use client";

import { AuthForm } from "@/components/auth/AuthForm";
import { signInWithOAuth, signUpWithEmailPassword } from "@/utils/auth";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Suspense } from "react";
import { LoadingForm } from "@/components/common/LoadingForm";
import { GoogleAuthProvider, GithubAuthProvider } from "firebase/auth";

interface AuthFormValues {
  email: string;
  password: string;
}

export default function Register() {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleEmailPasswordRegister = async ({
    email,
    password,
  }: AuthFormValues) => {
    try {
      setIsSubmitting(true);
      await signUpWithEmailPassword({ email, password });
      router.push("/confirmation-pending");
    } catch (error) {
      console.error("Registration error:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleOAuthRegister = async (provider: "google" | "github") => {
    try {
      setIsSubmitting(true);
      const authProvider =
        provider === "google"
          ? new GoogleAuthProvider()
          : new GithubAuthProvider();
      await signInWithOAuth(authProvider);
      router.push("/home");
    } catch (error) {
      console.error("OAuth registration error:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center">
      <Suspense fallback={<LoadingForm>Loading...</LoadingForm>}>
        <AuthForm
          mode="register"
          onSubmit={handleEmailPasswordRegister}
          onOAuthClick={handleOAuthRegister}
          isSubmitting={isSubmitting}
        />
      </Suspense>
    </div>
  );
}
