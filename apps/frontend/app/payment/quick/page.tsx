import { requireAuth } from '@/lib/server-auth';
import { QuickPaymentClient } from './QuickPaymentClient';

interface QuickPaymentPageProps {
  searchParams: { package?: string; amount?: string };
}

export default async function QuickPaymentPage({ searchParams }: QuickPaymentPageProps) {
  // Server-side auth check - redirect to login if not authenticated
  await requireAuth('/payment/quick');
  
  // Extract parameters from search params
  const pkg = searchParams.package || '';
  const amt = searchParams.amount || '';

  return <QuickPaymentClient pkg={pkg} amt={amt} />;
}
