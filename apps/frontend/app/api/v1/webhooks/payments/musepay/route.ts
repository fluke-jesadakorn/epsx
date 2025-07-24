import { NextResponse } from 'next/server';
import { createPaymentService } from '@/services/payment.service';

// Define webhook payload type locally since musepay.service was removed
interface WebhookPayload {
  order_no: string;
  status: string;
  amount: number;
  currency: string;
  tx_hash?: string;
  network?: string;
}

export async function POST(req: Request) {
  try {
    const body: WebhookPayload = await req.json();
    console.log('Received payment webhook:', body);

    // Process webhook using the new payment service
    const paymentService = createPaymentService();
    
    // For webhooks, we would typically confirm the payment
    const result = await paymentService.confirmPayment(
      body.order_no,
      'webhook',
      'BRONZE' // Default user level, should be determined by payment amount/package
    );

    if (!result.success) {
      console.error('Failed to process payment webhook:', result.message);
      return NextResponse.json(
        { error: 'Webhook processing failed' },
        { status: 400 },
      );
    }

    console.log(`Successfully processed webhook for order ${body.order_no}`);
    return NextResponse.json({ success: true }, { status: 200 });
  } catch (error) {
    console.error('Error processing payment webhook:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 },
    );
  }
}
