import { hex2b64, KJUR } from 'jsrsasign';
import queryString from 'query-string';
import {
  doc,
  setDoc,
  updateDoc,
  getDoc,
  collection,
  query,
  where,
  getDocs,
  serverTimestamp,
  increment,
  arrayUnion,
} from 'firebase/firestore';
import { db } from '@/lib/firebase';
import { getPackageById, PAYMENT_DURATION } from '@/app/constants/packages';

// MusePay Configuration
const MUSEPAY_CONFIG = {
  partnerId: process.env.MUSEPAY_PARTNER_ID || '2000109',
  privateKey: process.env.MUSEPAY_PRIVATE_KEY || '',
  publicKey: process.env.MUSEPAY_PUBLIC_KEY || '',
  apiUrl: process.env.MUSEPAY_API_URL || 'https://api.test.topay.mobi/v1',
  notifyUrl:
    process.env.NEXT_PUBLIC_MUSEPAY_NOTIFY_URL ||
    'http://localhost:3000/api/v1/webhook/musepay',
};

// Interfaces
export interface PaymentRequest {
  id: string;
  customerRefId: string;
  userId: string;
  packageId: string;
  amount: number;
  currency: string;
  status: 'pending' | 'completed' | 'failed' | 'expired';

  // MusePay Response Data
  musePayOrderNo?: string;
  musePayRequestId?: string;
  receiveAddress?: string;
  checkoutUrl?: string;

  // Timestamps
  createdAt: any;
  expiresAt: Date;
  completedAt?: any;

  // User & Package Info
  userEmail?: string;
  packageName: string;
  packageLevel: string;
}

export interface MusePayResponse {
  code: string;
  data: {
    request_id: string;
    partner_id: string;
    order_no: string;
    currency: string;
    order_amount: string;
    status: number;
    payment_method: string;
    receive_address: string;
    checkout_url: string;
    pay_currency: string;
    pay_amount: string;
  };
  message: string;
}

export interface WebhookPayload {
  actual_amount: string;
  currency: string;
  customer_ref_id: string;
  extra_info: string;
  fee_amount: string;
  finish_time: string;
  order_amount: string;
  order_no: string;
  order_type: string;
  partner_id: string;
  pay_amount: string;
  product_code: string;
  reason: string;
  request_id: string;
  settle_currency: string;
  sign: string;
  status: number | string;
}

// Utility Functions
function buildCommonParams() {
  return {
    partner_id: MUSEPAY_CONFIG.partnerId,
    sign_type: 'RSA',
    timestamp: new Date().getTime(),
    nonce: new Date().getTime(),
  };
}

function buildSignContent(params: Record<string, any>): string {
  // Remove undefined and sign parameters
  Object.keys(params).forEach((key) => {
    if (!params[key]) {
      params[key] = undefined;
    }
  });
  params.sign = undefined;
  return queryString.stringify(params, { encode: false });
}

function signRequest(content: string): string {
  console.log('Sign content:', content);

  const sig = new KJUR.crypto.Signature({
    alg: 'SHA1withRSA',
    prov: 'cryptojs/jsrsa',
    prvkeypem: MUSEPAY_CONFIG.privateKey,
  });

  sig.updateString(content);
  const signedHex = sig.sign();
  const result = hex2b64(signedHex);
  console.log('Signature result:', result);
  return result;
}

function verifySignature(content: string, signature: string): boolean {
  try {
    const sig = new KJUR.crypto.Signature({
      alg: 'SHA1withRSA',
      prov: 'cryptojs/jsrsa',
      pubkeypem: MUSEPAY_CONFIG.publicKey,
    });

    sig.updateString(content);
    return sig.verify(signature);
  } catch (error) {
    console.error('Signature verification error:', error);
    return false;
  }
}

export function generateCustomerRefId(
  userId: string,
  packageId: string,
): string {
  const timestamp = Date.now();
  return `USER:${userId}:PKG:${packageId}:REQ:${timestamp}`;
}

export function parseCustomerRefId(customerRefId: string): {
  userId: string;
  packageId: string;
  requestId: string;
} {
  const parts = customerRefId.split(':');
  return {
    userId: parts[1],
    packageId: parts[3],
    requestId: parts[5],
  };
}

