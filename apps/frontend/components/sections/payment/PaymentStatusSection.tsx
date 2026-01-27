'use client';

import { PaymentStatusCard } from '@/components/features/payment/PaymentStatusCard';
import { TransactionHistory } from '@/components/features/payment/TransactionHistory';
import { usePaymentAuth, PaymentAuthLoadingUI, PaymentAuthRequiredUI, PaymentAccessRequiredUI } from './usePaymentAuth';

// Transaction interface that matches what TransactionHistory expects
interface Transaction {
  orderNo: string;
  actualAmount: number;
  currency: string;
  status: string;
  finishTime: string;
  blockchainData: {
    txHash: string;
    network: string;
  };
  blockExplorerUrl: string;
}

interface PaymentStatusSectionProps {
  className?: string;
  showTitle?: boolean;
  transactions?: Transaction[];
  error?: string | null;
}

// Helper to derive the overall payment status from transactions
const getOverallPaymentStatus = (transactions: Transaction[]): 'pending' | 'completed' | 'failed' | 'processing' => {
  if (transactions.length === 0) return 'pending';

  // Get the most recent transaction
  const latestTransaction = transactions[0];
  const status = latestTransaction.status.toLowerCase();

  // Map transaction status to PaymentStatusCard status
  if (status === 'succeeded' || status === 'completed' || status === 'paid' || status === 'success') {
    return 'completed';
  } else if (status === 'failed' || status === 'cancelled' || status === 'expired') {
    return 'failed';
  } else if (status === 'processing') {
    return 'processing';
  }
  return 'pending';
};

// Helper to get latest transaction details for display
const getLatestTransactionDetails = (transactions: Transaction[]) => {
  if (transactions.length === 0) return {};
  const latest = transactions[0];
  return {
    transactionId: latest.blockchainData?.txHash || latest.orderNo,
    amount: latest.actualAmount,
    currency: latest.currency,
    timestamp: latest.finishTime,
  };
};

export function PaymentStatusSection({
  className = '',
  showTitle = true,
  transactions = [],
  error = null
}: PaymentStatusSectionProps) {
  const { isLoading, isAuthenticated, hasPaymentAccess, user } = usePaymentAuth();

  if (isLoading) return <PaymentAuthLoadingUI />;
  if (!isAuthenticated) return <PaymentAuthRequiredUI />;
  if (!hasPaymentAccess) return <PaymentAccessRequiredUI user={user} />;

  // Derive status from transactions instead of hardcoding
  const paymentStatus = getOverallPaymentStatus(transactions);
  const transactionDetails = getLatestTransactionDetails(transactions);

  return (
    <section className={`w-full ${className}`}>
      {showTitle && (
        <div className="mb-6">
          <h2 className="text-2xl font-semibold text-primary">Payment Information</h2>
          <p className="text-muted-foreground mt-1">
            View your current payment status and subscription details
          </p>
        </div>
      )}
      <div className="space-y-8">
        <PaymentStatusCard
          status={paymentStatus}
          transactionId={transactionDetails.transactionId}
          amount={transactionDetails.amount}
          currency={transactionDetails.currency}
          timestamp={transactionDetails.timestamp}
        />

        {/* Transaction History */}
        <div id="history">
          <h3 className="text-xl font-semibold mb-4">Transaction History</h3>
          {error ? (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <p className="text-red-600 dark:text-red-400">{error}</p>
            </div>
          ) : (
            <TransactionHistory
              transactions={transactions}
              className="bg-card"
            />
          )}
        </div>
      </div>
    </section>
  );
}
