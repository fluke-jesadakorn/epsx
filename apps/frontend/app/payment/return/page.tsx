'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Check, Clock, X, ArrowLeft, Loader2 } from 'lucide-react';
import { onSnapshot, collection, query, where } from 'firebase/firestore';
import { db } from '@/lib/firebase';

export default function PaymentReturnPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [paymentStatus, setPaymentStatus] = useState<'checking' | 'success' | 'pending' | 'failed'>('checking');
  const [paymentData, setPaymentData] = useState<any>(null);
  const [countdown, setCountdown] = useState(10);

  useEffect(() => {
    // Load payment data from session storage
    const storedPayment = sessionStorage.getItem('activePayment');
    if (storedPayment) {
      try {
        const payment = JSON.parse(storedPayment);
        setPaymentData(payment);
        
        // Monitor payment status in real-time
        if (payment.paymentRequest?.customerRefId) {
          const unsubscribe = onSnapshot(
            query(
              collection(db, 'payment_requests'),
              where('customerRefId', '==', payment.paymentRequest.customerRefId)
            ),
            (snapshot) => {
              if (!snapshot.empty) {
                const paymentRequest = snapshot.docs[0].data();
                console.log('Payment status update:', paymentRequest.status);
                
                if (paymentRequest.status === 'completed') {
                  setPaymentStatus('success');
                  // Clear session storage
                  sessionStorage.removeItem('activePayment');
                } else if (paymentRequest.status === 'failed') {
                  setPaymentStatus('failed');
                } else {
                  setPaymentStatus('pending');
                }
              }
            },
            (error) => {
              console.error('Error monitoring payment:', error);
              setPaymentStatus('failed');
            }
          );

          return () => unsubscribe();
        }
      } catch (error) {
        console.error('Failed to parse stored payment:', error);
        setPaymentStatus('failed');
      }
    } else {
      // No payment data found
      setPaymentStatus('failed');
    }
  }, []);

  // Countdown for automatic redirect on success
  useEffect(() => {
    if (paymentStatus === 'success' && countdown > 0) {
      const timer = setTimeout(() => {
        setCountdown(countdown - 1);
      }, 1000);
      return () => clearTimeout(timer);
    } else if (paymentStatus === 'success' && countdown === 0) {
      router.push('/dashboard?payment=success');
    }
  }, [paymentStatus, countdown, router]);

  const handleReturnToDashboard = () => {
    if (paymentStatus === 'success') {
      router.push('/dashboard?payment=success');
    } else {
      router.push('/dashboard');
    }
  };

  const handleRetryPayment = () => {
    router.push('/payment');
  };

  if (paymentStatus === 'checking') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">
        <Card className="max-w-md mx-auto border-0 shadow-2xl bg-gradient-to-br from-white via-blue-50/50 to-cyan-50/50 dark:from-gray-800 dark:via-blue-900/20 dark:to-cyan-900/20 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-400/5 via-cyan-400/5 to-teal-400/5"></div>
          <CardContent className="pt-8 text-center relative z-10">
            <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-blue-400 to-cyan-500 rounded-full flex items-center justify-center shadow-lg">
              <Loader2 className="h-10 w-10 text-white animate-spin" />
            </div>
            <h3 className="text-2xl font-bold mb-3 bg-gradient-to-r from-blue-600 to-cyan-600 bg-clip-text text-transparent">
              Checking Payment Status 🔍
            </h3>
            <p className="text-gray-600 dark:text-gray-300 mb-6 text-lg">
              Please wait while we verify your payment...
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (paymentStatus === 'success') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">
        <Card className="max-w-md mx-auto border-0 shadow-2xl bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-green-400/10 via-emerald-400/10 to-teal-400/10"></div>
          <CardContent className="pt-8 text-center relative z-10">
            <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-green-400 to-emerald-500 rounded-full flex items-center justify-center shadow-lg animate-bounce">
              <Check className="h-10 w-10 text-white" />
            </div>
            <h3 className="text-2xl font-bold mb-3 bg-gradient-to-r from-green-600 to-emerald-600 bg-clip-text text-transparent">
              Payment Successful! 🎉
            </h3>
            <p className="text-gray-600 dark:text-gray-300 mb-6 text-lg">
              Your {paymentData?.paymentRequest?.packageName} has been activated successfully.
            </p>
            <div className="mb-6">
              <div className="inline-flex items-center gap-2 text-sm font-medium text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30 px-4 py-2 rounded-full">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                Redirecting to dashboard in {countdown}s...
              </div>
            </div>
            <Button
              onClick={handleReturnToDashboard}
              className="w-full h-12 bg-gradient-to-r from-green-500 via-emerald-500 to-teal-500 hover:from-green-600 hover:via-emerald-600 hover:to-teal-600 text-white border-0 shadow-xl transition-all duration-300 hover:scale-[1.02]"
            >
              Go to Dashboard Now
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (paymentStatus === 'pending') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">
        <Card className="max-w-md mx-auto border-0 shadow-2xl bg-gradient-to-br from-white via-yellow-50/50 to-orange-50/50 dark:from-gray-800 dark:via-yellow-900/20 dark:to-orange-900/20 relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-yellow-400/5 via-orange-400/5 to-amber-400/5"></div>
          <CardContent className="pt-8 text-center relative z-10">
            <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-full flex items-center justify-center shadow-lg">
              <Clock className="h-10 w-10 text-white animate-pulse" />
            </div>
            <h3 className="text-2xl font-bold mb-3 bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-transparent">
              Payment Pending ⏳
            </h3>
            <p className="text-gray-600 dark:text-gray-300 mb-6 text-lg">
              Your payment is being processed. This usually takes 1-10 minutes.
            </p>
            <div className="space-y-3">
              <Button
                onClick={handleReturnToDashboard}
                variant="outline"
                className="w-full h-12 border-2 border-gray-300 dark:border-gray-600 hover:border-yellow-400 dark:hover:border-yellow-500 font-semibold flex items-center gap-2"
              >
                <ArrowLeft className="h-4 w-4" />
                Return to Dashboard
              </Button>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                We'll notify you once the payment is confirmed
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Failed status
  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800">
      <Card className="max-w-md mx-auto border-0 shadow-2xl bg-gradient-to-br from-white via-red-50/50 to-pink-50/50 dark:from-gray-800 dark:via-red-900/20 dark:to-pink-900/20 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-red-400/5 via-pink-400/5 to-rose-400/5"></div>
        <CardContent className="pt-8 text-center relative z-10">
          <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-red-400 to-pink-500 rounded-full flex items-center justify-center shadow-lg">
            <X className="h-10 w-10 text-white" />
          </div>
          <h3 className="text-2xl font-bold mb-3 bg-gradient-to-r from-red-600 to-pink-600 bg-clip-text text-transparent">
            Payment Issue ❌
          </h3>
          <p className="text-gray-600 dark:text-gray-300 mb-6 text-lg">
            There was an issue with your payment. Please try again or contact support.
          </p>
          <div className="space-y-3">
            <Button
              onClick={handleRetryPayment}
              className="w-full h-12 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 hover:from-blue-600 hover:via-purple-600 hover:to-pink-600 text-white border-0 shadow-xl transition-all duration-300 hover:scale-[1.02]"
            >
              Try Again
            </Button>
            <Button
              onClick={handleReturnToDashboard}
              variant="outline"
              className="w-full h-12 border-2 border-gray-300 dark:border-gray-600 hover:border-red-400 dark:hover:border-red-500 font-semibold flex items-center gap-2"
            >
              <ArrowLeft className="h-4 w-4" />
              Return to Dashboard
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
