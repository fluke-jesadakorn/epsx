import { getCurrentUser } from '@/lib/server-actions';
import { redirect } from 'next/navigation';
import { EnterprisePaymentClient } from './EnterprisePaymentClient';

export const dynamic = 'force-dynamic';

export default async function EnterprisePaymentPage() {
  // Server-side auth check - redirect to login if not authenticated
  let user = null;
  try {
    const result = await getCurrentUser({});
    user = result?.success ? result.data : null;
  } catch (error) {
    console.error('EnterprisePaymentPage: Failed to get user:', error);
  }

  if (!user) {
    const { redirectToBackendLogin } = await import('@/lib/server/auth');
    redirectToBackendLogin('/payment/enterprise');
  }

  return <EnterprisePaymentClient />;
}
