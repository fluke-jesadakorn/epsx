import { getPaymentHistoryAction } from '@/app/actions/payments';
import { AccessOverview } from '@/components/account/access-overview';
import { AccountClient } from '@/components/account/account-client';

export const dynamic = 'force-dynamic';

export default async function AccountPage() {
  // Fetch payment history with default pagination
  const limit = 10;
  const offset = 0;
  const historyResponse = await getPaymentHistoryAction({ limit, offset });

  let initialHistory = undefined;

  if (historyResponse.success && historyResponse.data) {
    const { payments, total } = historyResponse.data;

    // Transform to match PaymentHistoryData interface
    initialHistory = {
      payments: payments as any[], // Casting as shared API type might differ from Component expectation
      pagination: {
        page: 1,
        per_page: limit,
        total,
        total_pages: Math.ceil(total / limit)
      }
    };
  }

  return <AccountClient
    initialPaymentHistory={initialHistory}
    accessOverviewSlot={<AccessOverview />}
  />;
}