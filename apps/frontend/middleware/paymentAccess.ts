import { NextRequest, NextResponse } from 'next/server';
import { PaymentService } from '@/services/paymentService';
import { PaymentTier } from '@/types/payment/plans';

export function withPaymentAccess(requiredTier: PaymentTier) {
  return async (req: NextRequest) => {
    try {
      // Get user's payment tier from session/auth
      // This would be replaced with actual auth implementation
      const userTier = PaymentTier.BASIC; // This should come from auth

      if (!PaymentService.hasMinimumTier(userTier, requiredTier)) {
        return NextResponse.json(
          { 
            error: 'Higher payment tier required',
            required: requiredTier,
            current: userTier 
          },
          { status: 403 }
        );
      }

      return NextResponse.next();
    } catch (error) {
      return NextResponse.json(
        { error: 'Payment verification failed' },
        { status: 500 }
      );
    }
  };
}

export function withFeatureAccess(feature: string) {
  return async (req: NextRequest) => {
    try {
      // Get user's payment tier from session/auth
      const userTier = PaymentTier.BASIC; // This should come from auth

      if (!PaymentService.hasFeatureAccess(userTier, feature)) {
        return NextResponse.json(
          { 
            error: 'Feature not available in current plan',
            feature,
            requiredFeatures: PaymentService.getApiFeaturesByTier(PaymentTier.GOLD)
          },
          { status: 403 }
        );
      }

      return NextResponse.next();
    } catch (error) {
      return NextResponse.json(
        { error: 'Feature verification failed' },
        { status: 500 }
      );
    }
  };
}

export function withApiLimits() {
  return async (req: NextRequest) => {
    try {
      // Get user's payment tier from session/auth
      const userTier = PaymentTier.BASIC; // This should come from auth
      const limits = PaymentService.getApiLimitsByTier(userTier);

      // Add rate limiting headers
      const response = NextResponse.next();
      response.headers.set('X-Rate-Limit', limits.requestsPerMinute.toString());
      response.headers.set('X-Rate-Limit-Daily', limits.requestsPerDay.toString());
      response.headers.set('X-Max-File-Size', limits.maxFileSize.toString());

      return response;
    } catch (error) {
      return NextResponse.json(
        { error: 'Rate limit verification failed' },
        { status: 500 }
      );
    }
  };
}
