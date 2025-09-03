import { getCurrentUser } from '@/lib/server-actions';
import { redirect } from 'next/navigation';
import { QuickPaymentClient } from './QuickPaymentClient';

export const dynamic = 'force-dynamic';

interface QuickPaymentPageProps {
  searchParams: { package?: string; amount?: string };
}

export default async function QuickPaymentPage({ searchParams }: QuickPaymentPageProps) {
  // Server-side auth check - redirect to login if not authenticated
  const user = await getCurrentUser();

  if (!user) {
    const { redirectToBackendLogin } = await import('@/lib/server/auth');
    redirectToBackendLogin('/payment/quick');
  }
  
  // Extract parameters from search params
  const pkg = searchParams.package || '';
  const amt = searchParams.amount || '';

  return <QuickPaymentClient pkg={pkg} amt={amt} />;
}
