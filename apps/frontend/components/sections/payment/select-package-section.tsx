'use client';

import { useState } from 'react';
import { SelectPackage } from '@/components/features/payment/select-package';
import { usePaymentAuth, PaymentAuthLoadingUI, PaymentAuthRequiredUI, PaymentAccessRequiredUI } from './usePaymentauth';
import { PACKAGES, BLOCKCHAIN_CONFIG } from '@/app/constants/packages';
import { useRouter } from 'next/navigation';

interface SelectPackageSectionProps {
  className?: string;
  showTitle?: boolean;
}

export function SelectPackageSection({
  className = '',
}: SelectPackageSectionProps) {
  const { isLoading, isAuthenticated, hasPaymentAccess, user } = usePaymentAuth();
  const router = useRouter();
  const defaultPackage =
    PACKAGES.find((pkg) => pkg.id === 'silver') ?? PACKAGES[0];
  const [amount, setAmount] = useState(defaultPackage.price.toString());
  const [currency, setCurrency] = useState<string>(
    BLOCKCHAIN_CONFIG.BSC.currency,
  );

  if (isLoading) {return <PaymentAuthLoadingUI />;}
  if (!isAuthenticated) {return <PaymentAuthRequiredUI />;}
  if (!hasPaymentAccess) {return <PaymentAccessRequiredUI user={user} />;}

  const handleCurrencyChange = (newCurrency: string) => {
    setCurrency(newCurrency);
  };

  const getPackageType = (amount: string): string => {
    const pkg = PACKAGES.find((p) => p.price.toString() === amount);
    return pkg?.id ?? 'silver';
  };

  const handleNext = async () => {
    // You can add navigation or payment processing logic here
    if (typeof window !== 'undefined') {
      const packageType = getPackageType(amount);
      router.push(
        `/payment?amount=${amount}&currency=${currency}&packageType=${packageType}`,
      );
    }
  };

  return (
    <section className={`w-full ${className}`}>
      <SelectPackage
        amount={amount}
        currency={currency}
        onAmountChange={setAmount}
        onCurrencyChange={handleCurrencyChange}
        onNext={handleNext}
      />
    </section>
  );
}
