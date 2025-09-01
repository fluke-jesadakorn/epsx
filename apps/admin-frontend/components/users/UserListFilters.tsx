/**
 * User List Filters Component
 * Client-side form controls for server-side filtering
 */

'use client'

import { useRouter, useSearchParams } from 'next/navigation'
import { useState, useCallback } from 'react'
import { Search, X, SortAsc, SortDesc } from 'lucide-react'
import { Input } from '@/components/ui/input'

interface FilterState {
  search: string
  status: string
  role: string
  sortBy: string
  sortOrder: 'asc' | 'desc'
}

interface UserListFiltersProps {
  currentFilters: FilterState & {
    page: number
    limit: number
  }
}

export function UserListFilters({ currentFilters }: UserListFiltersProps) {
  const router = useRouter()
  const searchParams = useSearchParams()
  
  const [filters, setFilters] = useState<FilterState>({
    search: currentFilters.search,
    status: currentFilters.status,
    role: currentFilters.role,
    sortBy: currentFilters.sortBy,
    sortOrder: currentFilters.sortOrder
  })

  const updateURL = useCallback((newFilters: Partial<FilterState>) => {
    const params = new URLSearchParams(searchParams?.toString())
    
    Object.entries(newFilters).forEach(([key, value]) => {
      if (value && value !== 'all' && value !== '') {
        params.set(key, value)
      } else {
        params.delete(key)
      }
    })
    
    // Reset to page 1 when changing filters
    params.delete('page')
    
    const newURL = params.toString() ? `?${params.toString()}` : '/users'
    router.push(newURL)
  }, [router, searchParams])

  const handleSearch = useCallback((search: string) => {
    setFilters(prev => ({ ...prev, search }))
    updateURL({ search })
  }, [updateURL])

  const handleFilterChange = useCallback((key: keyof FilterState, value: string) => {
    const newFilters = { ...filters, [key]: value }
    setFilters(newFilters)
    updateURL({ [key]: value })
  }, [filters, updateURL])

  const clearFilters = useCallback(() => {
    const clearedFilters = {
      search: '',
      status: 'all',
      role: 'all',
      sortBy: 'createdAt',
      sortOrder: 'desc' as const
    }
    setFilters(clearedFilters)
    updateURL(clearedFilters)
  }, [updateURL])

  const hasActiveFilters = filters.search || filters.status !== 'all' || filters.role !== 'all'

  return (
    <div className="space-y-4">
      {/* Search Bar */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground z-10" />
        <Input
          type="text"
          placeholder="Search users..."
          value={filters.search}
          onChange={(e) => handleSearch(e.target.value)}
          variant="pancake"
          size="default"
          className="pl-10 pr-12"
        />
        {filters.search && (
          <button
            onClick={() => handleSearch('')}
            className="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 hover:bg-muted rounded z-10"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Filter Controls */}
      <div className="flex flex-wrap items-center gap-4">
        {/* Status Filter */}
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-muted-foreground">Status:</label>
          <select
            value={filters.status}
            onChange={(e) => handleFilterChange('status', e.target.value)}
            className="px-3 py-1 border border-muted rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            role="combobox"
            aria-label="Status filter"
          >
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
            <option value="disabled">Disabled</option>
            <option value="pending">Pending</option>
            <option value="suspended">Suspended</option>
          </select>
        </div>

        {/* Role Filter */}
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-muted-foreground">Role:</label>
          <select
            value={filters.role}
            onChange={(e) => handleFilterChange('role', e.target.value)}
            className="px-3 py-1 border border-muted rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            role="combobox"
            aria-label="Role filter"
          >
            <option value="all">All Roles</option>
            <option value="user-basic-001">Basic User</option>
            <option value="user-premium-002">Premium User</option>
            <option value="moderator-standard-003">Moderator</option>
            <option value="admin-full-004">Admin</option>
          </select>
        </div>

        {/* Sort Controls */}
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-muted-foreground">Sort:</label>
          <select
            value={filters.sortBy}
            onChange={(e) => handleFilterChange('sortBy', e.target.value)}
            className="px-3 py-1 border border-muted rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="createdAt">Created Date</option>
            <option value="lastLogin">Last Login</option>
            <option value="email">Email</option>
            <option value="displayName">Name</option>
          </select>
          <button
            onClick={() => handleFilterChange('sortOrder', filters.sortOrder === 'asc' ? 'desc' : 'asc')}
            className="p-1 border border-muted rounded hover:bg-muted transition-colors"
            title={`Sort ${filters.sortOrder === 'asc' ? 'descending' : 'ascending'}`}
          >
            {filters.sortOrder === 'asc' ? (
              <SortAsc className="h-4 w-4" />
            ) : (
              <SortDesc className="h-4 w-4" />
            )}
          </button>
        </div>

        {/* Clear Filters */}
        {hasActiveFilters && (
          <button
            onClick={clearFilters}
            className="inline-flex items-center gap-1 px-3 py-1 text-sm text-muted-foreground hover:text-foreground border border-muted rounded-md hover:bg-muted transition-colors"
            role="button"
            aria-label="Reset all filters"
          >
            <X className="h-4 w-4" />
            Clear Filters
          </button>
        )}
      </div>
    </div>
  )
}