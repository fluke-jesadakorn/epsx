/**
 * Web3 Permissions API Route
 * Fetches wallet permissions using unified client
 */
import { NextRequest, NextResponse } from 'next/server';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const wallet_address = searchParams.get('wallet_address');
    const permission = searchParams.get('permission');
    const source = searchParams.get('source');
    const limit = searchParams.get('limit');
    const offset = searchParams.get('offset');
    
    if (!wallet_address) {
      return NextResponse.json({
        success: false,
        error: 'Wallet address is required'
      }, { status: 400 });
    }

    console.log('🔍 Frontend: Fetching Web3 permissions for wallet:', wallet_address.slice(0, 8) + '...');

    // Create Web3 client for server-side permissions fetch
    const web3Client = createWeb3FrontendClient({ serverSide: true });
    
    // Get permissions using typed client with filters
    const permissionsResponse = await web3Client.getPermissions({
      wallet_address,
      permission: permission || undefined,
      source: source || undefined,
      limit: limit ? parseInt(limit) : undefined,
      offset: offset ? parseInt(offset) : undefined,
    });
    
    // Return permissions data in expected format
    return NextResponse.json(permissionsResponse);
    
  } catch (error) {
    console.error('❌ Frontend: Web3 permissions fetch error:', error);
    
    // Return safe defaults on error
    const { searchParams } = new URL(request.url);
    const wallet_address = searchParams.get('wallet_address');
    
    return NextResponse.json({
      permissions: [],
      total_count: 0,
      wallet_address,
      error: 'Failed to fetch permissions data'
    });
  }
}