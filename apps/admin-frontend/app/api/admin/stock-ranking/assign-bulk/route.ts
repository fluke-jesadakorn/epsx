import { NextRequest, NextResponse } from 'next/server';
import { 
  BulkStockRankingAssignment, 
  BulkStockRankingAssignmentResult,
  StockRankingPackageConfigs,
  PackageTier 
} from '@epsx/types/src/permission_profile';

export async function POST(request: NextRequest) {
  try {
    // Extract assignment data from request
    const assignmentData: BulkStockRankingAssignment = await request.json();

    // Validate request data
    if (!assignmentData.userIds || assignmentData.userIds.length === 0) {
      return NextResponse.json(
        { error: 'No users specified for assignment' },
        { status: 400 }
      );
    }

    if (!assignmentData.packageTier) {
      return NextResponse.json(
        { error: 'Package tier is required' },
        { status: 400 }
      );
    }

    if (!assignmentData.reason) {
      return NextResponse.json(
        { error: 'Assignment reason is required' },
        { status: 400 }
      );
    }

    // Get configuration for the selected package tier
    const stockRankingConfig = StockRankingPackageConfigs.getConfigForTier(assignmentData.packageTier);

    // Prepare the request payload for backend
    const backendPayload = {
      user_ids: assignmentData.userIds,
      package_tier: assignmentData.packageTier,
      permission_profile_id: assignmentData.permissionProfileId,
      stock_ranking_config: stockRankingConfig,
      reason: assignmentData.reason,
      expires_at: assignmentData.expiresAt?.toISOString(),
      assigned_by: assignmentData.assignedBy,
      notify_users: assignmentData.notifyUsers
    };

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/assign-bulk`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': request.headers.get('Authorization') || '',
      },
      body: JSON.stringify(backendPayload),
    });

    const backendResult = await response.json();

    if (!response.ok) {
      return NextResponse.json(
        { error: backendResult.error || 'Assignment failed' },
        { status: response.status }
      );
    }

    // Transform backend response to frontend format
    const result: BulkStockRankingAssignmentResult = {
      successful: backendResult.successful || [],
      failed: backendResult.failed || [],
      summary: {
        totalRequested: assignmentData.userIds.length,
        successful: backendResult.successful?.length || 0,
        failed: backendResult.failed?.length || 0,
        packageTier: assignmentData.packageTier,
        assignmentTime: new Date()
      }
    };

    return NextResponse.json(result);

  } catch (error) {
    console.error('Bulk stock ranking assignment error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  return NextResponse.json(
    { error: 'Method not allowed' },
    { status: 405 }
  );
}