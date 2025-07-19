import { NextRequest, NextResponse } from 'next/server';
import { withPaymentAccess, withApiLimits } from '@/middleware/paymentAccess';
import { PaymentTier } from '@/types/payment/plans';

// Example: Premium ranking API
async function handler(_req: NextRequest) {
  try {
    // This endpoint requires Gold tier or higher
    const rankings = [
      { id: 1, company: 'Company A', score: 95 },
      { id: 2, company: 'Company B', score: 89 },
      // ... more ranking data
    ];

    return NextResponse.json({ 
      success: true, 
      data: rankings,
      message: 'Premium rankings accessed successfully'
    });
  } catch (_error) {
    return NextResponse.json(
      { error: 'Failed to fetch premium rankings' },
      { status: 500 }
    );
  }
}

// Apply middleware
export async function GET(req: NextRequest) {
  // Check payment tier first
  const paymentCheck = await withPaymentAccess(PaymentTier.GOLD)(req);
  if (!paymentCheck.ok) return paymentCheck;

  // Apply API limits
  const limitsCheck = await withApiLimits()(req);
  if (!limitsCheck.ok) return limitsCheck;

  // Execute handler
  return handler(req);
}
