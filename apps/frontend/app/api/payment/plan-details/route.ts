import { getPlanDetails } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const planId = searchParams.get('planId');

    const result = await getPlanDetails(planId || undefined);
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/payment/plan-details:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get plan details' },
      { status: 500 }
    );
  }
}