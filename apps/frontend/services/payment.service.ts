import type { USDTDetails } from '@/types/userLevel';
import type { PaymentResponse } from '@/types/payment.d.ts';
import { db } from '@/lib/firebase';
import {
  doc,
  setDoc,
  getDoc,
  collection,
  addDoc,
  Timestamp,
} from 'firebase/firestore';
import { auth } from '@/lib/firebase';

export function createPaymentService() {
  // Function to record a payment transaction in Firestore
  const recordPayment = async (
    amount: number,
    currency: string,
    description: string = 'Subscription payment',
  ): Promise<string | null> => {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new Error('User not authenticated');
      }

      // Create a transaction record
      const transactionRef = await addDoc(collection(db, 'transactions'), {
        userId: user.uid,
        amount,
        currency,
        paymentMethod: 'Pending', // To be updated later with actual method
        transactionDate: Timestamp.now(),
        status: 'Pending',
        description,
      });

      // Update user's payment status
      const userRef = doc(db, 'users', user.uid);
      await setDoc(
        userRef,
        {
          paymentStatus: {
            hasPaid: false, // Initially false until payment is confirmed
            lastPaymentDate: Timestamp.now(),
            expirationDate: Timestamp.fromDate(
              new Date(Date.now() + 365 * 24 * 60 * 60 * 1000),
            ), // 1 year from now
          },
          userLevel: 'Pending', // To be updated upon confirmation
          email: user.email || 'N/A',
        },
        { merge: true },
      );

      return transactionRef.id;
    } catch (error) {
      console.error('Failed to record payment:', error);
      return null;
    }
  };

  // Function to confirm a payment and update status
  const confirmPayment = async (
    transactionId: string,
    paymentMethod: string,
    userLevel: string,
  ): Promise<boolean> => {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new Error('User not authenticated');
      }

      // Update transaction status
      const transactionRef = doc(db, 'transactions', transactionId);
      await setDoc(
        transactionRef,
        {
          status: 'Completed',
          paymentMethod,
        },
        { merge: true },
      );

      // Update user's payment status
      const userRef = doc(db, 'users', user.uid);
      await setDoc(
        userRef,
        {
          paymentStatus: {
            hasPaid: true,
            lastPaymentDate: Timestamp.now(),
            expirationDate: Timestamp.fromDate(
              new Date(Date.now() + 365 * 24 * 60 * 60 * 1000),
            ), // 1 year from now
          },
          userLevel,
        },
        { merge: true },
      );

      return true;
    } catch (error) {
      console.error('Failed to confirm payment:', error);
      return false;
    }
  };

  // Function to get user's payment status
  const getPaymentStatus = async (): Promise<{
    hasPaid: boolean;
    lastPaymentDate?: Date;
    expirationDate?: Date;
    userLevel?: string;
  } | null> => {
    try {
      const user = auth.currentUser;
      if (!user) {
        throw new Error('User not authenticated');
      }

      const userRef = doc(db, 'users', user.uid);
      const userDoc = await getDoc(userRef);
      if (userDoc.exists()) {
        const data = userDoc.data();
        return {
          hasPaid: data.paymentStatus?.hasPaid || false,
          lastPaymentDate: data.paymentStatus?.lastPaymentDate?.toDate(),
          expirationDate: data.paymentStatus?.expirationDate?.toDate(),
          userLevel: data.userLevel,
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to fetch payment status:', error);
      return null;
    }
  };

  // Placeholder for future QR code payment logic
  const initiateQRCodePayment = async (
    amount: number,
    currency: string,
  ): Promise<string | null> => {
    // This will be implemented in the future for QR code payment processing
    console.log('QR Code payment logic will be implemented in the future');
    return recordPayment(amount, currency);
  };

  const getPaymentDetails = async (userId: string, network: string = 'TRX') => {
    try {
      const status = await getPaymentStatus();
      if (!status) return undefined;

      // Define wallet addresses and QR codes for different USDT networks
      const paymentNetworks = {
        TRX: {
          name: 'TRC20' as const,
          address: 'TDcbvDd9aYX5cvQgCkLdvu6VbMxadDiC6F',
          qrPath: '/QRPayment/USDT_TRX.png',
        },
        BNB: {
          name: 'BEP20' as const,
          address: '0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59',
          qrPath: '/QRPayment/USDT_BNB.png',
        },
        ETH: {
          name: 'ERC20' as const,
          address: '0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59',
          qrPath: '/QRPayment/USDT_ETH.png',
        },
        ARB: {
          name: 'Arbitrum' as const,
          address: '0x1fE32489635fE7c94936cD1c5C9575aa8Ed56f59',
          qrPath: '/QRPayment/USDT_ARB.png',
        },
        TON: {
          name: 'TON' as const,
          address: 'UQDc3azM8KSuxe-Uz_l443CdLzZIIFWrFh9bh5sZ4v9CcgC5',
          tag: 'B0472569C74418F7512A',
          qrPath: '/QRPayment/USDT_TON.png',
        },
      };

      const selectedNetwork =
        paymentNetworks[network as keyof typeof paymentNetworks] ||
        paymentNetworks.TRX;

      return {
        network: selectedNetwork.name,
        walletAddress: selectedNetwork.address,
        qrCodePath: selectedNetwork.qrPath,
        tag: 'tag' in selectedNetwork ? selectedNetwork.tag : '',
        paymentStatus: {
          lastPaymentDate: status.lastPaymentDate || new Date(),
          expirationDate:
            status.expirationDate ||
            new Date(Date.now() + 365 * 24 * 60 * 60 * 1000),
          paymentMethod: 'USDT' as const,
          transactionId: 'N/A', // Transaction ID not available in status, placeholder
          amount: 0, // Amount not available in status, placeholder
        },
        userLevel: (status.userLevel || 'Basic') as any, // Cast to bypass type checking temporarily
      };
    } catch (error) {
      console.error('Failed to get payment details:', error);
      return undefined;
    }
  };

  return {
    recordPayment,
    confirmPayment,
    getPaymentStatus,
    initiateQRCodePayment,
    getPaymentDetails,
  };
}
