/**
 * Frontend Session API Route
 * Web3 Enterprise Authentication: Handles session management via wallet-based authentication
 */
import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function GET() {
  try {
    const cookieStore = await cookies();
    const accessToken = cookieStore.get('access_token')?.value;
    
    if (!accessToken) {
      return NextResponse.json({
        isAuthenticated: false,
        error: 'No active session'
      }, { status: 401 });
    }

    // Verify session with backend - use proper Web3 auth endpoint
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/auth/web3/verify-session`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      console.error('❌ Backend API error:', {
        status: response.status,
        statusText: response.statusText,
        url: `${backendUrl}/api/v1/enterprise/auth/permissions`
      });
      return NextResponse.json({
        isAuthenticated: false,
        error: 'Invalid session'
      }, { status: 401 });
    }

    // Validate response has content and is JSON
    const contentType = response.headers.get('content-type');
    if (!contentType || !contentType.includes('application/json')) {
      console.error('❌ Backend response is not JSON:', {
        contentType,
        status: response.status,
        url: `${backendUrl}/api/v1/enterprise/auth/permissions`
      });
      return NextResponse.json({
        isAuthenticated: false,
        error: 'Invalid backend response format'
      }, { status: 502 });
    }

    // Check if response has content
    const responseText = await response.text();
    if (!responseText || responseText.trim() === '') {
      console.error('❌ Backend response is empty:', {
        status: response.status,
        url: `${backendUrl}/api/v1/enterprise/auth/permissions`
      });
      return NextResponse.json({
        isAuthenticated: false,
        error: 'Empty backend response'
      }, { status: 502 });
    }

    // Parse JSON with error handling
    let enterpriseData;
    try {
      enterpriseData = JSON.parse(responseText);
    } catch (jsonError) {
      console.error('❌ JSON parsing error:', {
        error: jsonError,
        responseText: responseText.substring(0, 200), // First 200 chars for debugging
        url: `${backendUrl}/api/v1/enterprise/auth/permissions`
      });
      return NextResponse.json({
        isAuthenticated: false,
        error: 'Invalid JSON response from backend'
      }, { status: 502 });
    }
    
    // Return session data in expected format for compatibility
    return NextResponse.json({
      isAuthenticated: true,
      user: {
        wallet_address: enterpriseData.wallet_address,
        enterprise_tier: enterpriseData.enterprise_tier,
        permissions: enterpriseData.permissions || [],
        has_api_access: enterpriseData.has_api_access || false,
        verified_tokens_usd: enterpriseData.verified_tokens_usd || 0,
        nft_collections: enterpriseData.nft_collections || [],
        dao_memberships: enterpriseData.dao_memberships || [],
      },
      expiresAt: Date.now() + (24 * 60 * 60 * 1000), // 24 hours from now
    });
  } catch (error) {
    console.error('Enterprise session verification error:', error);
    return NextResponse.json({
      isAuthenticated: false,
      error: 'Session verification failed'
    }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const cookieStore = await cookies();
    
    // Clear Web3 enterprise authentication cookies
    const response = NextResponse.json({ 
      success: true, 
      message: 'Enterprise session cleared successfully' 
    });
    
    response.cookies.set('access_token', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });
    
    response.cookies.set('id_token', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });
    
    response.cookies.set('refresh_token', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });

    response.cookies.set('web3_session', '', {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: 0,
      path: '/',
    });

    console.log('✅ Enterprise session cookies cleared successfully');
    return response;
  } catch (error) {
    console.error('❌ Enterprise session clearing error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to clear enterprise session' },
      { status: 500 }
    );
  }
}