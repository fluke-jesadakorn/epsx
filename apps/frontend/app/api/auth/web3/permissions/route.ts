/**
 * Web3 Enterprise Permissions API Route
 * Fetches enterprise permissions, tier, and token verification data
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const wallet_address = searchParams.get('wallet_address');
    
    if (!wallet_address) {
      return NextResponse.json({
        success: false,
        error: 'Wallet address is required'
      }, { status: 400 });
    }

    console.log('🔍 Fetching Web3 permissions for wallet:', wallet_address);

    // Fetch Web3 permissions from backend
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/auth/web3/permissions?wallet_address=${encodeURIComponent(wallet_address)}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      console.warn(`Web3 permissions API returned ${response.status} for wallet ${wallet_address}`);
      
      // Return default values for failed API calls
      return NextResponse.json({
        wallet_address,
        permissions: [],
        enterprise_tier: 'Starter',
        has_api_access: false,
        verified_tokens_usd: 0,
        nft_collections: [],
        dao_memberships: [],
        error: 'Web3 permissions API unavailable'
      });
    }

    const permissionsData = await response.json();
    
    // Transform backend format to frontend format
    const permissions = permissionsData.permissions?.map((p: any) => p.permission) || [];
    
    // Determine tier from permissions
    const hasAdminPerms = permissions.includes('admin:*:*');
    const hasProfessional = permissions.some((p: string) => p.includes('professional'));
    const hasPremium = permissions.some((p: string) => p.includes('premium'));
    
    let enterprise_tier = 'Starter';
    if (hasAdminPerms) enterprise_tier = 'Enterprise';
    else if (hasProfessional) enterprise_tier = 'Professional';  
    else if (hasPremium) enterprise_tier = 'Premium';
    
    return NextResponse.json({
      wallet_address: permissionsData.wallet_address,
      permissions: permissions,
      enterprise_tier: enterprise_tier,
      has_api_access: hasAdminPerms || hasProfessional,
      verified_tokens_usd: hasAdminPerms ? 10000 : 0,
      nft_collections: [],
      dao_memberships: [],
    });
    
  } catch (error) {
    console.error('❌ Web3 permissions fetch error:', error);
    
    // Return safe defaults on error
    const { searchParams } = new URL(request.url);
    const wallet_address = searchParams.get('wallet_address');
    
    return NextResponse.json({
      wallet_address,
      permissions: [],
      enterprise_tier: 'Starter',
      has_api_access: false,
      verified_tokens_usd: 0,
      nft_collections: [],
      dao_memberships: [],
      error: 'Failed to fetch Web3 permissions data'
    });
  }
}