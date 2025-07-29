import { requireAuth } from '@/lib/server-auth';
import { EnterprisePaymentClient } from './EnterprisePaymentClient';

export default async function EnterprisePaymentPage() {
  // Server-side auth check - redirect to login if not authenticated
  await requireAuth('/payment/enterprise');

  return <EnterprisePaymentClient />;
}
