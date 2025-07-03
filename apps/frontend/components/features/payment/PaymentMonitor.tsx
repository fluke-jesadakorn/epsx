'use client';

import { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

// Simple Progress component if not available
const Progress = ({ value, className }: { value: number; className?: string }) => (
  <div className={`w-full bg-gray-200 rounded-full h-2.5 ${className}`}>
    <div 
      className="bg-blue-600 h-2.5 rounded-full transition-all duration-300 ease-out" 
      style={{ width: `${value}%` }}
    />
  </div>
);

interface PaymentMonitorProps {
  paymentId: string;
  onStatusChange?: (status: string) => void;
}

interface PaymentStatus {
  id: string;
  status: 'pending' | 'processing' | 'succeeded' | 'failed' | 'cancelled';
  amount: number;
  currency: string;
  created_at: string;
  updated_at: string;
  expiration_date: string;
  transaction_hash?: string;
  network?: string;
  confirmations?: number;
  required_confirmations?: number;
  error_message?: string;
}

export default function PaymentMonitor({ paymentId, onStatusChange }: PaymentMonitorProps) {
  const [payment, setPayment] = useState<PaymentStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchPaymentStatus = useCallback(async () => {
    try {
      const response = await fetch(`/api/v1/payment/${paymentId}/status`);
      if (!response.ok) {
        throw new Error('Failed to fetch payment status');
      }
      const data = await response.json();
      setPayment(data);
      setError(null);
      
      if (onStatusChange) {
        onStatusChange(data.status);
      }
    } catch (err) {
      console.error('Error fetching payment status:', err);
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [paymentId, onStatusChange]);

  useEffect(() => {
    fetchPaymentStatus();
    
    // Set up polling for real-time updates
    const interval = setInterval(fetchPaymentStatus, 5000); // Poll every 5 seconds
    
    return () => clearInterval(interval);
  }, [fetchPaymentStatus]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'succeeded':
        return 'bg-green-100 text-green-800';
      case 'processing':
        return 'bg-blue-100 text-blue-800';
      case 'pending':
        return 'bg-yellow-100 text-yellow-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
      case 'cancelled':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'succeeded':
        return '✅';
      case 'processing':
        return '🔄';
      case 'pending':
        return '⏳';
      case 'failed':
        return '❌';
      case 'cancelled':
        return '🚫';
      default:
        return '⚪';
    }
  };

  const getConfirmationProgress = () => {
    if (!payment?.confirmations || !payment?.required_confirmations) {
      return 0;
    }
    return Math.min((payment.confirmations / payment.required_confirmations) * 100, 100);
  };

  const formatTimeAgo = (date: string) => {
    const now = new Date();
    const past = new Date(date);
    const diffMs = now.getTime() - past.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return `${Math.floor(diffMins / 1440)}d ago`;
  };

  if (loading && !payment) {
    return (
      <Card className="w-full animate-pulse">
        <CardHeader>
          <div className="h-6 bg-gray-200 rounded w-1/3"></div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="h-4 bg-gray-200 rounded w-1/2"></div>
            <div className="h-4 bg-gray-200 rounded w-1/4"></div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="w-full border-red-200">
        <CardContent className="pt-6">
          <div className="text-center space-y-4">
            <div className="text-red-600 flex items-center justify-center gap-2">
              <span className="text-2xl">⚠️</span>
              <span className="font-medium">Error loading payment status</span>
            </div>
            <p className="text-sm text-red-600">{error}</p>
            <Button
              variant="outline"
              onClick={fetchPaymentStatus}
              className="border-red-200 text-red-600"
            >
              Retry
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!payment) {
    return null;
  }

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span className="flex items-center gap-2">
            <span className="text-2xl">💳</span>
            Payment Status
          </span>
          <Badge className={getStatusColor(payment.status)}>
            {getStatusIcon(payment.status)} {payment.status.toUpperCase()}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-sm text-gray-600">Amount</p>
            <p className="text-lg font-semibold">
              {payment.amount} {payment.currency}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-600">Payment ID</p>
            <p className="text-sm font-mono bg-gray-100 px-2 py-1 rounded">
              {payment.id}
            </p>
          </div>
        </div>

        {payment.transaction_hash && (
          <div>
            <p className="text-sm text-gray-600 mb-2">Transaction Hash</p>
            <div className="flex items-center gap-2">
              <p className="text-sm font-mono bg-gray-100 px-2 py-1 rounded flex-1">
                {payment.transaction_hash}
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => navigator.clipboard.writeText(payment.transaction_hash!)}
              >
                Copy
              </Button>
            </div>
          </div>
        )}

        {payment.confirmations !== undefined && payment.required_confirmations && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <p className="text-sm text-gray-600">Confirmations</p>
              <p className="text-sm font-semibold">
                {payment.confirmations} / {payment.required_confirmations}
              </p>
            </div>
            <Progress value={getConfirmationProgress()} className="w-full" />
          </div>
        )}

        {payment.error_message && (
          <div className="bg-red-50 border border-red-200 rounded-md p-3">
            <p className="text-sm text-red-800">
              <span className="font-medium">Error:</span> {payment.error_message}
            </p>
          </div>
        )}

        <div className="pt-4 border-t">
          <div className="flex items-center justify-between text-sm text-gray-600">
            <span>Last updated: {formatTimeAgo(payment.updated_at)}</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={fetchPaymentStatus}
              disabled={loading}
            >
              🔄 Refresh
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
