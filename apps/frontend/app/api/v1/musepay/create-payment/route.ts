import { NextResponse } from 'next/server';
import { musePayService } from '@/lib/musepay.service';

export async function POST(req: Request) {
  try {
    const { userId, packageId, userEmail } = await req.json();

    if (!userId || !packageId) {
      return NextResponse.json(
        { error: 'Missing required parameters: userId and packageId' },
        { status: 400 }
      );
    }

    console.log('Creating MusePay payment for:', { userId, packageId, userEmail });

    const result = await musePayService.createPayment(userId, packageId, userEmail);

    return NextResponse.json({
      success: true,
      data: result
    });
  } catch (error) {
    console.error('MusePay payment creation failed:', error);
    return NextResponse.json(
      { 
        error: 'Failed to create payment',
        message: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}
