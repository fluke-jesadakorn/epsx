'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { 
  ArrowRight, 
  Copy, 
  Shield, 
  Clock, 
  CheckCircle,
  AlertCircle,
  Loader2
} from 'lucide-react';

interface PaymentDetailsProps {
  selectedPackage: string;
  selectedMethod: string;
  amount: string;
  onBack: () => void;
  onSuccess: () => void;
}

interface PaymentAddress {
  address: string;
  network: string;
  qrCode?: string;
  minConfirmations: number;
}

// Mock payment addresses (in real app, these would come from API)
const PAYMENT_ADDRESSES: Record<string, PaymentAddress> = {
  'USDT_TRC20': {
    address: 'TXYZsample123addresshere456789',
    network: 'TRC20',
    minConfirmations: 1
  },
  'USDT_BSC': {
    address: '0x1234567890abcdef1234567890abcdef12345678',
    network: 'BSC',
    minConfirmations: 3
  },
  'USDT_ERC20': {
    address: '0xabcdef1234567890abcdef1234567890abcdef12',
    network: 'ERC20',
    minConfirmations: 6
  }
};

export default function PaymentDetails({ 
  selectedPackage, 
  selectedMethod, 
  amount,
  onBack,
  onSuccess
}: PaymentDetailsProps) {
  const router = useRouter();
  const [copied, setCopied] = useState(false);
  const [txHash, setTxHash] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [paymentStatus, setPaymentStatus] = useState<'pending' | 'confirmed' | 'failed'>('pending');

  const paymentAddress = PAYMENT_ADDRESSES[selectedMethod];
  const isCardPayment = selectedMethod === 'credit_card';

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy: ', err);
    }
  };

  const handleSubmitTxHash = async () => {
    if (!txHash.trim()) return;
    
    setIsSubmitting(true);
    
    // Simulate transaction verification
    setTimeout(() => {
      setPaymentStatus('confirmed');
      setIsSubmitting(false);
      setTimeout(() => onSuccess(), 1500);
    }, 2000);
  };

  const handleCardPayment = () => {
    // Redirect to card payment processor
    router.push(`/payment/card?package=${selectedPackage}&amount=${amount}`);
  };

  if (isCardPayment) {
    return (
      <Card className="max-w-lg mx-auto border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="text-gray-900 dark:text-white">Credit Card Payment</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="text-center">
            <div className="text-3xl font-bold text-primary mb-2">
              ${amount}
            </div>
            <p className="text-muted-foreground">
              You will be redirected to our secure payment processor
            </p>
          </div>
          
          <div className="flex flex-col sm:flex-row gap-2">
            <Button variant="outline" onClick={onBack} className="flex-1 border-gray-200 dark:border-gray-600">
              Back
            </Button>
            <Button onClick={handleCardPayment} className="flex-1 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700">
              Pay with Card
              <ArrowRight className="ml-2 h-4 w-4" />
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Payment Instructions */}
      <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
            <Shield className="h-5 w-5 text-blue-500" />
            Send Payment
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="bg-blue-50 dark:bg-blue-900/20 p-4 rounded-lg border border-blue-200 dark:border-blue-800">
            <div className="flex items-start gap-3">
              <AlertCircle className="h-5 w-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" />
              <div>
                <h4 className="font-semibold text-blue-900 dark:text-blue-100">Payment Instructions</h4>
                <p className="text-sm text-blue-800 dark:text-blue-200 mt-1">
                  Send exactly <strong>${amount} USDT</strong> to the address below using the {paymentAddress.network} network.
                </p>
              </div>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <Label className="text-gray-700 dark:text-gray-300">Payment Address</Label>
              <div className="flex flex-col sm:flex-row items-stretch gap-2 mt-1">
                <Input
                  value={paymentAddress.address}
                  readOnly
                  className="font-mono text-xs sm:text-sm bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600 flex-1"
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => copyToClipboard(paymentAddress.address)}
                  className="border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 min-w-[80px]"
                >
                  {copied ? (
                    <CheckCircle className="h-4 w-4 text-green-600" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <Label className="text-gray-700 dark:text-gray-300">Network</Label>
                <div className="mt-1">
                  <Badge variant="outline" className="bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100">
                    {paymentAddress.network}
                  </Badge>
                </div>
              </div>
              <div>
                <Label className="text-gray-700 dark:text-gray-300">Amount</Label>
                <div className="mt-1 font-semibold text-primary text-lg">${amount} USDT</div>
              </div>
            </div>
          </div>

          <Alert className="border-yellow-200 dark:border-yellow-800 bg-yellow-50 dark:bg-yellow-900/20">
            <AlertCircle className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
            <AlertDescription className="text-yellow-800 dark:text-yellow-200">
              <strong>Important:</strong> Make sure to use the {paymentAddress.network} network. 
              Using the wrong network will result in loss of funds.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Transaction Hash Submission */}
      <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="text-gray-900 dark:text-white">After Payment</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Once you&apos;ve sent the payment, paste your transaction hash below to speed up verification.
          </p>

          <div className="space-y-3">
            <div>
              <Label htmlFor="txHash" className="text-gray-700 dark:text-gray-300">Transaction Hash (Optional)</Label>
              <Input
                id="txHash"
                placeholder="0x..."
                value={txHash}
                onChange={(e) => setTxHash(e.target.value)}
                className="font-mono text-sm bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600 mt-1"
              />
            </div>

            {paymentStatus === 'confirmed' && (
              <Alert className="border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20">
                <CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400" />
                <AlertDescription className="text-green-800 dark:text-green-200">
                  Payment confirmed! Redirecting to dashboard...
                </AlertDescription>
              </Alert>
            )}

            <div className="flex flex-col sm:flex-row gap-2">
              <Button variant="outline" onClick={onBack} className="flex-1 border-gray-200 dark:border-gray-600">
                Back
              </Button>
              <Button
                onClick={handleSubmitTxHash}
                disabled={isSubmitting || paymentStatus === 'confirmed'}
                className="flex-1 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700"
              >
                {isSubmitting ? (
                  <div className="flex items-center gap-2">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Verifying...
                  </div>
                ) : paymentStatus === 'confirmed' ? (
                  <div className="flex items-center gap-2">
                    <CheckCircle className="h-4 w-4" />
                    Confirmed
                  </div>
                ) : (
                  'Submit Transaction'
                )}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Status Monitor */}
      <Card className="border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-gray-900 dark:text-white">
            <Clock className="h-5 w-5 text-orange-500" />
            Payment Status
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
              <span className="text-sm text-gray-600 dark:text-gray-300">Waiting for payment...</span>
            </div>
            <div className="text-xs text-muted-foreground">
              Payments are typically confirmed within 1-10 minutes depending on network congestion.
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
