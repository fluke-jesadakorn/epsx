'use client';

import { PaymentStatusCard } from '@/components/features/payment/PaymentStatusCard';
import { withPaymentAuth } from './withPaymentAuth';

interface PaymentStatusSectionProps {
  className?: string;
  showTitle?: boolean;
}


const BasePaymentStatusSection = ({ 
  className = '',
  showTitle = true 
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
      <div className="space-y-4">
        <PaymentStatusCard />
      </div>
    </section>
  );
};

export const PaymentStatusSection = withPaymentAuth(BasePaymentStatusSection);
