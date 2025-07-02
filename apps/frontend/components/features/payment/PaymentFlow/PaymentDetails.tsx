'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { createPaymentService } from '@/services/payment.service';
import type { Package, PaymentMethod, OrderDetails } from './types';

interface PaymentDetailsProps {
  package: Package;
  method: PaymentMethod;
  onSubmit: (orderDetails: OrderDetails) => void;
}

export default function PaymentDetails({ package: pkg, method, onSubmit }: PaymentDetailsProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const paymentService = createPaymentService();

  const handleSubmit = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // For cryptocurrency payments
      if (method === 'on_chain') {
        const paymentDetails = await paymentService.getPaymentDetails('TRC20');

        if (!paymentDetails) {
          throw new Error('Failed to get payment details');
        }

        const orderDetails: OrderDetails = {
          id: paymentDetails.paymentStatus.transactionId,
          status: 'pending',
          amount: pkg.price,
          currency: pkg.currency,
          paymentMethod: method,
          receiveAddress: paymentDetails.walletAddress,
          createdAt: new Date()
        };

        onSubmit(orderDetails);
      } 
      // For online payments
      else {
        const response = await fetch('/api/v1/payment/create', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            amount: pkg.price,
            currency: pkg.currency,
            payment_method: method,
            product_name: pkg.name
          })
        });

        if (!response.ok) {
          throw new Error('Failed to create payment');
        }

        const data = await response.json();
        const orderDetails: OrderDetails = {
          id: data.order_no,
          status: 'pending',
          amount: pkg.price,
          currency: pkg.currency,
          paymentMethod: method,
          checkoutUrl: data.checkout_url,
          createdAt: new Date()
        };

        onSubmit(orderDetails);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to process payment');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <h2 className="text-2xl font-bold">Payment Details</h2>
        <p className="text-muted-foreground">
          Review your package selection and proceed with payment
        </p>
      </div>

      <Card className="p-4">
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <div>
              <h3 className="font-semibold">{pkg.name}</h3>
              <p className="text-sm text-muted-foreground">{pkg.description}</p>
            </div>
            <div className="text-right">
              <p className="text-lg font-bold">${pkg.price}</p>
              <p className="text-sm text-muted-foreground">{pkg.currency}</p>
            </div>
          </div>

          <div className="border-t pt-4">
            <p className="text-sm font-medium">Payment Method</p>
            <p className="text-sm text-muted-foreground">
              {method === 'on_chain' ? 'Cryptocurrency' : 'Online Payment'}
            </p>
          </div>
        </div>
      </Card>

      {error && (
        <div className="text-sm text-red-500">
          {error}
        </div>
      )}

      <div className="flex gap-4">
        <Button
          variant="outline"
          onClick={() => window.history.back()}
          className="flex-1"
        >
          Back
        </Button>
        <Button
          onClick={handleSubmit}
          disabled={isLoading}
          className="flex-1"
        >
          {isLoading ? 'Processing...' : 'Confirm Payment'}
        </Button>
      </div>
    </div>
  );
}
