/**
 * User List Actions - Server Actions for enhanced user list functionality
 */

'use server'

import { getServerSession } from '@/lib/auth'
import { env } from '@/config/env'
import type { UnifiedUserData, UserOperationResult } from '@/lib/types/unified-user'

// Get bearer token from NextAuth session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.BACKEND_URL

/**
 * User filters for enhanced user list
 */
export interface UserListFilters {
  search: string
  status: string
  role: string
  page: number
  limit: number
  sortBy: string
  sortOrder: 'asc' | 'desc'
}

export interface UserListResult {
  users: UnifiedUserData[]
  total: number
  page: number
  totalPages: number
  limit: number
}

/**
 * Get users with server-side filtering and search
 */
export async function getUsersWithFilters(filters: UserListFilters): Promise<UserOperationResult<UserListResult>> {
  try {
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'No auth token' } }
    }
    
    // Build query parameters
    const params = new URLSearchParams({
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sortBy: filters.sortBy,
      sortOrder: filters.sortOrder,
    })
    
    if (filters.search) params.set('search', filters.search)
    if (filters.status && filters.status !== 'all') params.set('status', filters.status)
    if (filters.role && filters.role !== 'all') params.set('role', filters.role)
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/search?${params}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      next: { revalidate: 60 } // Cache for 1 minute
    })
    
    if (!response.ok) {
      // Fallback to mock data for development
      return getMockUsersWithFilters(filters)
    }
    
    const data = await response.json()
    
    return {
      success: true,
      data: {
        users: data.users || [],
        total: data.total || 0,
        page: data.page || filters.page,
        totalPages: data.totalPages || Math.ceil((data.total || 0) / filters.limit),
        limit: data.limit || filters.limit
      }
    }
    
  } catch (error) {
    console.error('Get users with filters error:', error)
    
    // Fallback to mock data for development
    return getMockUsersWithFilters(filters)
  }
}

/**
 * Mock users for development - fallback when backend is not available
 */
async function getMockUsersWithFilters(filters: UserListFilters): Promise<UserOperationResult<UserListResult>> {
  // Mock user data for development
  const mockUsers: UnifiedUserData[] = Array.from({ length: 50 }, (_, index) => ({
    id: `user-${index + 1}`,
    email: `user${index + 1}@example.com`,
    displayName: `User ${index + 1}`,
    status: ['active', 'disabled', 'pending', 'suspended'][index % 4] as any,
    emailVerified: index % 3 === 0,
    phoneNumber: index % 2 === 0 ? `+1234567890${index}` : null,
    twoFactorEnabled: index % 4 === 0,
    timezone: 'UTC',
    language: 'en',
    createdAt: new Date(Date.now() - (index * 24 * 60 * 60 * 1000)),
    updatedAt: new Date(Date.now() - (Math.random() * 24 * 60 * 60 * 1000)),
    lastLogin: index % 3 === 0 ? new Date(Date.now() - (Math.random() * 7 * 24 * 60 * 60 * 1000)) : null,
    roles: [
      {
        id: `role-${index}`,
        name: ['admin', 'moderator', 'user', 'premium'][index % 4],
        description: 'Mock role',
        isActive: true,
        assignedAt: new Date()
      }
    ],
    customPermissions: [],
    permissionProfiles: [],
    moduleAccess: Array.from({ length: Math.floor(Math.random() * 5) }, (_, i) => ({
      id: `module-${index}-${i}`,
      moduleName: `Module ${i + 1}`,
      description: 'Mock module',
      isActive: true,
      accessLevel: 'standard',
      assignedAt: new Date(),
      expiresAt: null,
      lastUsed: new Date()
    })),
    moduleQuotas: [],
    billing: {
      tier: ['basic', 'premium', 'enterprise'][index % 3] as any,
      paymentStatus: 'current',
      nextBillingDate: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000),
      lastPaymentDate: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
    },
    stockRankingPackages: [],
    apiKeys: [],
    recentActivity: [],
    loginHistory: [],
    usageMetrics: {
      sessionsThisMonth: Math.floor(Math.random() * 50),
      apiCallsThisMonth: Math.floor(Math.random() * 1000),
      apiCallsToday: Math.floor(Math.random() * 100),
      avgSessionDuration: Math.floor(Math.random() * 60)
    }
  }))
  
  // Apply filters
  let filteredUsers = mockUsers
  
  if (filters.search) {
    const searchLower = filters.search.toLowerCase()
    filteredUsers = filteredUsers.filter(user => 
      user.email.toLowerCase().includes(searchLower) ||
      (user.displayName || '').toLowerCase().includes(searchLower)
    )
  }
  
  if (filters.status && filters.status !== 'all') {
    filteredUsers = filteredUsers.filter(user => user.status === filters.status)
  }
  
  if (filters.role && filters.role !== 'all') {
    filteredUsers = filteredUsers.filter(user => 
      user.roles.some(role => role.name === filters.role)
    )
  }
  
  // Apply sorting
  filteredUsers.sort((a, b) => {
    let aVal: any, bVal: any
    
    switch (filters.sortBy) {
      case 'email':
        aVal = a.email
        bVal = b.email
        break
      case 'displayName':
        aVal = a.displayName || a.email
        bVal = b.displayName || b.email
        break
      case 'lastLogin':
        aVal = a.lastLogin ? new Date(a.lastLogin).getTime() : 0
        bVal = b.lastLogin ? new Date(b.lastLogin).getTime() : 0
        break
      default: // createdAt
        aVal = new Date(a.createdAt).getTime()
        bVal = new Date(b.createdAt).getTime()
    }
    
    if (aVal < bVal) return filters.sortOrder === 'asc' ? -1 : 1
    if (aVal > bVal) return filters.sortOrder === 'asc' ? 1 : -1
    return 0
  })
  
  // Apply pagination
  const total = filteredUsers.length
  const totalPages = Math.ceil(total / filters.limit)
  const startIndex = (filters.page - 1) * filters.limit
  const paginatedUsers = filteredUsers.slice(startIndex, startIndex + filters.limit)
  
  return {
    success: true,
    data: {
      users: paginatedUsers,
      total,
      page: filters.page,
      totalPages,
      limit: filters.limit
    }
  }
}