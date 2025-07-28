import { initQRPayment } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { amount, currency, orderNo, description } = body;

    if (!amount || !currency || !orderNo) {
      return NextResponse.json(
        { error: 'Amount, currency, and orderNo are required' },
        { status: 400 }
      );
    }

    const result = await initQRPayment({ amount, currency, orderNo, description });
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/payment/qr-init:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'QR payment initialization failed' },
      { status: 500 }
    );
  }
}