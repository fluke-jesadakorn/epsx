'use client';
import { useState, useEffect } from 'react';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/context/auth-context';
import { createPaymentService } from '@/services/payment.service';

export function PaymentStatus() {
  const [paymentStatus, setPaymentStatus] = useState<{
    lastPaymentDate?: Date;
    expirationDate?: Date;
    userLevel?: string;
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const { user, loading: authLoading } = useAuth();

  const paymentService = createPaymentService();

  useEffect(() => {
    async function fetchPaymentStatus() {
      setLoading(true);
      if (!user) {
        setPaymentStatus(null);
        setLoading(false);
        return;
      }
      try {
        const status = await paymentService.getPaymentStatus();
        setPaymentStatus(status);
      } catch (error) {
        console.error('Detailed error fetching payment status:', error);
        setPaymentStatus(null);
      } finally {
        setLoading(false);
      }
    }
    if (!authLoading) {
      fetchPaymentStatus();
    }
  }, [user, authLoading]);

  if (loading || authLoading) {
    return (
      <div className="text-muted-foreground">Loading payment status...</div>
    );
  }

  if (!user) {
    return (
      <div className="space-y-4">
        <h3 className="text-lg font-medium">Payment Status</h3>
        <div className="text-yellow-500">
          Please log in to view your payment status and transaction history.
        </div>
      </div>
    );
  }

  if (!paymentStatus) {
    return (
      <div className="space-y-4">
        <h3 className="text-lg font-medium">Payment Status</h3>
        <div className="overflow-x-auto">
          <table className="min-w-full border border-gray-300 dark:border-gray-700">
            <thead>
              <tr className="bg-gray-100 dark:bg-gray-800">
                <th className="w-1/3 px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300 border-b border-gray-300 dark:border-gray-700">Detail</th>
                <th className="w-2/3 px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300 border-b border-gray-300 dark:border-gray-700">Value</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-gray-300 dark:border-gray-700">
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300"><Label>User Level:</Label></td>
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300">Free</td>
              </tr>
              <tr className="border-b border-gray-300 dark:border-gray-700">
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300"><Label>Transaction History:</Label></td>
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300">No transactions available.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium">Payment Status</h3>
      <div className="overflow-x-auto">
        <table className="min-w-full border border-gray-300 dark:border-gray-700">
          <thead>
            <tr className="bg-gray-100 dark:bg-gray-800">
              <th className="w-1/3 px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300 border-b border-gray-300 dark:border-gray-700">Detail</th>
              <th className="w-2/3 px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300 border-b border-gray-300 dark:border-gray-700">Value</th>
            </tr>
          </thead>
          <tbody>
            {paymentStatus.lastPaymentDate && (
              <tr className="border-b border-gray-300 dark:border-gray-700">
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300"><Label>Last Payment Date:</Label></td>
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300">{paymentStatus.lastPaymentDate.toLocaleDateString()}</td>
              </tr>
            )}
            {paymentStatus.expirationDate && (
              <tr className="border-b border-gray-300 dark:border-gray-700">
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300"><Label>Expiration Date:</Label></td>
                <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300">{paymentStatus.expirationDate.toLocaleDateString()}</td>
              </tr>
            )}
            <tr className="border-b border-gray-300 dark:border-gray-700">
              <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300"><Label>User Level:</Label></td>
              <td className="px-4 py-2 text-sm text-gray-700 dark:text-gray-300">{paymentStatus.userLevel && paymentStatus.userLevel !== 'Pending' ? paymentStatus.userLevel : 'Free'}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
