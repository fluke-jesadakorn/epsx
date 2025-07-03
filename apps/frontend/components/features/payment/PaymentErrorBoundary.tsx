'use client';

import React from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader } from '@/components/ui/card';
import type { PaymentError } from '@/app/constants/packages';

interface Props {
  children: React.ReactNode;
  onReset?: () => void;
  fallback?: React.ReactNode;
}

interface State {
  error: PaymentError | null;
}

export class PaymentErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  private static isPaymentError(error: unknown): error is PaymentError {
    if (typeof error !== 'object' || !error) return false;
    return 'type' in error && typeof (error as any).type === 'string';
  }

  static getDerivedStateFromError(error: unknown): State {
    const defaultError: PaymentError = {
      type: 'TRANSACTION_FAILED',
      reason: error instanceof Error ? error.message : 'Unknown error occurred'
    };

    return {
      error: PaymentErrorBoundary.isPaymentError(error) ? error : defaultError
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Payment error:', error, errorInfo);
  }

  render() {
    if (this.state.error) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <Card className="w-full max-w-md mx-auto bg-destructive/5 border-destructive/50">
          <CardHeader className="space-y-1">
            <h3 className="text-lg font-semibold">Payment Error</h3>
            <p className="text-sm text-muted-foreground">
              {this.getErrorMessage(this.state.error)}
            </p>
          </CardHeader>
          <CardContent>
            {this.getErrorDetails(this.state.error)}
          </CardContent>
          <CardFooter>
            <Button
              variant="outline"
              className="w-full"
              onClick={() => {
                this.setState({ error: null });
                this.props.onReset?.();
              }}
            >
              Try Again
            </Button>
          </CardFooter>
        </Card>
      );
    }

    return this.props.children;
  }

  private getErrorMessage(error: PaymentError): string {
    switch (error.type) {
      case 'INSUFFICIENT_AMOUNT':
        return `Minimum amount required: ${error.minAmount} ${error.currency}`;
      case 'INVALID_CURRENCY':
        return 'Selected currency is not supported';
      case 'NETWORK_ERROR':
        return 'Network connection error. Please check your connection and try again.';
      case 'TRANSACTION_FAILED':
        return `Transaction failed: ${error.reason}`;
      default:
        return 'An unexpected error occurred';
    }
  }

  private getErrorDetails(error: PaymentError): React.ReactNode {
    switch (error.type) {
      case 'INSUFFICIENT_AMOUNT':
        return (
          <div className="space-y-2 text-sm">
            <p>Please ensure your payment meets the minimum amount requirement:</p>
            <ul className="list-disc list-inside">
              <li>Current Currency: {error.currency}</li>
              <li>Minimum Amount: {error.minAmount}</li>
            </ul>
          </div>
        );
      case 'NETWORK_ERROR':
        return (
          <div className="space-y-2 text-sm">
            <p>Troubleshooting steps:</p>
            <ul className="list-disc list-inside">
              <li>Check your internet connection</li>
              <li>Make sure you're connected to the correct network</li>
              <li>Try refreshing the page</li>
            </ul>
          </div>
        );
      case 'TRANSACTION_FAILED':
        return (
          <div className="space-y-2 text-sm">
            <p>Transaction Details:</p>
            <p className="text-destructive">{error.reason}</p>
            <p>Please try again or contact support if the issue persists.</p>
          </div>
        );
      default:
        return null;
    }
  }
}
