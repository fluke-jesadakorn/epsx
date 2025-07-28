import { getPaymentStatus } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const paymentId = searchParams.get('paymentId');

    if (!paymentId) {
      return NextResponse.json(
        { error: 'Payment ID is required' },
        { status: 400 }
      );
    }

    const result = await getPaymentStatus(paymentId);
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/payment/status:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Payment status check failed' },
      { status: 500 }
    );
  }
}