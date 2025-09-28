/**
 * Web3 Challenge API Route
 * Generates SIWE challenges for wallet authentication using unified client
 */
import { NextRequest, NextResponse } from 'next/server';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { wallet_address } = body;

    if (!wallet_address) {
      return NextResponse.json(
        { error: 'Wallet address is required' },
        { status: 400 }
      );
    }

    // Create Web3 client for server-side challenge generation
    const web3Client = createWeb3FrontendClient({ serverSide: true });
    
    // Get challenge using typed client
    const challengeData = await web3Client.getChallenge(wallet_address);
    
    console.log('✅ Frontend: Challenge generated for wallet:', wallet_address.slice(0, 8) + '...');
    
    return NextResponse.json(challengeData);

  } catch (error) {
    console.error('❌ Frontend: Web3 challenge error:', error);
    return NextResponse.json(
      { error: 'Challenge generation failed' },
      { status: 500 }
    );
  }
}