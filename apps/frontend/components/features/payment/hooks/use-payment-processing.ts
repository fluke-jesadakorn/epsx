import { useState } from 'react';
import type { PaymentStep } from '../types';

export function usePaymentProcessing() {
  const [isProcessing, setIsProcessing] = useState(false);
  const [transactionHash, setTransactionHash] = useState<string | null>(null);

  const handlePayment = async (setCurrentStep: (step: PaymentStep) => void) => {
    setIsProcessing(true);

    try {
      await new Promise(resolve => setTimeout(resolve, 2000));
      setCurrentStep('confirmation');
    } catch (_error) {
      alert('Payment failed. Please try again.');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleMetaMaskSuccess = async (txHash: string, setCurrentStep: (step: PaymentStep) => void) => {
    setTransactionHash(txHash);
    setCurrentStep('confirmation');
  };

  const handleMetaMaskError = (_error: string) => {
    // MetaMask error handled by payment system
  };

  return {
    isProcessing,
    transactionHash,
    handlePayment,
    handleMetaMaskSuccess,
    handleMetaMaskError,
  };
}