// MusePay Service Class
export class MusePayService {
  /**
   * Get a deposit address for a specific crypto asset from MusePay.
   * @param currency e.g. "USDT_BSC"
   * @param customerRefId unique reference for the customer/payment
   * @param description optional description
   * @returns {Promise<{currency: string, address: string, tag: string}>}
   */
  async getDepositAddress(
    currency: string,
    customerRefId: string,
    description: string = '',
  ): Promise<{ currency: string; address: string; tag: string }> {
    const params: Record<string, any> = {
      currency,
      customer_ref_id: customerRefId,
      description,
      ...buildCommonParams(),
    };
    const signContent = buildSignContent(params);
    params.sign = signRequest(signContent);

    const response = await fetch(
      `${MUSEPAY_CONFIG.apiUrl}/order/deposit_address`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(params),
      },
    );

    if (!response.ok) {
      throw new Error(
        `MusePay deposit_address error: ${response.status} ${response.statusText}`,
      );
    }

    const result = await response.json();
    if (result.code !== 200 && result.code !== '200') {
      throw new Error(`MusePay deposit_address error: ${result.message}`);
    }
    return result.data;
  }

  async createPayment(
    userId: string,
    packageId: string,
    userEmail?: string,
  ): Promise<{
    paymentRequest: PaymentRequest;
    musePayResponse: MusePayResponse;
  }> {
    const customerRefId = generateCustomerRefId(userId, packageId);
    const packageData = getPackageById(packageId);

    if (!packageData) {
      throw new Error(`Package not found: ${packageId}`);
    }

    // 1. Store payment request in Firestore FIRST
    const paymentRequestRef = doc(collection(db, 'payment_requests'));
    const paymentRequest: PaymentRequest = {
      id: paymentRequestRef.id,
      customerRefId,
      userId,
      packageId,
      amount: packageData.price,
      currency: 'USDT_BSC',
      status: 'pending',
      packageName: packageData.name,
      packageLevel: packageData.level,
      userEmail,
      createdAt: serverTimestamp(),
      expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 hours
    };

    await setDoc(paymentRequestRef, paymentRequest);

    // 2. Call MusePay API
    const params: Record<string, any> = {
      request_id: customerRefId,
      currency: 'USDT_BSC_TEST',
      amount: packageData.price.toString(),
      payment_method: 'on_chain',
      product_name: packageData.name,
      customer_ref_id: customerRefId,
      notify_url: MUSEPAY_CONFIG.notifyUrl,
      return_url: `${process.env.NEXT_PUBLIC_SITE_URL}/payment/return`,
      ...buildCommonParams(),
      remark: `Payment for ${packageData.name}`,
    };

    // Build sign content and sign
    const signContent = buildSignContent(params);
    const signature = signRequest(signContent);
    params.sign = signature;

    // Send POST request to MusePay
    const response = await fetch(`${MUSEPAY_CONFIG.apiUrl}/order/pay`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(params),
    });

    if (!response.ok) {
      throw new Error(
        `MusePay API error: ${response.status} ${response.statusText}`,
      );
    }

    const musePayResponse: MusePayResponse = await response.json();

    if (musePayResponse.code !== '200') {
      throw new Error(`MusePay error: ${musePayResponse.message}`);
    }

    // 3. Update payment request with MusePay response
    await updateDoc(paymentRequestRef, {
      musePayOrderNo: musePayResponse.data.order_no,
      musePayRequestId: musePayResponse.data.request_id,
      receiveAddress: musePayResponse.data.receive_address,
      checkoutUrl: musePayResponse.data.checkout_url,
    });

    // Update the local object for return
    paymentRequest.musePayOrderNo = musePayResponse.data.order_no;
    paymentRequest.musePayRequestId = musePayResponse.data.request_id;
    paymentRequest.receiveAddress = musePayResponse.data.receive_address;
    paymentRequest.checkoutUrl = musePayResponse.data.checkout_url;

    return { paymentRequest, musePayResponse };
  }

  async processWebhook(webhookPayload: WebhookPayload): Promise<boolean> {
    const { customer_ref_id, order_no, status } = webhookPayload;

    // 1. Verify signature
    const signContent = buildSignContent(webhookPayload);
    const isValidSignature = verifySignature(signContent, webhookPayload.sign);

    if (!isValidSignature) {
      console.error('Invalid webhook signature');
      return false;
    }

    // 2. Parse customer_ref_id
    const { userId, packageId } = parseCustomerRefId(customer_ref_id);

    // 3. Find payment request
    const paymentRequestQuery = query(
      collection(db, 'payment_requests'),
      where('customerRefId', '==', customer_ref_id),
    );
    const paymentRequestSnapshot = await getDocs(paymentRequestQuery);

    if (paymentRequestSnapshot.empty) {
      console.error('Payment request not found:', customer_ref_id);
      return false;
    }

    const paymentRequestDoc = paymentRequestSnapshot.docs[0];
    const paymentRequestData = paymentRequestDoc.data() as PaymentRequest;

    // 4. Create/update transaction record
    const extraInfoObj = webhookPayload.extra_info
      ? JSON.parse(webhookPayload.extra_info)
      : {};
    const blockchainData = {
      txHash: extraInfoObj.txnHash || '',
      blockHeight: extraInfoObj.blockHeight || '',
      network: extraInfoObj.network || '',
      sourceAddress: extraInfoObj.sourceAddress || '',
      destinationAddress: extraInfoObj.destinationAddress || '',
      networkFee: extraInfoObj.networkFee || '0',
    };

    const transactionRef = doc(db, 'transactions', order_no);
    const transactionData = {
      partnerId: webhookPayload.partner_id,
      orderNo: order_no,
      requestId: webhookPayload.request_id,
      orderType: webhookPayload.order_type,
      productCode: webhookPayload.product_code,
      currency: webhookPayload.currency,
      orderAmount: parseFloat(webhookPayload.order_amount) || 0,
      feeAmount: parseFloat(webhookPayload.fee_amount) || 0,
      actualAmount: parseFloat(webhookPayload.actual_amount) || 0,
      status: status === 1 || status === '1' ? 'completed' : 'pending',
      reason: webhookPayload.reason || 'N/A',
      finishTime: webhookPayload.finish_time
        ? new Date(parseInt(webhookPayload.finish_time))
        : new Date(),
      updatedAt: serverTimestamp(),
      signature: webhookPayload.sign,
      blockchainData,
      blockExplorerUrl: `https://bscscan.com/tx/${blockchainData.txHash}`,

      // User and package association
      customerRefId: customer_ref_id,
      userId,
      packageId,
      packageLevel: paymentRequestData.packageLevel,
      paymentRequestId: paymentRequestDoc.id,
    };

    await setDoc(transactionRef, transactionData, { merge: true });

    // 5. Update payment request and user if completed
    if (status === 1 || status === '1') {
      await updateDoc(paymentRequestDoc.ref, {
        status: 'completed',
        completedAt: serverTimestamp(),
      });

      await this.updateUserPaymentStatus(
        userId,
        paymentRequestData,
        transactionData,
      );
    }

    return true;
  }

  private async updateUserPaymentStatus(
    userId: string,
    paymentRequest: PaymentRequest,
    transaction: any,
  ): Promise<void> {
    const userRef = doc(db, 'users', userId);
    const packageData = getPackageById(paymentRequest.packageId);

    if (!packageData) {
      console.error(
        'Package not found for payment update:',
        paymentRequest.packageId,
      );
      return;
    }

    // Calculate new expiration date
    const expirationDate = new Date(Date.now() + PAYMENT_DURATION.MILLISECONDS);

    await updateDoc(userRef, {
      // Update payment status
      'paymentStatus.hasPaid': true,
      'paymentStatus.lastPaymentDate': serverTimestamp(),
      'paymentStatus.expirationDate': expirationDate,

      // Update user level based on package
      userLevel: packageData.level,
      numericLevel: packageData.numericLevel,
      rankingLimit: packageData.rankingLimit,

      // Update payment history
      'paymentHistory.lastPaymentRequestId': paymentRequest.id,
      'paymentHistory.lastTransactionId': transaction.orderNo,
      'paymentHistory.totalPayments': increment(1),
      'paymentHistory.packageHistory': arrayUnion({
        packageId: paymentRequest.packageId,
        packageLevel: packageData.level,
        activatedAt: serverTimestamp(),
        expiresAt: expirationDate,
      }),

      // Update totals
      paymentCount: increment(1),
      totalAmountPaid: increment(parseFloat(transaction.actualAmount) || 0),
    });

    console.log(`Updated payment status for user ${userId}`);
  }

  async getPaymentRequest(
    customerRefId: string,
  ): Promise<PaymentRequest | null> {
    const paymentRequestQuery = query(
      collection(db, 'payment_requests'),
      where('customerRefId', '==', customerRefId),
    );
    const paymentRequestSnapshot = await getDocs(paymentRequestQuery);

    if (paymentRequestSnapshot.empty) {
      return null;
    }

    return paymentRequestSnapshot.docs[0].data() as PaymentRequest;
  }
}

// Export singleton instance
export const musePayService = new MusePayService();
