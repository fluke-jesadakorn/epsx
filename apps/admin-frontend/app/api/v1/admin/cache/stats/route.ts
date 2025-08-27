import { NextRequest, NextResponse } from 'next/server'
import { adminSessionValidator } from '@/lib/session-validator'
import { requireAdminSession } from '@/lib/session-validator'

/**
 * GET /api/v1/admin/cache/stats
 * Get session validator cache statistics for monitoring
 */
export async function GET(request: NextRequest) {
  try {
    // Require admin session for accessing cache statistics
    await requireAdminSession({
      userAgent: request.headers.get('user-agent') || undefined,
      ipAddress: request.ip || request.headers.get('x-forwarded-for') || undefined,
      path: '/api/v1/admin/cache/stats',
      method: 'GET'
    })
    
    // Get cache statistics
    const stats = adminSessionValidator.getCacheStats()
    
    return NextResponse.json({
      success: true,
      data: {
        ...stats,
        timestamp: new Date().toISOString(),
        performance: {
          hitRatio: `${(stats.hitRatio * 100).toFixed(2)}%`,
          efficiency: stats.hitRatio > 0.7 ? 'excellent' : stats.hitRatio > 0.5 ? 'good' : stats.hitRatio > 0.3 ? 'fair' : 'poor'
        }
      }
    })
    
  } catch (error) {
    console.error('❌ Cache stats API error:', error)
    
    return NextResponse.json(
      { 
        success: false, 
        error: 'Failed to get cache statistics',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    )
  }
}

/**
 * DELETE /api/v1/admin/cache/stats
 * Reset cache statistics (for testing/monitoring reset)
 */
export async function DELETE(request: NextRequest) {
  try {
    // Require admin session
    await requireAdminSession({
      userAgent: request.headers.get('user-agent') || undefined,
      ipAddress: request.ip || request.headers.get('x-forwarded-for') || undefined,
      path: '/api/v1/admin/cache/stats',
      method: 'DELETE'
    })
    
    // Reset statistics
    adminSessionValidator.resetStats()
    
    return NextResponse.json({
      success: true,
      message: 'Cache statistics reset successfully',
      timestamp: new Date().toISOString()
    })
    
  } catch (error) {
    console.error('❌ Cache stats reset error:', error)
    
    return NextResponse.json(
      { 
        success: false, 
        error: 'Failed to reset cache statistics',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    )
  }
}