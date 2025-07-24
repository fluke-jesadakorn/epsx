import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { 
  CheckCircle, 
  Clock, 
  AlertCircle,
  CreditCard,
  Calendar
} from 'lucide-react';
import { getPaymentDetails } from '@/app/actions/payment';

interface PaymentStatusServerProps {
  userId?: string;
  className?: string;
}

export default async function PaymentStatusServer({
  userId,
  className = ''
}: PaymentStatusServerProps) {
  // Server-side data fetching
  const paymentDetails = await getPaymentDetails();
  
  if (!paymentDetails) {
    return (
      <Card className={`w-full ${className}`}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CreditCard className="h-5 w-5" />
            Payment Status
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-6">
            <AlertCircle className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-600 dark:text-gray-300">
              No payment information available
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const { paymentStatus, userLevel } = paymentDetails;
  const isActive = paymentStatus.expirationDate > new Date();

  return (
    <Card className={`w-full ${className}`}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <CreditCard className="h-5 w-5" />
          Payment Status
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Current Status */}
        <div className="flex items-center justify-between p-4 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-lg">
          <div className="flex items-center gap-2">
            {isActive ? (
              <CheckCircle className="h-5 w-5 text-green-500" />
            ) : (
              <Clock className="h-5 w-5 text-yellow-500" />
            )}
            <span className="font-medium">Current Status</span>
          </div>
          <Badge variant={isActive ? "default" : "secondary"}>
            {isActive ? "Active" : "Expired"}
          </Badge>
        </div>

        {/* User Level */}
        <div className="flex items-center justify-between p-4 bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 rounded-lg">
          <div className="flex items-center gap-2">
            <span className="font-medium">Current Level</span>
          </div>
          <Badge variant="outline" className="bg-white dark:bg-gray-800">
            {userLevel}
          </Badge>
        </div>

        {/* Payment Details */}
        <div className="space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-600 dark:text-gray-300">Payment Method:</span>
            <span className="font-medium">{paymentStatus.paymentMethod}</span>
          </div>
          
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-600 dark:text-gray-300">Last Payment:</span>
            <span className="font-medium">
              {paymentStatus.lastPaymentDate.toLocaleDateString()}
            </span>
          </div>
          
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-600 dark:text-gray-300">Expires:</span>
            <span className={`font-medium ${isActive ? 'text-green-600' : 'text-red-600'}`}>
              {paymentStatus.expirationDate.toLocaleDateString()}
            </span>
          </div>
        </div>

        {/* Renewal Notice */}
        {!isActive && (
          <div className="p-4 bg-gradient-to-r from-yellow-50 to-amber-50 dark:from-yellow-900/20 dark:to-amber-900/20 rounded-lg border border-yellow-200 dark:border-yellow-800">
            <div className="flex items-center gap-2 text-yellow-800 dark:text-yellow-200">
              <Calendar className="h-4 w-4" />
              <span className="font-medium text-sm">Renewal Required</span>
            </div>
            <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
              Your subscription has expired. Renew to continue accessing premium features.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}