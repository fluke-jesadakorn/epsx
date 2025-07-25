import { NextRequest, NextResponse } from 'next/server';
import { adminLogger } from '../../../../lib/logger';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const params = new URLSearchParams();
    
    // Forward query parameters
    if (searchParams.get('category')) params.set('category', searchParams.get('category')!);
    if (searchParams.get('package_tier')) params.set('package_tier', searchParams.get('package_tier')!);
    if (searchParams.get('active_only')) params.set('active_only', searchParams.get('active_only')!);
    if (searchParams.get('limit')) params.set('limit', searchParams.get('limit')!);
    if (searchParams.get('offset')) params.set('offset', searchParams.get('offset')!);

    const response = await fetch(`${BACKEND_URL}/admin/permission-profiles?${params.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        // Forward authentication cookies
        'Cookie': request.headers.get('cookie') || '',
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        { error: errorData.error || 'Failed to fetch permission profiles' },
        { status: response.status }
      );
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    adminLogger.error('Permission profile list API error', { error: error instanceof Error ? error.message : String(error) });
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}