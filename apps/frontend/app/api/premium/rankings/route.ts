import { NextRequest, NextResponse } from 'next/server';
import { withPaymentAccess, withApiLimits, PaymentTier } from '@/middleware/paymentAccess';

// Cache configuration for premium endpoints
export const dynamic = 'force-dynamic';
export const revalidate = 180; // 3 minutes for premium data

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
    }, {
      headers: {
        'Cache-Control': 'private, s-maxage=180, stale-while-revalidate=300',
        'CDN-Cache-Control': 'private, s-maxage=180',
        'Vercel-CDN-Cache-Control': 'private, s-maxage=180'
      }
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
