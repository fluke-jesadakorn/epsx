// POST /api/payment/deposit-address
import { NextRequest, NextResponse } from 'next/server';
import { createPaymentService } from '@/services/payment.service';
import { nanoid } from 'nanoid';

export async function POST(req: NextRequest) {
  try {
    const { currency, userId, packageId, description } = await req.json();

    if (!currency || !userId || !packageId) {
      return NextResponse.json(
        { error: 'Missing required fields' },
        { status: 400 },
      );
    }

    // Generate customer reference ID
    const customerRefId = `${userId}_${packageId}_${nanoid(8)}`;
    
    // Initialize payment service
    const paymentService = createPaymentService();
    
    // For deposit address, we need to create a payment record first
    const paymentId = await paymentService.recordPayment(
      100, // Default amount, should be passed from client
      currency,
      description || 'Deposit payment'
    );

    if (!paymentId) {
      return NextResponse.json(
        { error: 'Failed to create payment record' },
        { status: 500 },
      );
    }

    // Return payment information instead of deposit address
    // The actual crypto integration would be handled by the backend
    return NextResponse.json({ 
      paymentId,
      customerRefId,
      message: 'Payment initiated. Backend will handle crypto deposit address generation.'
    });
  } catch (error: any) {
    console.error('Payment deposit-address API error:', error);
    return NextResponse.json(
      { error: error.message || 'Internal error', details: error },
      { status: 500 },
    );
  }
}
