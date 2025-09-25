import { getCurrentUser } from '@/lib/server-actions';
import { redirect } from 'next/navigation';
import { EnterprisePaymentClient } from './EnterprisePaymentClient';

export const dynamic = 'force-dynamic';

export default async function EnterprisePaymentPage() {
  // Server-side auth check - redirect to login if not authenticated
  const user = await getCurrentUser();

  if (!user) {
    redirect('/login?redirectTo=/payment/enterprise');
  }

  return <EnterprisePaymentClient />;
}
