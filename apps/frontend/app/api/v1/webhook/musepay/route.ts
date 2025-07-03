import { NextResponse } from 'next/server';
import { db } from '../../../../../lib/firebase';
import { doc, setDoc, getDoc, increment } from 'firebase/firestore';
import {
  getUserLevel,
  PAYMENT_DURATION,
  BLOCKCHAIN_CONFIG,
  TRANSACTION_STATUSES,
} from '../../../../constants/packages';

// Status mapping
const STATUS_MAP: Record<number | string, string> = {
  1: 'completed',
  2: 'pending',
  3: 'failed',
  success: 'completed',
  pending: 'pending',
  failed: 'failed',
};

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
    console.log('Received webhook request:', body);
    const signature = body.sign;

    if (!signature) {
      console.error('No signature provided in webhook request');
      return NextResponse.json(
        { error: 'Invalid request: No signature provided' },
        { status: 403 },
      );
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

    console.log(`Received webhook for order ${order_no} with status ${status}`);

    const normalizedStatus = STATUS_MAP[status] || TRANSACTION_STATUSES.PENDING;
    const extraInfoObj = extra_info ? JSON.parse(extra_info) : {};

    // Extract blockchain data
    const blockchainData = {
      txHash: extraInfoObj.txnHash || '',
      blockHeight: extraInfoObj.blockHeight || '',
      network: extraInfoObj.network || '',
      sourceAddress: extraInfoObj.sourceAddress || '',
      destinationAddress: extraInfoObj.destinationAddress || '',
      networkFee: extraInfoObj.networkFee || '0',
    };

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
      status: normalizedStatus,
      reason: reason || 'N/A',
      finishTime: finish_time ? new Date(finish_time) : new Date(),
      updatedAt: new Date(),
      signature: body.sign || 'N/A',
      blockchainData,
      blockExplorerUrl: `${BLOCKCHAIN_CONFIG.BSC.explorerUrl}${blockchainData.txHash}`,
    };

    await setDoc(transactionRef, transactionData, { merge: true });

    // Update user payment status if transaction is completed
    if (normalizedStatus === 'completed') {
      // We need userId associated with this transaction to update user status
      // This assumes userId is stored in transaction or can be fetched. Future improvement: implement a fallback mechanism to lookup userId based on request_id or order_no if not stored.
      const transactionSnap = await getDoc(transactionRef);
      const userId = transactionSnap.data()?.userId;

      if (userId) {
        const userRef = doc(db, 'users', userId);
        const userSnap = await getDoc(userRef);
        const userData = userSnap.data() || {};

        // Increment payment count and get new level
        const currentPayments = (userData.paymentCount || 0) + 1;
        const newLevel = getUserLevel(currentPayments);

        // Set expiration to 1 month from now
        const newExpirationDate = new Date(
          Date.now() + PAYMENT_DURATION.MILLISECONDS,
        );

        await setDoc(
          userRef,
          {
            paymentStatus: {
              hasPaid: true,
              lastPaymentDate: new Date(),
              expirationDate: newExpirationDate,
            },
            userLevel: newLevel,
            paymentCount: increment(1),
            totalAmountPaid: increment(parseFloat(actual_amount) || 0),
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
