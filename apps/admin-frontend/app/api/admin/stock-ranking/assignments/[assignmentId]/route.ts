import { NextRequest, NextResponse } from 'next/server';
import { StockRankingPackageAssignment } from '@epsx/types/src/permission_profile';

interface RouteParams {
  params: {
    assignmentId: string;
  };
}

export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    const { assignmentId } = params;

    if (!assignmentId) {
      return NextResponse.json(
        { error: 'Assignment ID is required' },
        { status: 400 }
      );
    }

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/assignments/${assignmentId}`, {
      method: 'GET',
      headers: {
        'Authorization': request.headers.get('Authorization') || '',
        'Content-Type': 'application/json',
      },
    });

    const backendResult = await response.json();

    if (!response.ok) {
      return NextResponse.json(
        { error: backendResult.error || 'Assignment not found' },
        { status: response.status }
      );
    }

    // Transform backend response to frontend format
    const assignment: StockRankingPackageAssignment = {
      id: backendResult.id,
      userId: backendResult.user_id,
      packageTier: backendResult.package_tier,
      permissionProfileId: backendResult.permission_profile_id,
      stockRankingConfig: backendResult.stock_ranking_config,
      assignedBy: backendResult.assigned_by,
      assignedAt: new Date(backendResult.assigned_at),
      expiresAt: backendResult.expires_at ? new Date(backendResult.expires_at) : undefined,
      status: backendResult.status,
      reason: backendResult.reason,
      notes: backendResult.notes,
      usageStats: backendResult.usage_stats || {
        apiCallsUsed: 0,
        lastApiCall: undefined,
        rankingsViewed: 0,
        exportsUsed: 0,
        featuresAccessed: []
      }
    };

    return NextResponse.json({ assignment });

  } catch (error) {
    console.error('Get stock ranking assignment error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function PUT(request: NextRequest, { params }: RouteParams) {
  try {
    const { assignmentId } = params;

    if (!assignmentId) {
      return NextResponse.json(
        { error: 'Assignment ID is required' },
        { status: 400 }
      );
    }

    // Extract update data from request body
    const updateData = await request.json();

    // Prepare payload for backend
    const backendPayload = {
      notes: updateData.notes,
      status: updateData.status,
      expires_at: updateData.expiresAt,
      updated_by: updateData.updatedBy || 'current_admin'
    };

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/assignments/${assignmentId}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': request.headers.get('Authorization') || '',
      },
      body: JSON.stringify(backendPayload),
    });

    const backendResult = await response.json();

    if (!response.ok) {
      return NextResponse.json(
        { error: backendResult.error || 'Failed to update assignment' },
        { status: response.status }
      );
    }

    return NextResponse.json({
      success: true,
      message: 'Assignment updated successfully',
      assignmentId,
      updatedAt: new Date().toISOString()
    });

  } catch (error) {
    console.error('Update stock ranking assignment error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function DELETE(request: NextRequest, { params }: RouteParams) {
  return NextResponse.json(
    { error: 'Use /revoke endpoint to deactivate assignments' },
    { status: 405 }
  );
}