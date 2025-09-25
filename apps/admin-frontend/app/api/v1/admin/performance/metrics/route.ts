import { NextRequest, NextResponse } from 'next/server'
import { UnifiedAuth } from '@/lib/auth/unified-auth'

/**
 * GET /api/v1/admin/performance/metrics
 * Get performance metrics from middleware headers for monitoring
 */
export async function GET(request: NextRequest) {
  try {
    // Require admin session for accessing performance metrics
    const session = await UnifiedAuth.getSession()
    if (!session?.user || !UnifiedAuth.hasPermission(session.user, 'admin:system:view')) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 })
    }
    
    // Get performance metrics from current request headers
    const middlewarePerformance = parseFloat(request.headers.get('x-middleware-performance') || '0')
    const sessionCacheHit = request.headers.get('x-session-cache-hit') === 'true'
    const sessionValidationTime = parseFloat(request.headers.get('x-session-validation-time') || '0')
    const permissionCheckTime = parseFloat(request.headers.get('x-permission-check-time') || '0')
    
    // Calculate some basic metrics
    const totalMiddlewareTime = middlewarePerformance
    const authOverhead = sessionValidationTime + permissionCheckTime
    const authOverheadPercentage = totalMiddlewareTime > 0 ? (authOverhead / totalMiddlewareTime) * 100 : 0
    
    return NextResponse.json({
      success: true,
      data: {
        currentRequest: {
          middlewarePerformance: `${middlewarePerformance.toFixed(2)}ms`,
          sessionValidationTime: `${sessionValidationTime.toFixed(2)}ms`,
          permissionCheckTime: `${permissionCheckTime.toFixed(2)}ms`,
          sessionCacheHit: sessionCacheHit,
          authOverhead: `${authOverhead.toFixed(2)}ms`,
          authOverheadPercentage: `${authOverheadPercentage.toFixed(1)}%`
        },
        user: {
          email: session.user.email || 'unknown',
          userId: session.user.sub,
          authenticated: true
        },
        performance: {
          classification: classifyPerformance({
            middlewareTime: middlewarePerformance,
            sessionTime: sessionValidationTime,
            permissionTime: permissionCheckTime,
            cacheHit: sessionCacheHit
          }),
          recommendations: getPerformanceRecommendations({
            middlewareTime: middlewarePerformance,
            sessionTime: sessionValidationTime,
            permissionTime: permissionCheckTime,
            cacheHit: sessionCacheHit
          })
        },
        timestamp: new Date().toISOString()
      }
    })
    
  } catch (error) {
    console.error('❌ Performance metrics API error:', error)
    
    return NextResponse.json(
      { 
        success: false, 
        error: 'Failed to get performance metrics',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    )
  }
}

/**
 * Classify performance based on timing metrics
 */
function classifyPerformance(metrics: {
  middlewareTime: number
  sessionTime: number
  permissionTime: number
  cacheHit: boolean
}): string {
  const { middlewareTime, sessionTime, permissionTime, cacheHit } = metrics
  
  // Excellent: < 5ms total, cache hit
  if (middlewareTime < 5 && cacheHit) {
    return 'excellent'
  }
  
  // Good: < 20ms total, reasonable breakdown
  if (middlewareTime < 20 && sessionTime < 15 && permissionTime < 5) {
    return 'good'
  }
  
  // Fair: < 50ms total
  if (middlewareTime < 50) {
    return 'fair'
  }
  
  // Poor: >= 50ms total
  return 'poor'
}

/**
 * Get performance recommendations based on metrics
 */
function getPerformanceRecommendations(metrics: {
  middlewareTime: number
  sessionTime: number
  permissionTime: number
  cacheHit: boolean
}): string[] {
  const { middlewareTime, sessionTime, permissionTime, cacheHit } = metrics
  const recommendations: string[] = []
  
  // Cache recommendations
  if (!cacheHit && sessionTime > 10) {
    recommendations.push('Session cache miss detected - ensure cache is properly warmed')
  }
  
  // Session validation recommendations
  if (sessionTime > 20) {
    recommendations.push('Session validation is slow - check JWT verification performance')
  }
  
  // Permission check recommendations
  if (permissionTime > 5) {
    recommendations.push('Permission checks are slow - consider optimizing structured permission lookups')
  }
  
  // Overall middleware recommendations
  if (middlewareTime > 100) {
    recommendations.push('Overall middleware is very slow - investigate bottlenecks')
  } else if (middlewareTime > 50) {
    recommendations.push('Middleware performance could be improved')
  }
  
  // Cache efficiency recommendations
  if (sessionTime < 2 && cacheHit) {
    recommendations.push('Excellent cache performance - session validation is highly optimized')
  }
  
  if (recommendations.length === 0) {
    recommendations.push('Performance is optimal - no recommendations needed')
  }
  
  return recommendations
}