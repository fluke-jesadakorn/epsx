import { NextRequest, NextResponse } from 'next/server';
import { StockRankingPackageAssignment } from '@epsx/types/src/permission_profile';

export async function GET(request: NextRequest) {
  try {
    // Extract query parameters
    const { searchParams } = new URL(request.url);
    const packageTier = searchParams.get('packageTier');
    const status = searchParams.get('status');
    const userId = searchParams.get('userId');
    const limit = searchParams.get('limit') || '50';
    const offset = searchParams.get('offset') || '0';

    // Prepare query parameters for backend
    const queryParams = new URLSearchParams({
      limit,
      offset,
      ...(packageTier && { package_tier: packageTier }),
      ...(status && { status }),
      ...(userId && { user_id: userId })
    });

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/assignments?${queryParams}`, {
      method: 'GET',
      headers: {
        'Authorization': request.headers.get('Authorization') || '',
        'Content-Type': 'application/json',
      },
    });

    const backendResult = await response.json();

    if (!response.ok) {
      return NextResponse.json(
        { error: backendResult.error || 'Failed to fetch assignments' },
        { status: response.status }
      );
    }

    // Transform backend response to frontend format
    const assignments: StockRankingPackageAssignment[] = backendResult.assignments?.map((assignment: any) => ({
      id: assignment.id,
      userId: assignment.user_id,
      packageTier: assignment.package_tier,
      permissionProfileId: assignment.permission_profile_id,
      stockRankingConfig: assignment.stock_ranking_config,
      assignedBy: assignment.assigned_by,
      assignedAt: new Date(assignment.assigned_at),
      expiresAt: assignment.expires_at ? new Date(assignment.expires_at) : undefined,
      status: assignment.status,
      reason: assignment.reason,
      notes: assignment.notes,
      usageStats: assignment.usage_stats || {
        apiCallsUsed: 0,
        lastApiCall: undefined,
        rankingsViewed: 0,
        exportsUsed: 0,
        featuresAccessed: []
      },
      user: assignment.user ? {
        id: assignment.user.id,
        email: assignment.user.email,
        name: assignment.user.name
      } : undefined
    })) || [];

    return NextResponse.json({
      assignments,
      total: backendResult.total || assignments.length,
      limit: parseInt(limit),
      offset: parseInt(offset)
    });

  } catch (error) {
    console.error('Get stock ranking assignments error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  return NextResponse.json(
    { error: 'Use /assign-bulk endpoint for creating assignments' },
    { status: 405 }
  );
}