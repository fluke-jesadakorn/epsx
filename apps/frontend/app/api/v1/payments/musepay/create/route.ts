import { NextResponse } from 'next/server';
import { createPaymentService } from '@/services/payment.service';

export async function POST(req: Request) {
  try {
    const { userId, packageId, userEmail, amount, currency } = await req.json();

    if (!userId || !packageId || !amount || !currency) {
      return NextResponse.json(
        { error: 'Missing required parameters: userId, packageId, amount, currency' },
        { status: 400 }
      );
    }

    console.log('Creating payment for:', { userId, packageId, userEmail, amount, currency });

    const paymentService = createPaymentService();
    const paymentId = await paymentService.recordPayment(
      amount,
      currency,
      `Payment for package ${packageId} by user ${userId}`
    );

    if (!paymentId) {
      throw new Error('Failed to create payment record');
    }

    return NextResponse.json({
      success: true,
      data: {
        paymentId,
        orderNo: paymentId,
        message: 'Payment created successfully'
      }
    });
  } catch (error) {
    console.error('Payment creation failed:', error);
    return NextResponse.json(
      { 
        error: 'Failed to create payment',
        message: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}
