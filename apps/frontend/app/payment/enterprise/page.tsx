import { getCurrentUser } from '@epsx/server-actions';
import { redirect } from 'next/navigation';
import { EnterprisePaymentClient } from './EnterprisePaymentClient';

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
    redirect('/login?callbackUrl=/payment/enterprise');
  }

  return <EnterprisePaymentClient />;
}
