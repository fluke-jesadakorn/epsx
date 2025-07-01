'use client';

import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

interface ChoosePaymentMethodProps {
  currency: string;
  network: string;
  isLoading: boolean;
  onNetworkChange: (network: string) => void;
  onBack: () => void;
  onNext: () => void;
}

export function ChoosePaymentMethod({ currency, network, isLoading, onNetworkChange, onBack, onNext }: ChoosePaymentMethodProps) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium">Step 2: Select Payment Method</h3>
      <div className="space-y-2">
        <Label>Payment Method</Label>
        <div className="text-sm text-muted-foreground">
          <p>Currently, only cryptocurrency payments via QR code are supported.</p>
        </div>
        {currency === 'USDT' && (
          <div className="space-y-2">
            <Label htmlFor="network">USDT Network</Label>
            <Select value={network} onValueChange={onNetworkChange}>
              <SelectTrigger id="network">
                <SelectValue placeholder="Select network" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="TRX">TRX (TRC20)</SelectItem>
                <SelectItem value="BNB">BNB (BEP20)</SelectItem>
                <SelectItem value="ETH">ETH (ERC20)</SelectItem>
                <SelectItem value="ARB">ARB (Arbitrum)</SelectItem>
                <SelectItem value="TON">TON</SelectItem>
              </SelectContent>
            </Select>
          </div>
        )}
      </div>
      <div className="flex gap-2">
        <Button variant="outline" onClick={onBack}>Back</Button>
        <Button onClick={onNext} disabled={isLoading}>
          {isLoading ? 'Loading...' : 'Next'}
        </Button>
      </div>
    </div>
  );
}
