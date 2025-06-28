import { NextResponse } from 'next/server';
import { db } from '../../../../lib/firebase';
import { doc, setDoc, getDoc } from 'firebase/firestore';
import crypto from 'crypto';

// MusePay Public Key for signature verification
const MUSEPAY_PUBLIC_KEY = `-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAjsC3qAfxiqFohyGmRLC3gIp9GnQ0Q6lYKoLlD7JEX8JSXe9fKLKTnnw2RmezZUSFfmOLvJg7dUJO/g5lX467kI+vuNliu0+ATW/EsNPC6nxg1yWASjMVQhiiz77z7m11KqzFpNXmKuzgE41nai2hkQO1Yp/KWFePHOtegjx8GEVW5ll3lzHE+wkkAUXfBr9yoiB58mXZFQqli7pOSEgzVzGBeQ4IbEi2qdhsiSYnAEepRlF6KNfT9hy1nIBZ7ZfQYxpKwa60AhXar4PiJs9c14P3xHdwSpbM5/5SQRJxxEgnDf3ayHcKJTv6KvVySlU3Mq16k2X5CChE+QXzeo37UwIDAQAB
-----END PUBLIC KEY-----`;

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
      orderNo: order_no,
      requestId: request_id,
      orderType: order_type,
      productCode: product_code,
      currency: currency || 'Unknown',
      actualAmount: parseFloat(actual_amount) || 0,
      status: status.toString(),
      reason: reason || 'N/A',
      finishTime: finish_time ? new Date(finish_time) : new Date(),
      updatedAt: new Date(),
      extraInfo: extra_info ? JSON.parse(extra_info) : {},
    };

    await setDoc(transactionRef, transactionData, { merge: true });

    // If status indicates a completed transaction, update user payment status
    // Assuming status '1' or similar indicates completion - adjust based on MusePay documentation
    if (
      status === 1 ||
      status.toString().includes('complete') ||
      status.toString().includes('success')
    ) {
      // We need userId associated with this transaction to update user status
      // This assumes userId is stored in transaction or can be fetched
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
