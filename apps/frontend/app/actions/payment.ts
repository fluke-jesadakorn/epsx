'use server';

import { createPaymentService } from '@/services/payment.service';

const paymentService = createPaymentService();

export async function getPaymentDetails(
  userId: string,
  network: string = 'TRX',
) {
  try {
    const status = await paymentService.getPaymentStatus();
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
}
