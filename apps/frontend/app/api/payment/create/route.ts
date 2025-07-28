import { createPayment } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { amount, currency, description, orderNo } = body;

    if (!amount || !currency || !orderNo) {
      return NextResponse.json(
        { error: 'Amount, currency, and orderNo are required' },
        { status: 400 }
      );
    }

    const result = await createPayment({ amount, currency, description, orderNo });
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/payment/create:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Payment creation failed' },
      { status: 500 }
    );
  }
}