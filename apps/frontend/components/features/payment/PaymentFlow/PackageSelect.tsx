'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Card } from '@/components/ui/card';
import type { Package, PaymentMethod } from './types';

interface PackageSelectProps {
  onSelect: (pkg: Package, method: PaymentMethod) => void;
}

const PACKAGES: Package[] = [
  {
    id: 'basic',
    name: 'Basic Package',
    price: 29,
    currency: 'USD',
    description: 'Essential features for beginners',
    features: ['Basic Analysis', 'Market Updates', 'Limited Reports'],
  },
  {
    id: 'pro',
    name: 'Pro Package',
    price: 99,
    currency: 'USD',
    description: 'Advanced features for serious traders',
    features: ['Advanced Analysis', 'Real-time Updates', 'Unlimited Reports', 'Priority Support'],
  },
];

export default function PackageSelect({ onSelect }: PackageSelectProps) {
  const [selectedPackage, setSelectedPackage] = useState<Package | null>(null);
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>('on_line');

  const handleSubmit = () => {
    if (selectedPackage) {
      onSelect(selectedPackage, paymentMethod);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-4">Select Package</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {PACKAGES.map((pkg) => (
            <Card
              key={pkg.id}
              className={`p-4 cursor-pointer transition-colors ${
                selectedPackage?.id === pkg.id
                  ? 'border-primary bg-primary/5'
                  : 'hover:border-primary/50'
              }`}
              onClick={() => setSelectedPackage(pkg)}
            >
              <h3 className="text-lg font-semibold">{pkg.name}</h3>
              <p className="text-2xl font-bold mt-2">
                ${pkg.price}
                <span className="text-sm font-normal text-muted-foreground">
                  /{pkg.currency}
                </span>
              </p>
              <p className="text-sm text-muted-foreground mt-2">{pkg.description}</p>
              <ul className="mt-4 space-y-2">
                {pkg.features.map((feature, index) => (
                  <li key={index} className="text-sm flex items-center">
                    <svg
                      className="w-4 h-4 mr-2 text-green-500"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    {feature}
                  </li>
                ))}
              </ul>
            </Card>
          ))}
        </div>
      </div>

      <div className="space-y-4">
        <Label className="text-lg font-semibold">Payment Method</Label>
        <RadioGroup
          defaultValue={paymentMethod}
          onValueChange={(value) => setPaymentMethod(value as PaymentMethod)}
          className="grid grid-cols-2 gap-4"
        >
          <div className={`border rounded-lg p-4 ${
            paymentMethod === 'on_line' ? 'border-primary' : ''
          }`}>
            <RadioGroupItem value="on_line" id="on_line" className="sr-only" />
            <Label htmlFor="on_line" className="font-medium">Online Payment</Label>
            <p className="text-sm text-muted-foreground mt-1">
              Pay with credit card or bank transfer
            </p>
          </div>
          <div className={`border rounded-lg p-4 ${
            paymentMethod === 'on_chain' ? 'border-primary' : ''
          }`}>
            <RadioGroupItem value="on_chain" id="on_chain" className="sr-only" />
            <Label htmlFor="on_chain" className="font-medium">Cryptocurrency</Label>
            <p className="text-sm text-muted-foreground mt-1">
              Pay with USDT or other cryptocurrencies
            </p>
          </div>
        </RadioGroup>
      </div>

      <Button
        onClick={handleSubmit}
        disabled={!selectedPackage}
        className="w-full"
        size="lg"
      >
        Continue to Payment
      </Button>
    </div>
  );
}
