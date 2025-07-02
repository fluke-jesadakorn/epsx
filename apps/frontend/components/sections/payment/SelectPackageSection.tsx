'use client';

import { useState } from 'react';
import { SelectPackage } from '@/components/features/payment/SelectPackage';
import { withPaymentAuth } from './withPaymentAuth';

interface SelectPackageSectionProps {
  className?: string;
  showTitle?: boolean;
}

const BaseSelectPackageSection = ({ 
  className = '',
}: SelectPackageSectionProps) => {
  const [amount, setAmount] = useState('9.9'); // Default to PersonalX plan
  const [currency, setCurrency] = useState('USDT_TRC20');

  const getPackageType = (amount: string): string => {
    switch (amount) {
      case '9.9':
        return 'personalx';
      case '19.9':
        return 'professionaly';
      case '29.9':
        return 'enterprisez';
      case '999':
        return 'apipersonal';
      case '2999':
        return 'apicompany';
      case '999.1':
        return 'company';
      default:
        return 'personalx';
    }
  };

  const handleNext = async () => {
    // You can add navigation or payment processing logic here
    if (typeof window !== 'undefined') {
      const packageType = getPackageType(amount);
      window.location.href = `/payment?amount=${amount}&currency=${currency}&packageType=${packageType}`;
    }
  };

  return (
    <section className={`w-full ${className}`}>
      <SelectPackage
        amount={amount}
        currency={currency}
        onAmountChange={setAmount}
        onCurrencyChange={setCurrency}
        onNext={handleNext}
      />
    </section>
  );
};

export const SelectPackageSection = withPaymentAuth(BaseSelectPackageSection);
