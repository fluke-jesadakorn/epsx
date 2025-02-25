'use client';

import { AuthForm } from "@/components/auth/AuthForm";
import { signInWithOAuth, signInWithEmailPassword } from "@/utils/auth";
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

export default function Login() {
  const router = useRouter();
  const { user } = useAuth();
  const { setLoading } = useLoading();
  const { setError } = useError();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleEmailPasswordLogin = async ({ email, password }: AuthFormValues) => {
    try {
      setIsSubmitting(true);
      setLoading(true);
      await signInWithEmailPassword({ email, password });
      router.push('/home');
    } catch (error) {
      console.error('Login error:', error);
      setError(error instanceof Error ? error : new Error('Login failed'));
    } finally {
      setIsSubmitting(false);
      setLoading(false);
    }
  };

  const handleOAuthLogin = async (provider: 'google' | 'github') => {
    try {
      setIsSubmitting(true);
      setLoading(true);
      const authProvider = provider === 'google' ? new GoogleAuthProvider() : new GithubAuthProvider();
      await signInWithOAuth(authProvider);
      router.push('/home');
    } catch (error) {
      console.error('OAuth login error:', error);
      setError(error instanceof Error ? error : new Error('OAuth login failed'));
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
          mode="login"
          onSubmit={handleEmailPasswordLogin}
          onOAuthClick={handleOAuthLogin}
          isSubmitting={isSubmitting}
        />
      </Suspense>
    </div>
  );
}
