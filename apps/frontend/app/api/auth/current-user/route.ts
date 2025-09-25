/**
 * Current User API Route
 * Web3 Enterprise Authentication: Get current authenticated enterprise user data
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!accessToken) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'No enterprise session found' } },
        { status: 401 }
      );
    }

    // Get enterprise user data from enterprise API
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/v1/enterprise/auth/permissions`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'Invalid enterprise session' } },
        { status: 401 }
      );
    }

    const enterpriseData = await response.json();

    // Return enterprise user data
    return NextResponse.json({
      success: true,
      data: {
        wallet_address: enterpriseData.wallet_address,
        enterprise_tier: enterpriseData.enterprise_tier,
        permissions: enterpriseData.permissions || [],
        has_api_access: enterpriseData.has_api_access || false,
        verified_tokens_usd: enterpriseData.verified_tokens_usd || 0,
        nft_collections: enterpriseData.nft_collections || [],
        dao_memberships: enterpriseData.dao_memberships || [],
        last_updated: new Date().toISOString(),
      }
    });
  } catch (error) {
    console.error('❌ Failed to get current enterprise user:', error);
    return NextResponse.json(
      { success: false, error: { code: 'SERVER_ERROR', message: 'Failed to authenticate enterprise user' } },
      { status: 500 }
    );
  }
}