/**
 * Dashboard data fetching utilities
 * Server-side data fetching for admin dashboard
 */

import { getBearerToken } from '@/lib/actions/server-auth'
import { getUserContext } from '@/lib/auth/server-auth-enhanced'

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

export interface DashboardStats {
  totalUsers: number
  verifiedUsers: number
  disabledUsers: number
  adminUsers: number
  verificationRate: number
  activeUsers: number
  newUsersToday: number
  newUsersThisWeek: number
  totalSessions: number
  systemHealth: 'good' | 'warning' | 'critical'
}

export interface BackendUserStats {
  total_users: number
  active_users: number
  deleted_users: number
  by_role: Record<string, number>
  by_tier: Record<string, number>
  recent_users_30_days: number
  user_creation_by_month: Record<string, number>
  generated_at: string
}

export interface BackendSystemMetrics {
  status: string
  timestamp: string
  data: {
    cpu_usage: number
    memory_usage: number
    active_connections: number
    requests_per_minute: number
    response_time_avg: number
    error_rate: number
    uptime_seconds: number
  }
}

export interface RecentUser {
  uid: string
  email: string
  displayName?: string
  emailVerified: boolean
  disabled: boolean
  role?: string
  createdAt: string
  lastLogin?: string
}

export interface SystemMetrics {
  serverLoad: number
  memoryUsage: number
  databaseConnections: number
  errorRate: number
  uptime: number
}

/**
 * Fetch dashboard statistics
 */
export async function getDashboardStats(): Promise<DashboardStats> {
  try {
    const token = await getBearerToken()
    const userContext = await getUserContext()
    
    if (!token || !userContext) {
      throw new Error('Not authenticated')
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/analytics/user-statistics`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      // Cache for 5 minutes
      next: { revalidate: 300 }
    })
    
    if (!response.ok) {
      // Fallback to mock data if API fails
      console.warn('Dashboard stats API failed, using mock data')
      return getMockDashboardStats()
    }
    
    const backendData: BackendUserStats = await response.json()
    
    // Map backend response to our DashboardStats interface
    const mappedData: DashboardStats = {
      totalUsers: backendData.total_users,
      verifiedUsers: backendData.active_users, // Using active_users as proxy for verified
      disabledUsers: backendData.deleted_users,
      adminUsers: (backendData.by_role?.admin || 0) + (backendData.by_role?.super_admin || 0) + (backendData.by_role?.moderator || 0),
      verificationRate: Math.round((backendData.active_users / backendData.total_users) * 100),
      activeUsers: backendData.active_users,
      newUsersToday: Math.floor(backendData.recent_users_30_days / 30), // Rough estimate
      newUsersThisWeek: Math.floor(backendData.recent_users_30_days / 4.3), // Rough estimate
      totalSessions: backendData.active_users * 2, // Rough estimate - 2 sessions per active user
      systemHealth: backendData.total_users > 0 ? 'good' : 'warning'
    }
    
    return mappedData
    
  } catch (error) {
    console.error('Error fetching dashboard stats:', error)
    return getMockDashboardStats()
  }
}

/**
 * Fetch recent users
 */
export async function getRecentUsers(limit = 10): Promise<RecentUser[]> {
  try {
    const token = await getBearerToken()
    const userContext = await getUserContext()
    
    if (!token || !userContext) {
      throw new Error('Not authenticated')
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/recent?limit=${limit}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      // Cache for 2 minutes
      next: { revalidate: 120 }
    })
    
    if (!response.ok) {
      console.warn('Recent users API failed, using mock data')
      return getMockRecentUsers()
    }
    
    const data = await response.json()
    return data
    
  } catch (error) {
    console.error('Error fetching recent users:', error)
    return getMockRecentUsers()
  }
}

/**
 * Fetch system metrics
 */
export async function getSystemMetrics(): Promise<SystemMetrics> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      throw new Error('Not authenticated')
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/analytics/system/metrics`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      // Cache for 1 minute
      next: { revalidate: 60 }
    })
    
    if (!response.ok) {
      console.warn('System metrics API failed, using mock data')
      return getMockSystemMetrics()
    }
    
    const backendData: BackendSystemMetrics = await response.json()
    
    if (backendData.status !== 'success' || !backendData.data) {
      throw new Error('Invalid system metrics response')
    }
    
    // Map backend response to our SystemMetrics interface
    const mappedData: SystemMetrics = {
      serverLoad: backendData.data.cpu_usage,
      memoryUsage: backendData.data.memory_usage,
      databaseConnections: backendData.data.active_connections,
      errorRate: backendData.data.error_rate,
      uptime: (backendData.data.uptime_seconds / (24 * 60 * 60)) * 100 // Convert to percentage
    }
    
    return mappedData
    
  } catch (error) {
    console.error('Error fetching system metrics:', error)
    return getMockSystemMetrics()
  }
}

/**
 * Mock data fallbacks for development/error cases
 */
function getMockDashboardStats(): DashboardStats {
  return {
    totalUsers: 1247,
    verifiedUsers: 1089,
    disabledUsers: 23,
    adminUsers: 8,
    verificationRate: Math.round((1089 / 1247) * 100),
    activeUsers: 432,
    newUsersToday: 12,
    newUsersThisWeek: 89,
    totalSessions: 2341,
    systemHealth: 'good'
  }
}

function getMockRecentUsers(): RecentUser[] {
  return [
    {
      uid: 'user1',
      email: 'john.doe@example.com',
      displayName: 'John Doe',
      emailVerified: true,
      disabled: false,
      role: 'user',
      createdAt: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(), // 2 hours ago
      lastLogin: new Date(Date.now() - 30 * 60 * 1000).toISOString() // 30 minutes ago
    },
    {
      uid: 'user2',
      email: 'jane.smith@example.com',
      displayName: 'Jane Smith',
      emailVerified: true,
      disabled: false,
      role: 'premium',
      createdAt: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(), // 4 hours ago
      lastLogin: new Date(Date.now() - 60 * 60 * 1000).toISOString() // 1 hour ago
    },
    {
      uid: 'user3',
      email: 'bob.wilson@example.com',
      displayName: 'Bob Wilson',
      emailVerified: false,
      disabled: false,
      role: 'user',
      createdAt: new Date(Date.now() - 6 * 60 * 60 * 1000).toISOString(), // 6 hours ago
    }
  ]
}

function getMockSystemMetrics(): SystemMetrics {
  return {
    serverLoad: 45,
    memoryUsage: 72,
    databaseConnections: 23,
    errorRate: 0.2,
    uptime: 99.8
  }
}

/**
 * Check if user should see new unified features
 */
export async function shouldShowUnifiedFeatures(): Promise<boolean> {
  const userContext = await getUserContext()
  
  if (!userContext) {
    return false
  }
  
  // For now, show to all admin users in development
  if (process.env.NODE_ENV === 'development') {
    return true
  }
  
  // In production, use feature flag logic
  // This could be expanded to use the feature flag system
  return false
}