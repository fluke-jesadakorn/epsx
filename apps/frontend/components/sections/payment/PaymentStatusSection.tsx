'use client';

import { useState, useEffect } from 'react';
import { PaymentStatusCard } from '@/components/features/payment/PaymentStatusCard';
import { TransactionHistory } from '@/components/features/payment/TransactionHistory';
import { withPaymentAuth } from './withPaymentAuth';
import { createPaymentService } from '@/services/payment.service';
import { auth } from '@/lib/firebase';

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
}

const BasePaymentStatusSection = ({ 
  className = '',
  showTitle = true 
}: PaymentStatusSectionProps) => {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const paymentService = createPaymentService();

  useEffect(() => {
    const fetchTransactions = async () => {
      try {
        const userTransactions = await paymentService.getTxHistory();
        // Map PaymentTx to Transaction format expected by TransactionHistory
        const mappedTransactions: Transaction[] = userTransactions.map(tx => ({
          ...tx,
          actualAmount: tx.amount, // Map amount to actualAmount
        }));
        setTransactions(mappedTransactions || []);
      } catch (error) {
        console.error('Failed to fetch transactions:', error);
      }
    };

    const unsubscribe = auth.onAuthStateChanged((user) => {
      if (user) {
        fetchTransactions();
      } else {
        setTransactions([]);
      }
    });

    return () => unsubscribe();
  }, [paymentService]);

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
          <TransactionHistory 
            transactions={transactions}
            className="bg-card"
          />
        </div>
      </div>
    </section>
  );
};

export const PaymentStatusSection = withPaymentAuth(BasePaymentStatusSection);
