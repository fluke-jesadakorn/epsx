import { requireGuest } from '@/app/actions/auth';
import { redirect } from 'next/navigation';
import { serverConfig } from '@/config/env';

interface ResetPasswordPageProps {
  searchParams: {
    token?: string;
    error?: string;
  };
}

export default async function ResetPasswordPage({ searchParams }: ResetPasswordPageProps) {
  // Security: Ensure only unauthenticated users can access password reset
  await requireGuest();

  const token = searchParams.token;

  // Security: Validate reset token presence to prevent unauthorized access
  if (!token) {
    redirect('/forgot-password?error=Invalid or missing reset token');
  }

  // Redirect to backend password reset confirmation with token
  const params = new URLSearchParams({
    token: token,
    redirect_to: `${serverConfig.siteUrl}/login?message=password_reset_complete`
  });

  const resetUrl = `${serverConfig.backendUrl}/oauth/reset-password/confirm?${params.toString()}`;
  
  redirect(resetUrl);
}