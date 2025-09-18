import { redirect } from 'next/navigation';
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';

interface ForgotPasswordPageProps {
  searchParams: {
    email?: string;
    success?: string;
    error?: string;
  };
}

export const dynamic = 'force-dynamic';

export default async function ForgotPasswordPage({ searchParams }: ForgotPasswordPageProps) {
  const awaitedSearchParams = await searchParams;
  
  // Get backend URL from environment
  const backendUrl = URL.get(Service.BACKEND, URLContext.SERVER);
  
  // Build reset password URL
  const params = new URLSearchParams({
    client_id: 'epsx-frontend',
  });
  
  // If there's an email, pass it along
  if (awaitedSearchParams.email) {
    params.set('email', awaitedSearchParams.email);
  }
  
  const resetUrl = `${backendUrl}/oauth/reset-password?${params.toString()}`;
  
  // Server-side redirect to backend password reset
  redirect(resetUrl);
}