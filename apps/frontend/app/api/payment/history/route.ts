import { getTransactionHistory } from '@epsx/server-actions';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  try {
    const result = await getTransactionHistory();
    
    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in /api/payment/history:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get transaction history' },
      { status: 500 }
    );
  }
}