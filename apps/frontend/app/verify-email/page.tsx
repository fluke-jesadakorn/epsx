import type { Metadata } from 'next';
import { checkAuth } from '@/lib/auth';
import { EmailVerificationPage } from '@/components/auth/EmailVerificationPage';

export const metadata: Metadata = {
  title: 'Verify Email - EPSX',
  description: 'Verify your email address to access all features',
};

export default async function VerifyEmailPage() {
  // Ensure user is authenticated
  await checkAuth();

  return <EmailVerificationPage />;
}
