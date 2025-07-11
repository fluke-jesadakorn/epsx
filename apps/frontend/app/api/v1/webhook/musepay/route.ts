import { NextResponse } from 'next/server';
import { musePayService, type WebhookPayload } from '@/lib/musepay.service';

export async function POST(req: Request) {
  try {
    const body: WebhookPayload = await req.json();
    console.log('Received MusePay webhook:', body);

    // Process webhook using the MusePay service
    const success = await musePayService.processWebhook(body);

    if (!success) {
      console.error('Failed to process MusePay webhook');
      return NextResponse.json(
        { error: 'Webhook processing failed' },
        { status: 400 },
      );
    }

    console.log(`Successfully processed webhook for order ${body.order_no}`);
    return NextResponse.json({ success: true }, { status: 200 });
  } catch (error) {
    console.error('Error processing MusePay webhook:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 },
    );
  }
}
