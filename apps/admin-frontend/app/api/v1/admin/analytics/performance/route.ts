import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken, getCurrentUser } from '@/lib/actions/server-auth';
import { UnifiedDataFetchers } from '@/lib/server/unified-data-fetchers';
import { logger } from '@/lib/logger';

export async function GET() {
  try {
    logger.action.start('getSystemMetrics', {});
    
    const user = await getCurrentUser();
    const token = await getBearerToken();
    
    if (!user || !token) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } },
        { status: 401 }
      );
    }

    // Check system metrics viewing permissions
    const hasPermission = user.permissions?.includes('admin:system:view') || 
                         user.permissions?.includes('admin:analytics:view') ||
                         user.permissions?.includes('*');

    if (!hasPermission) {
      return NextResponse.json(
        { 
          success: false, 
          error: { code: 'FORBIDDEN', message: 'System metrics viewing permission required' } 
        },
        { status: 403 }
      );
    }

    // Fetch real system metrics from backend
    const systemMetrics = await UnifiedDataFetchers.getSystemMetrics();
    
    logger.action.success('getSystemMetrics', { 
      apiResponseTime: systemMetrics.api_response_time,
      memoryUsage: systemMetrics.memory_usage,
      activeUsers: systemMetrics.active_users
    });
    
    return NextResponse.json({ success: true, data: systemMetrics });
    
  } catch (error) {
    logger.action.error('getSystemMetrics', error);
    return NextResponse.json(
      { 
        success: false, 
        error: { 
          code: 'UNKNOWN_ERROR', 
          message: 'Failed to fetch system metrics' 
        } 
      },
      { status: 500 }
    );
  }
}