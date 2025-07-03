'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { validatePayment } from '@/app/constants/packages';
import type { CurrencyType } from '@/app/constants/packages';

interface PaymentFormProps {
  selectedAsset: string;
  amount: string;
  packageType: string;
  onSubmit: () => void;
}

export default function PaymentForm({
  selectedAsset,
  amount,
  packageType,
  onSubmit,
}: PaymentFormProps) {
  const [localAmount, setLocalAmount] = useState(amount);
  const [validationError, setValidationError] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Validate payment amount
    const error = validatePayment(
      Number(localAmount),
      selectedAsset as CurrencyType
    );

    if (error) {
      switch (error.type) {
        case 'INSUFFICIENT_AMOUNT':
          setValidationError(
            `Minimum amount required: ${error.minAmount} ${error.currency}`
          );
          break;
        case 'INVALID_CURRENCY':
          setValidationError('Selected currency is not supported');
          break;
        default:
          setValidationError('Invalid payment details');
      }
      return;
    }

    setValidationError('');
    onSubmit();
  };

  return (
    <form onSubmit={handleSubmit}>
      <Card>
        <CardContent className="pt-6">
          <div className="space-y-6">
            {/* Package Information */}
            <div>
              <Label>Selected Package</Label>
              <div className="mt-1 p-3 bg-muted rounded-lg">
                <p className="font-medium">{packageType}</p>
              </div>
            </div>

            {/* Asset Information */}
            <div>
              <Label>Payment Asset</Label>
              <div className="mt-1 p-3 bg-muted rounded-lg">
                <p className="font-medium">{selectedAsset}</p>
              </div>
            </div>

            {/* Amount */}
            <div>
              <Label htmlFor="amount">Amount</Label>
              <div className="mt-1">
                <Input
                  id="amount"
                  type="number"
                  step="0.01"
                  min="0"
                  value={localAmount}
                  onChange={(e) => {
                    setLocalAmount(e.target.value);
                    setValidationError('');
                  }}
                  placeholder="Enter amount"
                  className={validationError ? 'border-destructive' : ''}
                />
                {validationError && (
                  <p className="mt-1 text-sm text-destructive">
                    {validationError}
                  </p>
                )}
              </div>
            </div>

            {/* Submit Button */}
            <Button type="submit" className="w-full">
              Proceed to Payment
            </Button>
          </div>
        </CardContent>
      </Card>
    </form>
  );
}
