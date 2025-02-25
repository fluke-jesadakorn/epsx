'use client';

import { AuthForm } from "@/components/auth/AuthForm";
import { signInWithOAuth, signUpWithEmailPassword } from "@/utils/auth";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Suspense } from "react";
import { LoadingForm } from "@/components/common/LoadingForm";
import { GoogleAuthProvider, GithubAuthProvider } from "firebase/auth";
import { useAuth } from "@/hooks/useAuth";
import { useLoading } from "@/contexts/LoadingContext";
import { useError } from "@/contexts/ErrorContext";

interface AuthFormValues {
  email: string;
  password: string;
}

export default function Register() {
  const router = useRouter();
  const { user } = useAuth();
  const { setLoading } = useLoading();
  const { setError } = useError();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleEmailPasswordRegister = async ({ email, password }: AuthFormValues) => {
    try {
      setIsSubmitting(true);
      setLoading(true);
      await signUpWithEmailPassword({ email, password });
      router.push('/confirmation-pending');
    } catch (error) {
      console.error('Registration error:', error);
      setError(error instanceof Error ? error : new Error('Registration failed'));
    } finally {
      setIsSubmitting(false);
      setLoading(false);
    }
  };

  const handleOAuthRegister = async (provider: 'google' | 'github') => {
    try {
      setIsSubmitting(true);
      setLoading(true);
      const authProvider = provider === 'google' ? new GoogleAuthProvider() : new GithubAuthProvider();
      await signInWithOAuth(authProvider);
      router.push('/home');
    } catch (error) {
      console.error('OAuth registration error:', error);
      setError(error instanceof Error ? error : new Error('OAuth registration failed'));
    } finally {
      setIsSubmitting(false);
      setLoading(false);
    }
  };

  if (user) {
    router.push('/home');
    return null;
  }

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
