'use client';
import { useState } from 'react';
import Image from 'next/image';

import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useToast } from '@/components/ui/use-toast';
import { apiClient } from '@/lib/api-client';
import { createPaymentService } from '@/services/payment.service';

export function PaymentForm() {
  const [step, setStep] = useState(1);
  const [amount, setAmount] = useState('');
  const [currency, setCurrency] = useState('USDT');
  const [network, setNetwork] = useState('TRX');
  const [isLoading, setIsLoading] = useState(false);
  const [qrCodeUrl, setQrCodeUrl] = useState<string | undefined>(undefined);
  const [paymentId, setPaymentId] = useState<string | undefined>(undefined);
  const [walletAddress, setWalletAddress] = useState<string | undefined>(
    undefined,
  );
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
      // Fetch payment details based on selected network
      if (currency === 'USDT') {
        setIsLoading(true);
        try {
          // Define wallet addresses and QR codes for different USDT networks
          const paymentNetworks = {
            TRX: {
              name: 'TRC20',
              address: 'TDcbvDd9aYX5cvQgCkLdvu6VbMxadDiC6F',
              qrPath: '/QRPayment/USDT_TRX.png',
            },
            BNB: {
              name: 'BEP20',
              address: '0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59',
              qrPath: '/QRPayment/USDT_BNB.png',
            },
            ETH: {
              name: 'ERC20',
              address: '0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59',
              qrPath: '/QRPayment/USDT_ETH.png',
            },
            ARB: {
              name: 'Arbitrum',
              address: '0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59',
              qrPath: '/QRPayment/USDT_ARB.png',
            },
            TON: {
              name: 'TON',
              address: 'UQDc3azM8KSuxe-Uz_l443CdLzZIIFWrFh9bh5sZ4v9CcgC5',
              tag: 'B0472569C74418F7512A',
              qrPath: '/QRPayment/USDT_TON.png',
            },
          };

          const selectedNetwork =
            paymentNetworks[network as keyof typeof paymentNetworks] ||
            paymentNetworks.TRX;

          setQrCodeUrl(selectedNetwork.qrPath);
          setWalletAddress(selectedNetwork.address);
          setTag('tag' in selectedNetwork ? selectedNetwork.tag : '');
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
      // Move directly to Step 3 (Complete Payment with QR Code)
      setStep(step + 2);
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
        setStep(4); // Move to final step to show QR code and status
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
        description:
          error instanceof Error ? error.message : 'An error occurred',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const renderStep = () => {
    switch (step) {
      case 1:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Step 1: Select Package</h3>
            <div className="space-y-2">
              <Label>Select Package</Label>
              <div className="space-y-6">
                <div className="space-y-2">
                  <h4 className="text-md font-medium text-primary">Personal Plans</h4>
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                    <Button
                      type="button"
                      variant={amount === '9.9' ? 'default' : 'outline'}
                      onClick={() => setAmount('9.9')}
                      className="w-full"
                    >
                      PersonalX 9.9 {currency}
                    </Button>
                    <Button
                      type="button"
                      variant={amount === '19.9' ? 'default' : 'outline'}
                      onClick={() => setAmount('19.9')}
                      className="w-full"
                    >
                      ProfessionalY 19.9 {currency}
                    </Button>
                    <Button
                      type="button"
                      variant={amount === '29.9' ? 'default' : 'outline'}
                      onClick={() => setAmount('29.9')}
                      className="w-full"
                    >
                      EnterpriseZ 29.9 {currency}
                    </Button>
                  </div>
                </div>
                <div className="space-y-2">
                  <h4 className="text-md font-medium text-primary">Business & API Plans</h4>
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                    <Button
                      type="button"
                      variant={amount === '999' ? 'default' : 'outline'}
                      onClick={() => setAmount('999')}
                      className="w-full"
                    >
                      API Personal 999 {currency}/M
                    </Button>
                    <Button
                      type="button"
                      variant={amount === '2999' ? 'default' : 'outline'}
                      onClick={() => setAmount('2999')}
                      className="w-full"
                    >
                      API Company 2,999 {currency}/M
                    </Button>
                    <Button
                      type="button"
                      variant={amount === '999.1' ? 'default' : 'outline'}
                      onClick={() => setAmount('999.1')}
                      className="w-full"
                    >
                      Company 999 {currency}/M
                    </Button>
                  </div>
                </div>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="currency">Currency</Label>
              <Select value={currency} onValueChange={setCurrency} disabled>
                <SelectTrigger id="currency">
                  <SelectValue placeholder="Select currency" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="USDT">USDT</SelectItem>
                </SelectContent>
              </Select>
              <div className="text-sm text-orange-500">
                <p>
                  Currently, only USDT is supported for cryptocurrency payments.
                </p>
              </div>
            </div>
            <Button onClick={handleNext}>Next</Button>
          </div>
        );
      case 2:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">
              Step 2: Select Payment Method
            </h3>
            <div className="space-y-2">
              <Label>Payment Method</Label>
              <div className="text-sm text-muted-foreground">
                <p>
                  Currently, only cryptocurrency payments via QR code are
                  supported.
                </p>
              </div>
              {currency === 'USDT' && (
                <div className="space-y-2">
                  <Label htmlFor="network">USDT Network</Label>
                  <Select value={network} onValueChange={setNetwork}>
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
              <Button variant="outline" onClick={handleBack}>
                Back
              </Button>
              <Button
                onClick={(e) => {
                  e.preventDefault();
                  handleNext();
                }}
                disabled={isLoading}
              >
                {isLoading ? 'Loading...' : 'Next'}
              </Button>
            </div>
          </div>
        );
      case 3:
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
                <p>
                  QR code will be displayed here after initiating a payment.
                </p>
              </div>
            )}
            <div className="flex gap-2">
              <Button variant="outline" onClick={handleBack}>
                Back
              </Button>
              <Button type="submit" disabled={isLoading} onClick={handleSubmit}>
                {isLoading ? 'Processing...' : 'Confirm and Pay'}
              </Button>
            </div>
          </div>
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
