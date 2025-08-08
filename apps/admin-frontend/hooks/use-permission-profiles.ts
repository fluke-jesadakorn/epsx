/**
 * Custom hook for managing permission profiles
 */

'use client'

import { useState, useEffect, useCallback } from 'react'
import type { 
  PermissionProfile, 
  PermissionProfileQuery,
  ListPermissionProfilesResponse,
  ApiResponse 
} from '@/lib/types/permission-profiles'
import { 
  listPermissionProfiles,
  createPermissionProfile,
  updatePermissionProfile,
  deletePermissionProfile,
  getPermissionProfile
} from '@/lib/actions/permission-profile-actions'

interface UsePermissionProfilesOptions {
  search?: string
  category?: string
  activeOnly?: boolean
  limit?: number
}

interface UsePermissionProfilesReturn {
  profiles: PermissionProfile[]
  loading: boolean
  error: string | null
  pagination: {
    page: number
    limit: number
    totalCount: number
    hasMore: boolean
  }
  refreshProfiles: () => Promise<void>
  createProfile: (data: Partial<PermissionProfile>) => Promise<ApiResponse<PermissionProfile>>
  updateProfile: (id: string, data: Partial<PermissionProfile>) => Promise<ApiResponse<PermissionProfile>>
  deleteProfile: (id: string) => Promise<ApiResponse<void>>
  getProfile: (id: string) => Promise<ApiResponse<PermissionProfile>>
}

export function usePermissionProfiles(
  options: UsePermissionProfilesOptions = {}
): UsePermissionProfilesReturn {
  const [profiles, setProfiles] = useState<PermissionProfile[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [pagination, setPagination] = useState({
    page: 1,
    limit: options.limit || 20,
    totalCount: 0,
    hasMore: false
  })

  const fetchProfiles = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      const query: PermissionProfileQuery = {
        page: pagination.page,
        limit: pagination.limit,
        name: options.search,
        category: options.category,
        activeOnly: options.activeOnly ?? true
      }

      const result = await listPermissionProfiles(query)

      if (result.success && result.data) {
        setProfiles(result.data.profiles)
        setPagination(prev => ({
          ...prev,
          totalCount: result.data!.totalCount,
          hasMore: result.data!.hasMore
        }))
      } else {
        setError(result.error?.message || 'Failed to load permission profiles')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unexpected error occurred')
    } finally {
      setLoading(false)
    }
  }, [
    pagination.page, 
    pagination.limit, 
    options.search, 
    options.category, 
    options.activeOnly
  ])

  useEffect(() => {
    fetchProfiles()
  }, [fetchProfiles])

  const refreshProfiles = useCallback(async () => {
    await fetchProfiles()
  }, [fetchProfiles])

  const createProfile = useCallback(async (data: Partial<PermissionProfile>) => {
    try {
      const result = await createPermissionProfile(data)
      if (result.success) {
        await refreshProfiles()
      }
      return result
    } catch (err) {
      return {
        success: false,
        error: {
          message: err instanceof Error ? err.message : 'Failed to create profile'
        }
      }
    }
  }, [refreshProfiles])

  const updateProfile = useCallback(async (id: string, data: Partial<PermissionProfile>) => {
    try {
      const result = await updatePermissionProfile(id, data)
      if (result.success) {
        await refreshProfiles()
      }
      return result
    } catch (err) {
      return {
        success: false,
        error: {
          message: err instanceof Error ? err.message : 'Failed to update profile'
        }
      }
    }
  }, [refreshProfiles])

  const deleteProfile = useCallback(async (id: string) => {
    try {
      const result = await deletePermissionProfile(id)
      if (result.success) {
        await refreshProfiles()
      }
      return result
    } catch (err) {
      return {
        success: false,
        error: {
          message: err instanceof Error ? err.message : 'Failed to delete profile'
        }
      }
    }
  }, [refreshProfiles])

  const getProfile = useCallback(async (id: string) => {
    try {
      return await getPermissionProfile(id)
    } catch (err) {
      return {
        success: false,
        error: {
          message: err instanceof Error ? err.message : 'Failed to get profile'
        }
      }
    }
  }, [])

  return {
    profiles,
    loading,
    error,
    pagination,
    refreshProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    getProfile
  }
}