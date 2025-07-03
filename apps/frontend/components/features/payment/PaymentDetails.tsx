'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { 
  ArrowRight, 
  Copy, 
  Shield, 
  Clock, 
  CheckCircle,
  AlertCircle,
  Loader2,
  CreditCard
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
      <Card className="max-w-lg mx-auto border-0 shadow-2xl bg-gradient-to-br from-white via-purple-50/50 to-pink-50/50 dark:from-gray-800 dark:via-purple-900/20 dark:to-pink-900/20 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-purple-400/5 via-pink-400/5 to-orange-400/5"></div>
        <CardHeader className="relative z-10">
          <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
            <div className="w-8 h-8 bg-gradient-to-br from-purple-500 to-pink-500 rounded-full flex items-center justify-center">
              <CreditCard className="h-5 w-5 text-white" />
            </div>
            Credit Card Payment
            <div className="ml-auto text-2xl">💳</div>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-6 relative z-10">
          <div className="text-center p-6 bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/30 dark:to-pink-900/30 rounded-2xl border-2 border-purple-200 dark:border-purple-700">
            <div className="text-4xl font-black bg-gradient-to-r from-purple-600 via-pink-600 to-orange-600 bg-clip-text text-transparent mb-3">
              ${amount}
            </div>
            <p className="text-gray-600 dark:text-gray-300 font-medium">
              You will be redirected to our secure payment processor
            </p>
          </div>
          
          <div className="flex flex-col sm:flex-row gap-3">
            <Button 
              variant="outline" 
              onClick={onBack} 
              className="flex-1 h-12 border-2 border-gray-300 dark:border-gray-600 hover:border-purple-400 dark:hover:border-purple-500 font-semibold"
            >
              ← Back
            </Button>
            <Button 
              onClick={handleCardPayment} 
              className="flex-1 h-12 bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 hover:from-purple-600 hover:via-pink-600 hover:to-orange-600 font-bold text-white border-0 shadow-xl transition-all duration-300 hover:scale-[1.02]"
            >
              <div className="flex items-center gap-2">
                <span>Pay with Card</span>
                <ArrowRight className="h-4 w-4" />
                <span>🚀</span>
              </div>
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Payment Instructions */}
      <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-blue-50/50 to-cyan-50/50 dark:from-gray-800 dark:via-blue-900/20 dark:to-cyan-900/20 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-400/5 via-cyan-400/5 to-teal-400/5"></div>
        <CardHeader className="relative z-10">
          <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
            <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-full flex items-center justify-center">
              <Shield className="h-5 w-5 text-white" />
            </div>
            Send Payment
            <div className="ml-auto text-2xl animate-bounce">💰</div>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-6 relative z-10">
          <div className="bg-gradient-to-r from-blue-50 to-cyan-50 dark:from-blue-900/30 dark:to-cyan-900/30 p-6 rounded-2xl border-2 border-blue-200 dark:border-blue-700 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-r from-blue-400/10 to-cyan-400/10"></div>
            <div className="relative z-10">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-xl flex items-center justify-center shadow-lg">
                  <AlertCircle className="h-6 w-6 text-white" />
                </div>
                <div className="flex-1">
                  <h4 className="font-bold text-blue-900 dark:text-blue-100 text-lg mb-2">Payment Instructions</h4>
                  <p className="text-blue-800 dark:text-blue-200 font-medium">
                    Send exactly <span className="font-black text-2xl bg-gradient-to-r from-blue-600 to-cyan-600 bg-clip-text text-transparent">${amount} USDT</span> to the address below using the <span className="font-bold text-blue-600 dark:text-blue-400">{paymentAddress.network}</span> network.
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div className="space-y-6">
            <div className="bg-white/60 dark:bg-gray-700/60 p-4 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
              <Label className="text-gray-700 dark:text-gray-300 font-bold text-lg mb-3 block">Payment Address</Label>
              <div className="flex flex-col sm:flex-row items-stretch gap-3">
                <Input
                  value={paymentAddress.address}
                  readOnly
                  className="font-mono text-sm bg-gray-50 dark:bg-gray-700 border-2 border-gray-200 dark:border-gray-600 flex-1 h-12 rounded-xl"
                />
                <Button
                  variant="outline"
                  onClick={() => copyToClipboard(paymentAddress.address)}
                  className="h-12 px-6 border-2 border-gray-200 dark:border-gray-600 hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20 font-semibold rounded-xl"
                >
                  {copied ? (
                    <div className="flex items-center gap-2">
                      <CheckCircle className="h-4 w-4 text-green-600" />
                      <span className="text-green-600">Copied!</span>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2">
                      <Copy className="h-4 w-4" />
                      <span>Copy</span>
                    </div>
                  )}
                </Button>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="bg-white/60 dark:bg-gray-700/60 p-4 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
                <Label className="text-gray-700 dark:text-gray-300 font-bold">Network</Label>
                <div className="mt-2">
                  <Badge className="bg-gradient-to-r from-blue-500 to-cyan-500 text-white border-0 shadow-lg text-lg px-4 py-2">
                    {paymentAddress.network}
                  </Badge>
                </div>
              </div>
              <div className="bg-white/60 dark:bg-gray-700/60 p-4 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
                <Label className="text-gray-700 dark:text-gray-300 font-bold">Amount</Label>
                <div className="mt-2 font-black text-2xl bg-gradient-to-r from-green-600 to-emerald-600 bg-clip-text text-transparent">
                  ${amount} USDT
                </div>
              </div>
            </div>
          </div>

          <Alert className="border-0 bg-gradient-to-r from-yellow-50 to-orange-50 dark:from-yellow-900/20 dark:to-orange-900/20 p-6 rounded-2xl shadow-lg relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-r from-yellow-400/10 to-orange-400/10"></div>
            <div className="relative z-10">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 bg-gradient-to-br from-yellow-500 to-orange-500 rounded-xl flex items-center justify-center shadow-lg">
                  <AlertCircle className="h-6 w-6 text-white" />
                </div>
                <div>
                  <h4 className="font-bold text-yellow-900 dark:text-yellow-100 text-lg mb-2">⚠️ Important Notice</h4>
                  <p className="text-yellow-800 dark:text-yellow-200 font-medium">
                    Make sure to use the <span className="font-bold text-yellow-600 dark:text-yellow-400">{paymentAddress.network}</span> network. 
                    Using the wrong network will result in <span className="font-bold text-red-600 dark:text-red-400">permanent loss of funds</span>.
                  </p>
                </div>
              </div>
            </div>
          </Alert>
        </CardContent>
      </Card>

      {/* Transaction Hash Submission */}
      <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-green-50/50 to-emerald-50/50 dark:from-gray-800 dark:via-green-900/20 dark:to-emerald-900/20 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-green-400/5 via-emerald-400/5 to-teal-400/5"></div>
        <CardHeader className="relative z-10">
          <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
            <div className="w-8 h-8 bg-gradient-to-br from-green-500 to-emerald-500 rounded-full flex items-center justify-center">
              <CheckCircle className="h-5 w-5 text-white" />
            </div>
            After Payment
            <div className="ml-auto text-2xl animate-bounce">📝</div>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-6 relative z-10">
          <div className="bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/30 dark:to-emerald-900/30 p-6 rounded-2xl border-2 border-green-200 dark:border-green-700">
            <p className="text-green-800 dark:text-green-200 font-medium text-lg">
              Once you&apos;ve sent the payment, paste your transaction hash below to speed up verification. 🚀
            </p>
          </div>

          <div className="space-y-6">
            <div className="bg-white/60 dark:bg-gray-700/60 p-4 rounded-xl border border-gray-200/50 dark:border-gray-600/50">
              <Label htmlFor="txHash" className="text-gray-700 dark:text-gray-300 font-bold text-lg mb-3 block">
                Transaction Hash (Optional)
              </Label>
              <Input
                id="txHash"
                placeholder="0x... or paste your transaction hash here"
                value={txHash}
                onChange={(e) => setTxHash(e.target.value)}
                className="font-mono text-sm bg-gray-50 dark:bg-gray-700 border-2 border-gray-200 dark:border-gray-600 h-12 rounded-xl"
              />
            </div>

            {paymentStatus === 'confirmed' && (
              <Alert className="border-0 bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/30 dark:to-emerald-900/30 p-6 rounded-2xl shadow-lg relative overflow-hidden">
                <div className="absolute inset-0 bg-gradient-to-r from-green-400/10 to-emerald-400/10"></div>
                <div className="relative z-10">
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 bg-gradient-to-br from-green-500 to-emerald-500 rounded-xl flex items-center justify-center shadow-lg animate-pulse">
                      <CheckCircle className="h-6 w-6 text-white" />
                    </div>
                    <div>
                      <h4 className="font-bold text-green-900 dark:text-green-100 text-lg">🎉 Payment Confirmed!</h4>
                      <p className="text-green-800 dark:text-green-200 font-medium">Redirecting to dashboard...</p>
                    </div>
                  </div>
                </div>
              </Alert>
            )}

            <div className="flex flex-col sm:flex-row gap-3">
              <Button 
                variant="outline" 
                onClick={onBack} 
                className="flex-1 h-12 border-2 border-gray-300 dark:border-gray-600 hover:border-green-400 dark:hover:border-green-500 font-semibold rounded-xl"
              >
                ← Back
              </Button>
              <Button
                onClick={handleSubmitTxHash}
                disabled={isSubmitting || paymentStatus === 'confirmed'}
                className="flex-1 h-12 bg-gradient-to-r from-green-500 via-emerald-500 to-teal-500 hover:from-green-600 hover:via-emerald-600 hover:to-teal-600 font-bold text-white border-0 shadow-xl transition-all duration-300 hover:scale-[1.02] disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none rounded-xl"
              >
                {isSubmitting ? (
                  <div className="flex items-center gap-2">
                    <Loader2 className="h-5 w-5 animate-spin" />
                    <span>Verifying...</span>
                  </div>
                ) : paymentStatus === 'confirmed' ? (
                  <div className="flex items-center gap-2">
                    <CheckCircle className="h-5 w-5" />
                    <span>Confirmed</span>
                    <span>✅</span>
                  </div>
                ) : (
                  <div className="flex items-center gap-2">
                    <span>Submit Transaction</span>
                    <span>🚀</span>
                  </div>
                )}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Status Monitor */}
      <Card className="border-0 shadow-2xl bg-gradient-to-br from-white via-orange-50/50 to-yellow-50/50 dark:from-gray-800 dark:via-orange-900/20 dark:to-yellow-900/20 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-orange-400/5 via-yellow-400/5 to-amber-400/5"></div>
        <CardHeader className="relative z-10">
          <CardTitle className="flex items-center gap-3 text-gray-900 dark:text-white text-2xl font-bold">
            <div className="w-8 h-8 bg-gradient-to-br from-orange-500 to-yellow-500 rounded-full flex items-center justify-center">
              <Clock className="h-5 w-5 text-white" />
            </div>
            Payment Status
            <div className="ml-auto text-2xl animate-bounce">⏰</div>
          </CardTitle>
        </CardHeader>
        <CardContent className="relative z-10">
          <div className="bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-900/30 dark:to-yellow-900/30 p-6 rounded-2xl border-2 border-orange-200 dark:border-orange-700">
            <div className="space-y-4">
              <div className="flex items-center gap-4">
                <div className="w-6 h-6 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-white rounded-full animate-pulse"></div>
                </div>
                <span className="text-orange-800 dark:text-orange-200 font-bold text-lg">
                  Waiting for payment... 🔍
                </span>
              </div>
              <div className="text-orange-700 dark:text-orange-300 font-medium">
                💡 Payments are typically confirmed within <span className="font-bold text-orange-600 dark:text-orange-400">1-10 minutes</span> depending on network congestion.
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
