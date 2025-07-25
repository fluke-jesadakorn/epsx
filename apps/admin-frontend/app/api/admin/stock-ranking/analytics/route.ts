import { NextRequest, NextResponse } from 'next/server';
import { PackageTier } from '@epsx/types/src/permission_profile';

interface StockRankingAnalytics {
  assignmentCounts: Record<PackageTier, number>;
  statusCounts: Record<string, number>;
  usageStats: {
    totalApiCalls: number;
    totalRankingsViewed: number;
    totalExports: number;
    activeUsers: number;
  };
  trendsData: {
    assignmentsOverTime: Array<{
      date: string;
      count: number;
      packageTier: PackageTier;
    }>;
    usageOverTime: Array<{
      date: string;
      apiCalls: number;
      rankingsViewed: number;
    }>;
  };
  topUsers: Array<{
    userId: string;
    userEmail: string;
    packageTier: PackageTier;
    apiCallsUsed: number;
    rankingsViewed: number;
  }>;
}

export async function GET(request: NextRequest) {
  try {
    // Extract query parameters
    const { searchParams } = new URL(request.url);
    const startDate = searchParams.get('startDate');
    const endDate = searchParams.get('endDate');
    const packageTier = searchParams.get('packageTier');

    // Prepare query parameters for backend
    const queryParams = new URLSearchParams({
      ...(startDate && { start_date: startDate }),
      ...(endDate && { end_date: endDate }),
      ...(packageTier && { package_tier: packageTier })
    });

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/analytics?${queryParams}`, {
      method: 'GET',
      headers: {
        'Authorization': request.headers.get('Authorization') || '',
        'Content-Type': 'application/json',
      },
    });

    const backendResult = await response.json();

    if (!response.ok) {
      return NextResponse.json(
        { error: backendResult.error || 'Failed to fetch analytics' },
        { status: response.status }
      );
    }

    // Transform backend response to frontend format
    const analytics: StockRankingAnalytics = {
      assignmentCounts: backendResult.assignment_counts || {},
      statusCounts: backendResult.status_counts || {},
      usageStats: {
        totalApiCalls: backendResult.usage_stats?.total_api_calls || 0,
        totalRankingsViewed: backendResult.usage_stats?.total_rankings_viewed || 0,
        totalExports: backendResult.usage_stats?.total_exports || 0,
        activeUsers: backendResult.usage_stats?.active_users || 0
      },
      trendsData: {
        assignmentsOverTime: backendResult.trends_data?.assignments_over_time?.map((item: any) => ({
          date: item.date,
          count: item.count,
          packageTier: item.package_tier
        })) || [],
        usageOverTime: backendResult.trends_data?.usage_over_time?.map((item: any) => ({
          date: item.date,
          apiCalls: item.api_calls,
          rankingsViewed: item.rankings_viewed
        })) || []
      },
      topUsers: backendResult.top_users?.map((user: any) => ({
        userId: user.user_id,
        userEmail: user.user_email,
        packageTier: user.package_tier,
        apiCallsUsed: user.api_calls_used,
        rankingsViewed: user.rankings_viewed
      })) || []
    };

    return NextResponse.json(analytics);

  } catch (error) {
    console.error('Get stock ranking analytics error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  return NextResponse.json(
    { error: 'Method not allowed' },
    { status: 405 }
  );
}