'use client';

import { PaymentStatusCard } from '@/components/features/payment/PaymentStatusCard';
import { TransactionHistory } from '@/components/features/payment/TransactionHistory';
import { withPaymentAuth } from './withPaymentAuth';

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

const BasePaymentStatusSection = ({ 
  className = '',
  showTitle = true,
  transactions = [],
  error = null
}: PaymentStatusSectionProps) => {

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
        <PaymentStatusCard status="pending" />
        
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
};

export const PaymentStatusSection = withPaymentAuth(BasePaymentStatusSection);
