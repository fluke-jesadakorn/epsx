'use client';

import Image from 'next/image';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';

interface CompletePaymentProps {
  qrCodeUrl: string | undefined;
  walletAddress: string | undefined;
  tag: string | undefined;
  paymentId: string | undefined;
  isLoading: boolean;
  onBack: () => void;
  onSubmit: (e: React.FormEvent) => void;
}

export function CompletePayment({ qrCodeUrl, walletAddress, tag, paymentId, isLoading, onBack, onSubmit }: CompletePaymentProps) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium">Step 3: Complete Payment</h3>
      {qrCodeUrl ? (
        <div className="mt-2">
          <h4 className="text-md font-medium mb-2">Scan to Pay</h4>
          <Image
            src={qrCodeUrl}
            alt="Payment QR Code"
            width={160}
            height={160}
            className="mx-auto"
          />
          {walletAddress && (
            <div className="text-sm text-center mt-2">
              <p>
                <Label>Wallet Address:</Label> {walletAddress}
              </p>
              {tag && (
                <p>
                  <Label>Tag:</Label> {tag}
                </p>
              )}
              <p>Payment ID: {paymentId}</p>
            </div>
          )}
        </div>
      ) : (
        <div className="mt-2 text-sm text-muted-foreground">
          <p>QR code will be displayed here after initiating a payment.</p>
        </div>
      )}
      <div className="flex gap-2">
        <Button variant="outline" onClick={onBack}>Back</Button>
        <Button type="submit" disabled={isLoading} onClick={onSubmit}>
          {isLoading ? 'Processing...' : 'Confirm and Pay'}
        </Button>
      </div>
    </div>
  );
}
