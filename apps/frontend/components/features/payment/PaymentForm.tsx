'use client';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { createPaymentService } from '@/services/payment.service';
import { useToast } from '@/components/ui/use-toast';
import { apiClient } from '@/lib/api-client';

export function PaymentForm() {
  const [step, setStep] = useState(1);
  const [amount, setAmount] = useState('');
  const [currency, setCurrency] = useState('USDT');
  const [isLoading, setIsLoading] = useState(false);
  const [qrCodeUrl, setQrCodeUrl] = useState<string | undefined>(undefined);
  const [paymentId, setPaymentId] = useState<string | undefined>(undefined);
  const { toast } = useToast();

  const apiUrl = '';
  console.log(
    'API URL being used in apiClient:',
    process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002',
  );
  const paymentService = createPaymentService(
    {
      apiUrl: apiUrl,
      endpoints: {
        createPayment: '/create',
        validatePayment: '/validate',
        getPayment: '',
        getQrCode: '/qrcode',
      },
    },
    apiClient,
  );

  const handleNext = () => {
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
      // Ensure progression to Step 3 (Confirm Payment Details)
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

      const response = await paymentService.createPayment(
        numericAmount,
        currency,
      );

      if (response?.id) {
        setPaymentId(response.id);
        try {
          const qrCode = await paymentService.getQrCode(response.id);
          if (qrCode) {
            setQrCodeUrl(qrCode);
          }
        } catch (error) {
          console.error('Failed to fetch QR code:', error);
          toast({
            title: 'QR Code Fetch Failed',
            description:
              'Proceeding without QR code display. You can still complete the payment manually.',
          });
        }
        setStep(4); // Move to final step to show QR code and status
        toast({
          title: 'Payment Initiated',
          description: `Payment ID: ${response?.id}`,
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
              <Label>Select Package Amount</Label>
              <div className="grid grid-cols-3 gap-2">
                <Button
                  type="button"
                  variant={amount === '199' ? 'default' : 'outline'}
                  onClick={() => setAmount('199')}
                  className="w-full"
                >
                  199 {currency}
                </Button>
                <Button
                  type="button"
                  variant={amount === '299' ? 'default' : 'outline'}
                  onClick={() => setAmount('299')}
                  className="w-full"
                >
                  299 {currency}
                </Button>
                <Button
                  type="button"
                  variant={amount === '399' ? 'default' : 'outline'}
                  onClick={() => setAmount('399')}
                  className="w-full"
                >
                  399 {currency}
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="currency">Currency</Label>
              <Select value={currency} onValueChange={setCurrency}>
                <SelectTrigger id="currency">
                  <SelectValue placeholder="Select currency" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="USDT">USDT</SelectItem>
                  <SelectItem value="BTC">BTC</SelectItem>
                  <SelectItem value="ETH">ETH</SelectItem>
                </SelectContent>
              </Select>
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
              >
                Next
              </Button>
            </div>
          </div>
        );
      case 3:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">
              Step 3: Confirm Payment Details
            </h3>
            <div className="space-y-2">
              <div>
                <Label>Amount:</Label> {amount} {currency}
              </div>
              <div>
                <Label>Payment Method:</Label> Cryptocurrency (QR Code)
              </div>
            </div>
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
      case 4:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Step 4: Complete Payment</h3>
            {qrCodeUrl ? (
              <div className="mt-2">
                <h4 className="text-md font-medium mb-2">Scan to Pay</h4>
                <img
                  src={qrCodeUrl}
                  alt="Payment QR Code"
                  className="w-40 h-40 mx-auto"
                />
                <p className="text-sm text-center mt-2">
                  Payment ID: {paymentId}
                </p>
              </div>
            ) : (
              <div className="mt-2 text-sm text-muted-foreground">
                <p>
                  QR code will be displayed here after initiating a payment.
                </p>
              </div>
            )}
            <Button variant="outline" onClick={() => setStep(1)}>
              Make Another Payment
            </Button>
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
