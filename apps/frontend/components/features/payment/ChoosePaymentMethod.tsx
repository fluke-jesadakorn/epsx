'use client';

import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface ChoosePaymentMethodProps {
  currency: string;
  network: string;
  isLoading: boolean;
  onNetworkChange: (network: string) => void;
  onBack: () => void;
  onNext: () => void;
}

export function ChoosePaymentMethod({
  currency,
  network,
  isLoading,
  onNetworkChange,
  onBack,
  onNext,
}: ChoosePaymentMethodProps) {
  return (
    <div className="space-y-8 p-6 rounded-xl border bg-card shadow-sm">
      <div className="space-y-2">
        <h3 className="text-2xl font-bold text-primary">
          Step 2: Select Payment Method
        </h3>
        <p className="text-muted-foreground">
          Choose your preferred cryptocurrency payment network
        </p>
      </div>

      <div className="space-y-6">
        <div className="p-6 bg-muted/50 rounded-xl border space-y-4">
          <div className="flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="12" cy="12" r="8" />
              <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
              <path d="M12 17h.01" />
            </svg>
            <Label className="font-semibold">Available Payment Methods</Label>
          </div>
          <div className="flex items-start gap-2 text-sm text-muted-foreground">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" />
              <path d="m9 12 2 2 4-4" />
            </svg>
            <p>Cryptocurrency payments via QR code</p>
          </div>
        </div>

        {currency === 'USDT' && (
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M20 12V8H6a2 2 0 0 1-2-2c0-1.1.9-2 2-2h12v4" />
                <path d="M4 6v12c0 1.1.9 2 2 2h14v-4" />
                <path d="M18 12a2 2 0 0 0-2 2c0 1.1.9 2 2 2h4v-4h-4z" />
              </svg>
              <Label htmlFor="network" className="font-semibold">
                USDT Network
              </Label>
            </div>
            <Select value={network} onValueChange={onNetworkChange}>
              <SelectTrigger id="network" className="bg-background border-2">
                <SelectValue placeholder="Select network" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="TRX">
                  <div className="flex justify-between items-center">
                    <span>TRX (TRC20)</span>
                    <span className="text-xs bg-green-500/10 text-green-600 px-2 py-0.5 rounded-full">
                      Recommended
                    </span>
                  </div>
                </SelectItem>
                <SelectItem value="BNB">BNB (BEP20)</SelectItem>
                <SelectItem value="ETH">ETH (ERC20)</SelectItem>
                <SelectItem value="ARB">ARB (Arbitrum)</SelectItem>
                <SelectItem value="TON">TON</SelectItem>
              </SelectContent>
            </Select>
            <div className="flex items-start gap-2 text-sm text-orange-500">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
                <path d="M12 9v4" />
                <path d="M12 17h.01" />
              </svg>
              <p>TRC20 network is recommended for lower transaction fees</p>
            </div>
          </div>
        )}
      </div>

      <div className="flex gap-3 pt-4">
        <Button
          variant="outline"
          onClick={onBack}
          className="flex-1 py-6 text-lg font-medium"
        >
          Back
        </Button>
        <Button
          onClick={onNext}
          disabled={isLoading}
          className="flex-1 py-6 text-lg font-medium bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg hover:shadow-xl transition-all"
        >
          {isLoading ? (
            <div className="flex items-center gap-2">
              <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                  fill="none"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              Loading...
            </div>
          ) : (
            'Continue to Payment'
          )}
        </Button>
      </div>
    </div>
  );
}
