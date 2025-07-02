'use client';

import { useState, useEffect } from 'react';
import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { createPaymentService } from '@/services/payment.service';
import { auth } from '@/lib/firebase';
import { useRouter } from 'next/navigation';
import type { PaymentResponse } from '@/types/payment';
import type { UserLevel } from '@/types/userLevel';

const StatusIndicator = ({ status }: { status: PaymentResponse['status'] }) => {
  const getStatusColor = () => {
    switch (status) {
      case 'Succeeded':
        return 'text-green-500';
      case 'Pending':
      case 'Processing':
        return 'text-yellow-500';
      case 'Failed':
        return 'text-red-500';
      default:
        return 'text-gray-500';
    }
  };

  const getStatusEmoji = () => {
    switch (status) {
      case 'Succeeded':
        return '🟢';
      case 'Pending':
      case 'Processing':
        return '🟡';
      case 'Failed':
        return '🔴';
      default:
        return '⚪';
    }
  };

  return (
    <div className={`flex items-center gap-2 text-lg font-medium ${getStatusColor()}`}>
      <span>{getStatusEmoji()}</span>
      <span>{status}</span>
    </div>
  );
};

export function PaymentStatusCard() {
  const [paymentStatus, setPaymentStatus] = useState<PaymentResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date>(new Date());
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  const router = useRouter();
  const paymentService = createPaymentService();

  useEffect(() => {
    const unsubscribe = auth.onAuthStateChanged((user) => {
      setIsAuthenticated(!!user);
      if (user) {
        fetchStatus();
      } else {
        setLoading(false);
        setError('Please log in to view payment status');
      }
    });

    return () => unsubscribe();
  }, []);

  const fetchStatus = async () => {
    setLoading(true);
    setError(null);
    try {
      const status = await paymentService.getPaymentStatus();
      if (status) {
        // Convert the payment status to PaymentResponse format
        const paymentResponse: PaymentResponse = {
          id: 'current',
          amount: 0, // This would come from the actual transaction
          currency: 'USDT',
          status: status.hasPaid ? 'Succeeded' : 'Pending',
          created_at: status.lastPaymentDate?.toISOString() || new Date().toISOString(),
          expiration_date: status.expirationDate?.toISOString() || new Date().toISOString(),
          user_level: (status.userLevel || 'Basic') as UserLevel,
          qr_code: '' // This would be set if needed
        };
        setPaymentStatus(paymentResponse);
      }
    } catch (error) {
      console.error('Failed to fetch payment status:', error);
      setError('Failed to load payment status. Please try again.');
    } finally {
      setLoading(false);
      setLastUpdated(new Date());
    }
  };

  // Auto-refresh every 5 minutes if authenticated
  useEffect(() => {
    if (isAuthenticated) {
      const interval = setInterval(fetchStatus, 5 * 60 * 1000);
      return () => clearInterval(interval);
    }
  }, [isAuthenticated]);

  if (error) {
    return (
      <Card className="w-full transition-shadow hover:shadow-lg border-warning border bg-warning/10">
        <CardContent className="pt-6">
          <div className="text-center space-y-4">
            <div className="text-warning flex items-center justify-center gap-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
              <span className="font-medium">{error}</span>
            </div>
            {!isAuthenticated && (
              <Button
                onClick={() => router.push('/login')}
                className="bg-warning text-white hover:bg-warning/90"
              >
                Log In
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    );
  }

  if (loading && !paymentStatus) {
    return (
      <Card className="w-full animate-pulse">
        <CardHeader>
          <div className="h-6 bg-gray-200 rounded w-1/3 dark:bg-gray-700"></div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="h-4 bg-gray-200 rounded w-1/2 dark:bg-gray-700"></div>
            <div className="h-4 bg-gray-200 rounded w-1/4 dark:bg-gray-700"></div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-full transition-shadow hover:shadow-lg border-primary border bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-pink-500/10">
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="bg-primary/10 rounded-full p-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6 text-primary"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </span>
          <h2 className="text-xl font-semibold text-primary">Payment Status</h2>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={fetchStatus}
          title="Refresh status"
          className="rounded-full hover:bg-primary/10"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z"
              clipRule="evenodd"
            />
          </svg>
        </Button>
      </CardHeader>
      <CardContent className="space-y-4">
        {paymentStatus && (
          <>
            <StatusIndicator status={paymentStatus.status} />
            <div className="space-y-2 text-sm text-muted-foreground">
              <p>
                <span className="font-medium">Last Updated:</span>{' '}
                {lastUpdated.toLocaleTimeString()}
              </p>
              <p>
                <span className="font-medium">User Level:</span>{' '}
                {paymentStatus.user_level}
              </p>
              {paymentStatus.expiration_date && (
                <p>
                  <span className="font-medium">Valid Until:</span>{' '}
                  {new Date(paymentStatus.expiration_date).toLocaleDateString()}
                </p>
              )}
            </div>
            <Button
              variant="outline"
              className="w-full mt-4"
              onClick={() => window.location.href = '/settings/payment'}
            >
              View Payment History
            </Button>
          </>
        )}
      </CardContent>
    </Card>
  );
}
