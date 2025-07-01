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
    <div className="space-y-8 p-6 rounded-xl border bg-card shadow-sm">
      <div className="space-y-2">
        <h3 className="text-2xl font-bold text-primary">Step 3: Complete Payment</h3>
        <p className="text-muted-foreground">Scan the QR code or use the wallet details to complete your payment</p>
      </div>

      {qrCodeUrl ? (
        <div className="space-y-6">
          <div className="flex flex-col items-center space-y-4">
            <div className="p-4 bg-white rounded-xl shadow-sm">
              <Image
                src={qrCodeUrl}
                alt="Payment QR Code"
                width={200}
                height={200}
                className="mx-auto"
              />
            </div>
            <h4 className="text-lg font-semibold text-primary">Scan QR Code to Pay</h4>
          </div>

          {walletAddress && (
            <div className="space-y-4 p-6 bg-muted/50 rounded-xl border">
              <div className="space-y-3">
                <div className="flex flex-col gap-1.5">
                  <Label className="text-sm font-medium text-muted-foreground">Wallet Address</Label>
                  <p className="font-mono text-sm bg-background p-2.5 rounded border select-all">{walletAddress}</p>
                </div>
                {tag && (
                  <div className="flex flex-col gap-1.5">
                    <Label className="text-sm font-medium text-muted-foreground">Tag</Label>
                    <p className="font-mono text-sm bg-background p-2.5 rounded border select-all">{tag}</p>
                  </div>
                )}
                <div className="flex flex-col gap-1.5">
                  <Label className="text-sm font-medium text-muted-foreground">Payment ID</Label>
                  <p className="font-mono text-sm bg-background p-2.5 rounded border select-all">{paymentId}</p>
                </div>
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="flex items-center justify-center h-40 bg-muted/50 rounded-xl border">
          <div className="text-center space-y-2">
            <svg 
              xmlns="http://www.w3.org/2000/svg" 
              width="24" 
              height="24" 
              viewBox="0 0 24 24" 
              fill="none" 
              stroke="currentColor" 
              strokeWidth="2" 
              strokeLinecap="round" 
              strokeLinejoin="round" 
              className="mx-auto text-muted-foreground mb-2"
            >
              <rect width="18" height="18" x="3" y="3" rx="2" />
              <path d="M8 12h8" />
              <path d="M12 8v8" />
            </svg>
            <p className="text-sm text-muted-foreground">QR code will be displayed here after initiating a payment</p>
          </div>
        </div>
      )}

      <div className="flex gap-3 pt-4">
        <Button 
          variant="outline" 
          onClick={onBack}
          className="flex-1 py-6 text-lg font-medium"
        >
          Back
        </Button>
        <Button 
          type="submit" 
          disabled={isLoading} 
          onClick={onSubmit}
          className="flex-1 py-6 text-lg font-medium bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg hover:shadow-xl transition-all"
        >
          {isLoading ? (
            <div className="flex items-center gap-2">
              <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              Processing...
            </div>
          ) : (
            'Confirm and Pay'
          )}
        </Button>
      </div>
    </div>
  );
}
