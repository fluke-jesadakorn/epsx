import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken, getCurrentUser } from '@/lib/actions/server-auth';
import { logger } from '@/lib/logger';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export async function GET() {
  try {
    logger.action.start('getApiKeys', {});
    
    const user = await getCurrentUser();
    const token = await getBearerToken();
    
    if (!user || !token) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } },
        { status: 401 }
      );
    }

    // Check API key management permissions
    const hasPermission = user.permissions?.includes('admin:api-keys:view') || 
                         user.permissions?.includes('admin:system:view') ||
                         user.permissions?.includes('*');

    if (!hasPermission) {
      return NextResponse.json(
        { 
          success: false, 
          error: { code: 'FORBIDDEN', message: 'API key viewing permission required' } 
        },
        { status: 403 }
      );
    }

    try {
      // Try to fetch from backend
      const response = await fetch(`${BACKEND_URL}/api/v1/admin/api-keys`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        next: { revalidate: 300 } // 5-minute cache
      });

      if (response.ok) {
        const data = await response.json();
        logger.action.success('getApiKeys', { source: 'backend', count: data.keys?.length || 0 });
        return NextResponse.json({ success: true, data });
      }
    } catch (error) {
      logger.action.error('getApiKeysFromBackend', error);
    }

    // Fallback to mock data for development
    const mockApiKeys = {
      keys: [
        {
          id: 'ak_1',
          client_name: 'Frontend Application',
          key_prefix: 'epsx_pk_',
          status: 'active',
          total_requests: 15420,
          rate_limit_per_minute: 100,
          created_at: new Date(Date.now() - 86400000 * 30).toISOString(),
          last_used_at: new Date().toISOString(),
          allowed_modules: [
            { module_name: 'Analytics', access_level: 'gold' },
            { module_name: 'User Management', access_level: 'silver' }
          ],
          rate_limits: {
            'per_minute': 100,
            'per_hour': 5000,
            'per_day': 100000
          }
        },
        {
          id: 'ak_2',
          client_name: 'Mobile Application',
          key_prefix: 'epsx_pk_',
          status: 'active',
          total_requests: 8760,
          rate_limit_per_minute: 60,
          created_at: new Date(Date.now() - 86400000 * 15).toISOString(),
          last_used_at: new Date(Date.now() - 86400000).toISOString(),
          allowed_modules: [
            { module_name: 'Analytics', access_level: 'bronze' },
            { module_name: 'Basic Access', access_level: 'bronze' }
          ],
          rate_limits: {
            'per_minute': 60,
            'per_hour': 3000,
            'per_day': 50000
          }
        },
        {
          id: 'ak_3',
          client_name: 'Enterprise Dashboard',
          key_prefix: 'epsx_ek_',
          status: 'active',
          total_requests: 45230,
          rate_limit_per_minute: 500,
          created_at: new Date(Date.now() - 86400000 * 60).toISOString(),
          last_used_at: new Date(Date.now() - 3600000).toISOString(),
          allowed_modules: [
            { module_name: 'Analytics', access_level: 'enterprise' },
            { module_name: 'User Management', access_level: 'enterprise' },
            { module_name: 'System Monitoring', access_level: 'enterprise' }
          ],
          rate_limits: {
            'per_minute': 500,
            'per_hour': 25000,
            'per_day': 500000
          }
        }
      ],
      total: 3,
      active: 3,
      total_requests_today: 69410
    };
    
    logger.action.success('getApiKeys', { source: 'mock', count: mockApiKeys.keys.length });
    return NextResponse.json({ success: true, data: mockApiKeys });
    
  } catch (error) {
    logger.action.error('getApiKeys', error);
    return NextResponse.json(
      { 
        success: false, 
        error: { 
          code: 'UNKNOWN_ERROR', 
          message: 'Failed to fetch API keys' 
        } 
      },
      { status: 500 }
    );
  }
}