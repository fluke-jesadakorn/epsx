'use client';

import { useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { createPaymentService } from '@/services/payment.service';
import type { OrderDetails } from './types';

interface StatusMonitorProps {
  orderId: string;
  status: OrderDetails['status'];
  onStatusChange: (status: OrderDetails['status']) => void;
}

const STATUS_MESSAGES = {
  pending: 'Your payment is being processed...',
  processing: 'Verifying your payment...',
  completed: 'Payment successful! Redirecting to dashboard...',
  failed: 'Payment failed. Please try again.',
};

const STATUS_COLORS = {
  pending: 'text-yellow-500',
  processing: 'text-blue-500',
  completed: 'text-green-500',
  failed: 'text-red-500',
};

export default function StatusMonitor({ orderId, status, onStatusChange }: StatusMonitorProps) {
  const [isLoading, setIsLoading] = useState(false);
  const paymentService = createPaymentService();

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const paymentStatus = await paymentService.getPaymentStatus();
        
        if (paymentStatus?.hasPaid) {
          onStatusChange('completed');
          // Redirect to dashboard after successful payment
          setTimeout(() => {
            window.location.href = '/dashboard';
          }, 2000);
        }
      } catch (error) {
        console.error('Error checking payment status:', error);
      }
    };

    if (status === 'pending' || status === 'processing') {
      const interval = setInterval(checkStatus, 5000);
      return () => clearInterval(interval);
    }
  }, [status, orderId, onStatusChange]);

  const handleRetry = async () => {
    setIsLoading(true);
    try {
      const paymentStatus = await paymentService.getPaymentStatus();
      if (paymentStatus?.hasPaid) {
        onStatusChange('completed');
      } else {
        onStatusChange('pending');
      }
    } catch (error) {
      console.error('Error retrying payment:', error);
      onStatusChange('failed');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <h2 className="text-2xl font-bold">Payment Status</h2>
        <p className="text-muted-foreground">
          Order ID: {orderId}
        </p>
      </div>

      <Card className="p-6">
        <div className="flex flex-col items-center space-y-4">
          {/* Status Icon */}
          <div className={`text-4xl ${STATUS_COLORS[status]}`}>
            {status === 'completed' ? '✓' : 
             status === 'failed' ? '×' : 
             '⟳'}
          </div>

          {/* Status Message */}
          <div className="text-center">
            <p className={`text-lg font-medium ${STATUS_COLORS[status]}`}>
              {STATUS_MESSAGES[status]}
            </p>
          </div>

          {/* Action Button for Failed Status */}
          {status === 'failed' && (
            <Button
              onClick={handleRetry}
              disabled={isLoading}
              className="mt-4"
            >
              {isLoading ? 'Retrying...' : 'Retry Payment'}
            </Button>
          )}

          {/* Loading Spinner for Pending/Processing */}
          {(status === 'pending' || status === 'processing') && (
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          )}
        </div>
      </Card>

      {/* Support Information */}
      <div className="text-center text-sm text-muted-foreground">
        <p>Having issues? Contact our support team</p>
        <a 
          href="mailto:support@example.com"
          className="text-primary hover:underline"
        >
          support@example.com
        </a>
      </div>
    </div>
  );
}
