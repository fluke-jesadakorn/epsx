'use client';

import { useState } from 'react';
import { Card } from '@/components/ui/card';
import { createPaymentService } from '@/services/payment.service';
import type { USDTDetails } from '@/types/userLevel';

interface PaymentFormProps {
  selectedAsset: string;
  amount: string;
  packageType: string;
  onSubmit: (details: any) => Promise<void>;
}

export default function PaymentForm({
  selectedAsset,
  amount,
  packageType,
  onSubmit,
}: PaymentFormProps) {
  const [loading, setLoading] = useState(false);
  const [paymentDetails, setPaymentDetails] = useState<USDTDetails | null>(
    null,
  );
  const paymentService = createPaymentService();

  const network = selectedAsset.split('_')[1] || 'TRX';

  // Fetch payment details on mount
  useState(() => {
    const fetchPaymentDetails = async () => {
      try {
        const details = await paymentService.getPaymentDetails('', network);
        if (details) {
          setPaymentDetails(details);
        }
      } catch (error) {
        console.error('Error fetching payment details:', error);
      }
    };

    fetchPaymentDetails();
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      await onSubmit({
        asset: selectedAsset,
        amount,
        network,
      });
    } catch (error) {
      console.error('Payment submission error:', error);
    } finally {
      setLoading(false);
    }
  };

  if (!paymentDetails) {
    return (
      <div className="flex justify-center items-center min-h-[200px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <Card className="p-6">
      <form onSubmit={handleSubmit}>
        <div className="mb-6">
          <h3 className="text-lg font-semibold mb-2">Payment Summary</h3>
          <div className="grid gap-2 text-sm">
            <div className="flex justify-between py-2 border-b">
              <span className="text-muted-foreground">Package</span>
              <span className="font-medium">{packageType}</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span className="text-muted-foreground">Amount</span>
              <span className="font-medium">${amount} USD</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span className="text-muted-foreground">Payment Method</span>
              <span className="font-medium">{selectedAsset}</span>
            </div>
          </div>
        </div>

        <div className="mb-6">
          <h3 className="text-lg font-semibold mb-4">Payment Instructions</h3>
          <div className="bg-muted/50 rounded-lg p-4 mb-4">
            <p className="text-sm text-muted-foreground mb-4">
              Please send exactly{' '}
              <span className="font-bold">${amount} USD</span> worth of{' '}
              {selectedAsset.split('_')[0]} to the following address:
            </p>
            <div className="bg-background p-3 rounded border mb-2">
              <code className="text-xs break-all">
                {paymentDetails.walletAddress}
              </code>
            </div>
            {paymentDetails.tag && (
              <>
                <p className="text-sm text-muted-foreground mt-4 mb-2">
                  Memo/Tag (Required):
                </p>
                <div className="bg-background p-3 rounded border">
                  <code className="text-xs break-all">
                    {paymentDetails.tag}
                  </code>
                </div>
              </>
            )}
          </div>

          <div className="flex justify-center mb-6">
            <div className="w-48 h-48 bg-white rounded-lg p-2">
              <img
                src={paymentDetails.qrCodePath}
                alt="Payment QR Code"
                className="w-full h-full object-contain"
              />
            </div>
          </div>

          <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-100 dark:border-yellow-900/50 rounded-lg p-4 mb-6">
            <p className="text-sm text-yellow-800 dark:text-yellow-200">
              Important: Please make sure to send the exact amount using the
              correct network ({network}). Sending through a different network
              may result in loss of funds.
            </p>
          </div>
        </div>

        <button
          type="submit"
          disabled={loading}
          className={`w-full py-3 rounded-md bg-primary text-primary-foreground ${
            loading ? 'opacity-50 cursor-not-allowed' : 'hover:opacity-90'
          }`}
        >
          {loading ? (
            <div className="flex items-center justify-center gap-2">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-foreground"></div>
              <span>Processing...</span>
            </div>
          ) : (
            'Confirm Payment'
          )}
        </button>
      </form>
    </Card>
  );
}
