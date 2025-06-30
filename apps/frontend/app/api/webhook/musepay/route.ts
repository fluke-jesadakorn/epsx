import { NextResponse } from 'next/server';
import { db } from '../../../../lib/firebase';
import { doc, setDoc, getDoc } from 'firebase/firestore';
import crypto from 'crypto';

// MusePay Public Key for signature verification, loaded from environment variable
const MUSEPAY_PUBLIC_KEY = process.env.MUSEPAY_PUBLIC_KEY || '';
if (!MUSEPAY_PUBLIC_KEY) {
  throw new Error(
    'MUSEPAY_PUBLIC_KEY environment variable is not set. Please set it in your environment variables.',
  );
}

export async function POST(req: Request) {
  try {
    const body = await req.json();
    const signature = body.sign;

    if (!signature) {
      console.error('No signature provided in webhook request');
      return NextResponse.json(
        { error: 'Invalid request: No signature provided' },
        { status: 403 },
      );
    }

    // Verify the signature
    const msgBody = JSON.stringify(body);
    const verifier = crypto.createVerify('SHA1');
    verifier.update(msgBody);
    const isValid = verifier.verify(MUSEPAY_PUBLIC_KEY, signature, 'base64');

    if (!isValid) {
      console.error('Signature verification failed for webhook request');
      return NextResponse.json({ error: 'Invalid signature' }, { status: 403 });
    }

    // Extract relevant data from the notification
    const {
      order_no,
      status,
      actual_amount,
      currency,
      finish_time,
      request_id,
      order_type,
      product_code,
      reason,
      extra_info,
    } = body;

    // Log the received webhook for debugging
    console.log(`Received webhook for order ${order_no} with status ${status}`);

    // Update transaction record in Firestore
    const transactionRef = doc(db, 'transactions', order_no || request_id);
    const transactionData = {
      partnerId: body.partner_id || 'N/A',
      orderNo: order_no,
      requestId: request_id,
      orderType: order_type,
      productCode: product_code,
      currency: currency || 'Unknown',
      orderAmount: parseFloat(body.order_amount) || 0,
      feeAmount: parseFloat(body.fee_amount) || 0,
      actualAmount: parseFloat(actual_amount) || 0,
      status: status.toString(),
      reason: reason || 'N/A',
      finishTime: finish_time ? new Date(finish_time) : new Date(),
      updatedAt: new Date(),
      signature: body.sign || 'N/A',
      extraInfo: extra_info ? JSON.parse(extra_info) : {},
    };

    await setDoc(transactionRef, transactionData, { merge: true });

    // If status indicates a completed transaction, update user payment status
    // According to MusePay documentation, status is a Number. Adjust the condition below to match specific status codes for completed transactions.
    if (
      status === 1 ||
      status.toString().includes('complete') ||
      status.toString().includes('success')
    ) {
      // We need userId associated with this transaction to update user status
      // This assumes userId is stored in transaction or can be fetched. Future improvement: implement a fallback mechanism to lookup userId based on request_id or order_no if not stored.
      const transactionSnap = await getDoc(transactionRef);
      const userId = transactionSnap.data()?.userId;

      if (userId) {
        const userRef = doc(db, 'users', userId);
        await setDoc(
          userRef,
          {
            paymentStatus: {
              hasPaid: true,
              lastPaymentDate: new Date(),
              expirationDate: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000), // 1 year from now
            },
            userLevel: 'Premium', // Adjust based on transaction details or product_code if needed
          },
          { merge: true },
        );
        console.log(`Updated payment status for user ${userId}`);
      } else {
        console.warn(
          `No userId found for transaction ${order_no || request_id}`,
        );
      }
    }

    // Return 200 OK to acknowledge receipt
    return NextResponse.json({ success: true }, { status: 200 });
  } catch (error) {
    console.error('Error processing MusePay webhook:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 },
    );
  }
}
