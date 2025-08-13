'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Clock, CheckCircle, XCircle, AlertCircle } from 'lucide-react';

interface PaymentStatusCardProps {
  status: 'pending' | 'completed' | 'failed' | 'processing';
  transactionId?: string;
  amount?: number;
  currency?: string;
  timestamp?: string;
  className?: string;
}

export function PaymentStatusCard({
  status,
  transactionId,
  amount,
  currency = 'USD',
  timestamp,
  className = '',
}: PaymentStatusCardProps) {
  const statusConfig = {
    pending: {
      title: 'Payment Pending',
      description: 'Your payment is being processed. This may take a few minutes.',
      icon: <Clock className="h-6 w-6 text-yellow-500" />,
      badgeColor: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/50 dark:text-yellow-300',
      borderColor: 'border-yellow-200 dark:border-yellow-800/30',
    },
    processing: {
      title: 'Payment Processing',
      description: 'Your payment is being processed. This may take a few minutes.',
      icon: <Clock className="h-6 w-6 text-blue-500" />,
      badgeColor: 'bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-300',
      borderColor: 'border-blue-200 dark:border-blue-800/30',
    },
    completed: {
      title: 'Payment Successful',
      description: 'Your payment has been processed successfully.',
      icon: <CheckCircle className="h-6 w-6 text-green-500" />,
      badgeColor: 'bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-300',
      borderColor: 'border-green-200 dark:border-green-800/30',
    },
    failed: {
      title: 'Payment Failed',
      description: 'There was an issue processing your payment. Please try again.',
      icon: <XCircle className="h-6 w-6 text-red-500" />,
      badgeColor: 'bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-300',
      borderColor: 'border-red-200 dark:border-red-800/30',
    },
  };

  const config = statusConfig[status];

  return (
    <Card className={`${className} ${config.borderColor}`}>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center justify-between text-gray-900 dark:text-white">
          <div className="flex items-center gap-2">
            {config.icon}
            <span>{config.title}</span>
          </div>
          <Badge className={config.badgeColor}>
            {status.charAt(0).toUpperCase() + status.slice(1)}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground mb-4">{config.description}</p>

        {amount !== undefined && (
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-600 dark:text-gray-400">Amount:</span>
            <span className="font-semibold text-gray-900 dark:text-white">
              {amount} {currency}
            </span>
          </div>
        )}

        {transactionId && (
          <div className="flex items-center justify-between mb-2">
            <span className="text-gray-600 dark:text-gray-400">Transaction ID:</span>
            <span className="font-medium text-sm text-gray-800 dark:text-gray-300 max-w-[200px] truncate">
              {transactionId}
            </span>
          </div>
        )}

        {timestamp && (
          <div className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400">Time:</span>
            <span className="text-sm text-gray-700 dark:text-gray-300">
              {new Date(timestamp).toLocaleString()}
            </span>
          </div>
        )}

        {status === 'failed' && (
          <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 rounded-md flex items-start gap-2">
            <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0 mt-0.5" />
            <div className="text-sm text-red-600 dark:text-red-400">
              Please try again or contact support if the problem persists.
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
