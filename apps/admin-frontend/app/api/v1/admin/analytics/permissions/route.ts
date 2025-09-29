import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken, getCurrentUser } from '@/lib/actions/server-auth';
import { UnifiedDataFetchers } from '@/lib/server/unified-data-fetchers';
import { logger } from '@/lib/logger';

export async function GET() {
  try {
    logger.action.start('getPermissionAnalytics', {});
    
    const user = await getCurrentUser();
    const token = await getBearerToken();
    
    if (!user || !token) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } },
        { status: 401 }
      );
    }

    // Check permission analytics viewing permissions
    const hasPermission = user.permissions?.includes('admin:permissions:view') || 
                         user.permissions?.includes('admin:analytics:view') ||
                         user.permissions?.includes('*');

    if (!hasPermission) {
      return NextResponse.json(
        { 
          success: false, 
          error: { code: 'FORBIDDEN', message: 'Permission analytics viewing permission required' } 
        },
        { status: 403 }
      );
    }

    // Fetch real permission analytics from backend
    const permissionAnalytics = await UnifiedDataFetchers.getPermissionAnalytics();
    
    logger.action.success('getPermissionAnalytics', { 
      totalPermissions: permissionAnalytics.total_permissions,
      healthScore: permissionAnalytics.health_score
    });
    
    return NextResponse.json({ success: true, data: permissionAnalytics });
    
  } catch (error) {
    logger.action.error('getPermissionAnalytics', error);
    return NextResponse.json(
      { 
        success: false, 
        error: { 
          code: 'UNKNOWN_ERROR', 
          message: 'Failed to fetch permission analytics' 
        } 
      },
      { status: 500 }
    );
  }
}