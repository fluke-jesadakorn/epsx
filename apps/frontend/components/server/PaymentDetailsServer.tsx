import { 
  Shield, 
  Clock, 
  CreditCard,
  Wallet
} from 'lucide-react';
import { getAssetInfo } from '@/app/actions/payment-server';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@/components/ui';

interface PaymentDetailsServerProps {
  selectedPackage: string;
  selectedMethod: string;
  amount: string;
}

export default async function PaymentDetailsServer({
  selectedPackage,
  selectedMethod,
  amount
}: PaymentDetailsServerProps) {
  // Server-side data fetching
  const assetInfo = await getAssetInfo(selectedMethod);
  
  return (
    <Card className="w-full max-w-2xl mx-auto">
      <CardHeader className="text-center">
        <CardTitle className="flex items-center justify-center gap-2 text-2xl font-bold">
          <CreditCard className="h-6 w-6" />
          Payment Details
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Package Information */}
        <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 p-4 rounded-lg">
          <h3 className="font-semibold text-lg mb-2">Selected Package</h3>
          <div className="flex items-center justify-between">
            <span className="text-xl font-bold">{selectedPackage}</span>
            <Badge variant="secondary" className="text-lg px-3 py-1">
              ${amount}
            </Badge>
          </div>
        </div>

        {/* Payment Method Information */}
        <div className="bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 p-4 rounded-lg">
          <h3 className="font-semibold text-lg mb-2 flex items-center gap-2">
            <Wallet className="h-5 w-5" />
            Payment Method
          </h3>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span>Currency:</span>
              <Badge variant="outline">{selectedMethod}</Badge>
            </div>
            {assetInfo && (
              <>
                <div className="flex items-center justify-between">
                  <span>Network:</span>
                  <Badge variant="outline">{assetInfo.network}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span>Confirmation Time:</span>
                  <span className="text-sm text-gray-600 dark:text-gray-300">
                    {assetInfo.confirmationTime || '1-10 minutes'}
                  </span>
                </div>
              </>
            )}
          </div>
        </div>

        {/* Security Information */}
        <div className="bg-gradient-to-r from-amber-50 to-yellow-50 dark:from-amber-900/20 dark:to-yellow-900/20 p-4 rounded-lg">
          <h3 className="font-semibold text-lg mb-2 flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Security Notice
          </h3>
          <div className="space-y-2 text-sm text-gray-600 dark:text-gray-300">
            <p>• Your payment is protected by blockchain security</p>
            <p>• Transaction confirmations are required for activation</p>
            <p>• All payments are final and non-refundable</p>
          </div>
        </div>

        {/* Processing Time */}
        <div className="bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 p-4 rounded-lg">
          <h3 className="font-semibold text-lg mb-2 flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Processing Time
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Payment processing typically takes 1-10 minutes depending on network conditions.
            You will receive a confirmation once your payment is verified.
          </p>
        </div>
      </CardContent>
    </Card>
  );
}