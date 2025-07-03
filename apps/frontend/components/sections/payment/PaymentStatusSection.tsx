'use client';

import { useState, useEffect } from 'react';
import { PaymentStatusCard } from '@/components/features/payment/PaymentStatusCard';
import { TransactionHistory } from '@/components/features/payment/TransactionHistory';
import { withPaymentAuth } from './withPaymentAuth';
import { createPaymentService } from '@/services/payment.service';
import type { PaymentTransaction } from '@/services/payment.service';
import { auth } from '@/lib/firebase';

interface PaymentStatusSectionProps {
  className?: string;
  showTitle?: boolean;
}

const BasePaymentStatusSection = ({ 
  className = '',
  showTitle = true 
}: PaymentStatusSectionProps) => {
  const [transactions, setTransactions] = useState<PaymentTransaction[]>([]);
  const [loading, setLoading] = useState(true);
  const paymentService = createPaymentService();

  useEffect(() => {
    const fetchTransactions = async () => {
      try {
        const userTransactions = await paymentService.getTransactionHistory();
        setTransactions(userTransactions || []);
      } catch (error) {
        console.error('Failed to fetch transactions:', error);
      } finally {
        setLoading(false);
      }
    };

    const unsubscribe = auth.onAuthStateChanged((user) => {
      if (user) {
        fetchTransactions();
      } else {
        setTransactions([]);
        setLoading(false);
      }
    });

    return () => unsubscribe();
  }, []);

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
        <PaymentStatusCard />
        
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
