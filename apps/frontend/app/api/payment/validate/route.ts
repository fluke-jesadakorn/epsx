import { validatePayment } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { paymentId, signature } = body;

    if (!paymentId) {
      return NextResponse.json(
        { error: 'Payment ID is required' },
        { status: 400 }
      );
    }

    const result = await validatePayment({ paymentId, signature });
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/payment/validate:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Payment validation failed' },
      { status: 500 }
    );
  }
}