import { NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function GET(req: NextRequest) {
  try {
    const searchParams = req.nextUrl.searchParams;
    const limit = searchParams.get('limit') || '10';
    const days = searchParams.get('days') || '7';

    // Proxy to backend admin endpoint
    const res = await fetch(`${BACKEND_URL}/api/admin/web3/recent-wallets?limit=${limit}&days=${days}`, {
      headers: {
        'Content-Type': 'application/json',
        // Forward auth headers if present
        ...(req.headers.get('authorization') && {
          'authorization': req.headers.get('authorization')!
        })
      },
    });

    if (!res.ok) {
      return NextResponse.json(
        { success: false, error: `Backend error: ${res.status}` },
        { status: res.status }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (err) {
    console.error('Recent wallets proxy error:', err);
    return NextResponse.json(
      { success: false, error: 'Failed to fetch recent wallets' },
      { status: 500 }
    );
  }
}
