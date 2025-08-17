import { getCurrentUser } from '@/lib/server-actions';
import { redirect } from 'next/navigation';
import { QuickPaymentClient } from './QuickPaymentClient';

interface QuickPaymentPageProps {
  searchParams: { package?: string; amount?: string };
}

export default async function QuickPaymentPage({ searchParams }: QuickPaymentPageProps) {
  // Server-side auth check - redirect to login if not authenticated
  let user = null;
  try {
    const result = await getCurrentUser({});
    user = result?.success ? result.data : null;
  } catch (error) {
    console.error('QuickPaymentPage: Failed to get user:', error);
  }

  if (!user) {
    redirect('/login?callbackUrl=/payment/quick');
  }
  
  // Extract parameters from search params
  const pkg = searchParams.package || '';
  const amt = searchParams.amount || '';

  return <QuickPaymentClient pkg={pkg} amt={amt} />;
}
