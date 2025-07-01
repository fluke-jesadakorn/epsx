'use client';

import { useState } from 'react';
import { useToast } from '@/components/ui/use-toast';
import { createPaymentService } from '@/services/payment.service';
import { auth } from '@/lib/firebase';
import { SelectPackage } from './SelectPackage';
import { ChoosePaymentMethod } from './ChoosePaymentMethod';
import { CompletePayment } from './CompletePayment';

export function PaymentForm() {
  const [step, setStep] = useState(1);
  const [amount, setAmount] = useState('');
  const [currency, setCurrency] = useState('USDT');
  const [network, setNetwork] = useState('TRX');
  const [isLoading, setIsLoading] = useState(false);
  const [qrCodeUrl, setQrCodeUrl] = useState<string | undefined>(undefined);
  const [paymentId, setPaymentId] = useState<string | undefined>(undefined);
  const [walletAddress, setWalletAddress] = useState<string | undefined>(undefined);
  const [tag, setTag] = useState<string | undefined>(undefined);
  const { toast } = useToast();

  const paymentService = createPaymentService();

  const handleNext = async () => {
    if (step === 1) {
      if (!amount) {
        toast({
          title: 'Invalid Input',
          description: 'Please select a package.',
        });
        return;
      }
      setStep(step + 1);
    } else if (step === 2) {
      if (currency === 'USDT') {
        setIsLoading(true);
        try {
          const user = auth.currentUser;
          if (!user) {
            throw new Error('User not authenticated');
          }
          const paymentDetails = await paymentService.getPaymentDetails(user.uid, network);
          if (!paymentDetails) {
            throw new Error('Failed to fetch payment details');
          }
          setQrCodeUrl(paymentDetails.qrCodePath);
          setWalletAddress(paymentDetails.walletAddress);
          setTag(paymentDetails.tag);
        } catch (error) {
          console.error('Error setting payment details:', error);
          toast({
            title: 'Error',
            description: 'Failed to set payment details. Please try again.',
          });
          return;
        } finally {
          setIsLoading(false);
        }
      }
      setStep(step + 1);
    }
  };

  const handleBack = () => {
    setStep(step - 1);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const numericAmount = parseFloat(amount);
      if (isNaN(numericAmount) || numericAmount <= 0) {
        throw new Error('Invalid amount');
      }

      const response = await paymentService.recordPayment(
        numericAmount,
        currency,
        `Subscription payment via ${network}`,
      );

      if (response) {
        setPaymentId(response);
        setStep(4);
        toast({
          title: 'Payment Initiated',
          description: `Payment ID: ${response}`,
        });
      } else {
        throw new Error('Payment initiation failed');
      }
    } catch (error) {
      toast({
        title: 'Payment Failed',
        description: error instanceof Error ? error.message : 'An error occurred',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const renderStep = () => {
    switch (step) {
      case 1:
        return (
          <SelectPackage
            amount={amount}
            currency={currency}
            onAmountChange={setAmount}
            onCurrencyChange={setCurrency}
            onNext={handleNext}
          />
        );
      case 2:
        return (
          <ChoosePaymentMethod
            currency={currency}
            network={network}
            isLoading={isLoading}
            onNetworkChange={setNetwork}
            onBack={handleBack}
            onNext={handleNext}
          />
        );
      case 3:
        return (
          <CompletePayment
            qrCodeUrl={qrCodeUrl}
            walletAddress={walletAddress}
            tag={tag}
            paymentId={paymentId}
            isLoading={isLoading}
            onBack={handleBack}
            onSubmit={handleSubmit}
          />
        );
      default:
        return null;
    }
  };

  return (
    <div>
      <form className="space-y-4">{renderStep()}</form>
    </div>
  );
}
