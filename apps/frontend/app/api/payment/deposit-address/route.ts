// POST /api/payment/deposit-address
import { NextRequest, NextResponse } from 'next/server';
import { musePayService, generateCustomerRefId } from '@/lib/musepay.service';

export async function POST(req: NextRequest) {
  try {
    const { currency, userId, packageId, description } = await req.json();

    if (!currency || !userId || !packageId) {
      return NextResponse.json(
        { error: 'Missing required fields' },
        { status: 400 },
      );
    }

    // Map to testnet currency code if in test environment
    let mappedCurrency = currency;
    if (process.env.NODE_ENV !== 'production') {
      if (currency === 'USDT_BSC') mappedCurrency = 'USDT_BSC_TEST';
      if (currency === 'ETH') mappedCurrency = 'ETH_TEST';
      if (currency === 'BTC') mappedCurrency = 'BTC_TEST';
      if (currency === 'BNB') mappedCurrency = 'BNB_TEST';
    }

    const customerRefId = generateCustomerRefId(userId, packageId);
    const deposit = await musePayService.getDepositAddress(
      mappedCurrency,
      customerRefId,
      description || '',
    );

    console.debug('[DEBUG] MusePay deposit-address full response:', deposit);

    return NextResponse.json({ deposit, customerRefId });
  } catch (error: any) {
    console.error('MusePay deposit-address API error:', error);
    return NextResponse.json(
      { error: error.message || 'Internal error', details: error },
      { status: 500 },
    );
  }
}
