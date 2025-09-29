import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken, getCurrentUser } from '@/lib/actions/server-auth';
import { UnifiedDataFetchers } from '@/lib/server/unified-data-fetchers';
import { logger } from '@/lib/logger';

export async function GET() {
  try {
    logger.action.start('getUserStats', {});
    
    const user = await getCurrentUser();
    const token = await getBearerToken();
    
    if (!user || !token) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } },
        { status: 401 }
      );
    }

    // Check user management permissions
    const hasPermission = user.permissions?.includes('admin:users:view') || 
                         user.permissions?.includes('*');

    if (!hasPermission) {
      return NextResponse.json(
        { 
          success: false, 
          error: { code: 'FORBIDDEN', message: 'User management viewing permission required' } 
        },
        { status: 403 }
      );
    }

    // Fetch real user statistics from backend
    const userStats = await UnifiedDataFetchers.getUserStats();
    
    logger.action.success('getUserStats', { 
      totalUsers: userStats.total_users,
      activeUsers: userStats.active_users
    });
    
    return NextResponse.json({ success: true, data: userStats });
    
  } catch (error) {
    logger.action.error('getUserStats', error);
    return NextResponse.json(
      { 
        success: false, 
        error: { 
          code: 'UNKNOWN_ERROR', 
          message: 'Failed to fetch user statistics' 
        } 
      },
      { status: 500 }
    );
  }
}