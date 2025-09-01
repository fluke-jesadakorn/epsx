'use client'

import React, { useState, useTransition, useEffect, useMemo } from 'react'
import { Search, Plus, Upload, RefreshCw, Edit, Trash2, Settings, Users, TrendingUp, Crown, Shield, DollarSign, CheckSquare, Square } from 'lucide-react'
import { ServerUserAPI } from '@/lib/api/admin-client'
import { searchUsersAction, getUsersList } from '@/lib/actions/users'
import { bulkGrantPermissionsAction, bulkRevokePermissionsAction, bulkAssignRolesAction, bulkApplyTemplateAction, bulkValidatePermissionsAction } from '@/lib/actions/bulk-permissions'
import { PancakePhoneTheme } from '@/design-system/pancake-phone-theme'
import CreateUserModal from '@/components/users/CreateUserModal'
import EditUserModal from '@/components/users/EditUserModal'
import DeleteUserModal from '@/components/users/DeleteUserModal'
import BulkActionsBar from '@/components/users/BulkActionsBar'
import BulkGrantPermissionsModal from '@/components/users/BulkGrantPermissionsModal'
import BulkRevokePermissionsModal from '@/components/users/BulkRevokePermissionsModal'
import BulkAssignRolesModal from '@/components/users/BulkAssignRolesModal'
import Pagination, { PaginationInfo } from '@/components/ui/Pagination'
import { useRouter } from 'next/navigation'

/**
 * PancakeSwap x Windows Phone Users Hub
 * Modern tile-based user management with DeFi aesthetics and real backend data
 */

// Windows Phone style user tile
interface UserTileProps {
  user: any
  onEdit: (user: any) => void
  onDelete: (user: any) => void
  isSelected: boolean
  onToggleSelect: (user: any) => void
  selectionMode: boolean
}

function UserTile({ user, onEdit, onDelete, isSelected, onToggleSelect, selectionMode }: UserTileProps) {
  const isActive = user.is_active
  const hasAdminPerms = user.permissions.some((p: string) => p.startsWith('admin:'))
  const isPremium = user.subscription_tier === 'premium'
  
  // PancakeSwap + Windows Phone color scheme
  const getTileColor = () => {
    if (hasAdminPerms) return 'bg-gradient-to-br from-red-600 to-red-800'
    if (isPremium) return 'bg-gradient-to-br from-yellow-500 to-orange-600'
    return 'bg-gradient-to-br from-blue-600 to-blue-800'
  }
  
  const handleEditClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onEdit(user)
  }
  
  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onDelete(user)
  }

  const handleTileClick = () => {
    if (selectionMode) {
      onToggleSelect(user)
    }
  }

  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onToggleSelect(user)
  }
  
  return (
    <div 
      className={`${getTileColor()} text-white p-5 transition-all duration-300 hover:scale-105 cursor-pointer shadow-lg relative overflow-hidden border-2 ${
        isSelected ? 'border-yellow-400 ring-2 ring-yellow-400/50' : 'border-transparent hover:border-yellow-400'
      } group`}
      onClick={handleTileClick}
    >
      {/* PancakeSwap corner accent */}
      <div className="absolute top-0 right-0 w-6 h-6 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
      
      {/* Selection checkbox - always visible in selection mode */}
      {selectionMode && (
        <div className="absolute top-3 left-3 z-10">
          <button
            onClick={handleCheckboxClick}
            className={`w-6 h-6 rounded-lg flex items-center justify-center transition-all ${
              isSelected 
                ? 'bg-yellow-400 text-black' 
                : 'bg-black/20 hover:bg-black/40 text-white'
            }`}
          >
            {isSelected ? <CheckSquare size={14} /> : <Square size={14} />}
          </button>
        </div>
      )}
      
      {/* Action buttons - appear on hover */}
      <div className="absolute top-3 right-3 flex gap-1 opacity-0 group-hover:opacity-100 transition-all duration-200 z-10">
        <button
          onClick={handleEditClick}
          className="w-7 h-7 bg-black/20 hover:bg-black/40 rounded-lg flex items-center justify-center transition-all"
          title="Edit user"
        >
          <Edit size={12} className="text-white" />
        </button>
        <button
          onClick={handleDeleteClick}
          className="w-7 h-7 bg-red-500/20 hover:bg-red-500/40 rounded-lg flex items-center justify-center transition-all"
          title="Delete user"
        >
          <Trash2 size={12} className="text-white" />
        </button>
      </div>
      
      {/* Status and role indicators */}
      <div className="flex items-center justify-between mb-4">
        <div className={`w-2 h-2 rounded-full animate-pulse ${isActive ? 'bg-green-300' : 'bg-red-300'}`}></div>
        <div className="flex items-center gap-1">
          {hasAdminPerms && <Crown size={14} className="text-yellow-400" />}
          {isPremium && <DollarSign size={14} className="text-yellow-400" />}
        </div>
      </div>

      {/* User info - Windows Phone typography */}
      <div className="mb-4">
        <h3 className="text-lg font-extralight mb-1 truncate tracking-wide">
          {user.email.split('@')[0]}
        </h3>
        <p className="text-xs opacity-75 mb-2 font-light">
          {user.email.split('@')[1]}
        </p>
        <div className="text-xs font-normal opacity-90 uppercase tracking-wider">
          {hasAdminPerms ? '🔑 admin' : isPremium ? '⭐ premium' : '👤 user'}
        </div>
      </div>

      {/* PancakeSwap-style metrics bar */}
      <div className="text-xs opacity-90 font-light">
        <div className="flex justify-between items-center">
          <span>{user.permissions.length} permissions</span>
          <span className={`px-2 py-1 rounded text-xs ${isActive ? 'bg-green-500/20' : 'bg-red-500/20'}`}>
            {isActive ? 'online' : 'offline'}
          </span>
        </div>
      </div>
      
      {/* Hover indicator */}
      <div className="absolute bottom-2 right-2 w-1 h-1 bg-yellow-400 rounded-full opacity-0 group-hover:opacity-100 transition-opacity"></div>
    </div>
  )
}

// PancakeSwap x Windows Phone Live Tile for stats
function StatsTile({ title, value, subtitle, icon: Icon, color }: any) {
  return (
    <div className={`${color} text-white p-5 transition-all duration-300 hover:scale-105 cursor-pointer shadow-lg relative overflow-hidden border-2 border-transparent hover:border-yellow-400`}>
      {/* PancakeSwap corner accent */}
      <div className="absolute top-0 right-0 w-6 h-6 bg-gradient-to-bl from-yellow-400 to-transparent opacity-60"></div>
      
      <div className="flex items-center justify-between mb-3">
        <div className="p-2 bg-white/10 rounded-lg">
          <Icon size={18} />
        </div>
        <div className="text-right">
          <div className="text-3xl font-extralight tracking-tight">{value.toLocaleString()}</div>
        </div>
      </div>
      <div className="text-xs font-normal opacity-80 mb-1 uppercase tracking-wider">{title}</div>
      <div className="text-xs opacity-75 font-light">{subtitle}</div>
    </div>
  )
}

// PancakeSwap x Windows Phone Live Tiles for stats
function LiveTiles({ stats }: { stats: any }) {
  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 mb-10">
      <StatsTile 
        title="TOTAL USERS"
        value={stats.total_users || 0}
        subtitle={`${stats.active_users || 0} active now`}
        icon={Users}
        color="bg-gradient-to-br from-yellow-500 to-orange-600"
      />
      <StatsTile 
        title="GROWTH"
        value={stats.recent_users_30_days || 0}
        subtitle="new this month"
        icon={TrendingUp}
        color="bg-gradient-to-br from-green-500 to-green-700"
      />
      <StatsTile 
        title="ADMINS"
        value={Object.values(stats.by_permissions || {}).reduce((sum: number, count) => sum + (count as number), 0) || 0}
        subtitle="privileged users"
        icon={Crown}
        color="bg-gradient-to-br from-red-600 to-red-800"
      />
      <StatsTile 
        title="SECURITY"
        value={stats.active_users || 0}
        subtitle="protected sessions"
        icon={Shield}
        color="bg-gradient-to-br from-purple-600 to-purple-800"
      />
    </div>
  )
}

interface UsersHubProps {
  initialData?: {
    users: any[]
    total: number
    stats: any
  }
}

export default function UsersHub({ initialData }: UsersHubProps) {
  const router = useRouter()
  const [isPending, startTransition] = useTransition()
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const [isEditModalOpen, setIsEditModalOpen] = useState(false)
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false)
  const [selectedUser, setSelectedUser] = useState<any>(null)
  
  // Search state - ensure consistent SSR/client initialization
  const [searchQuery, setSearchQuery] = useState('')
  const [activeFilter, setActiveFilter] = useState('all')
  const [searchResults, setSearchResults] = useState<any>(null)
  const [isSearching, setIsSearching] = useState(false)
  const [isClient, setIsClient] = useState(false)

  // Pagination state
  const [currentPage, setCurrentPage] = useState(1)
  const [itemsPerPage] = useState(20) // Fixed items per page
  const [paginatedData, setPaginatedData] = useState<any>(null)
  const [isLoadingPage, setIsLoadingPage] = useState(false)

  // Multi-select state
  const [selectedUsers, setSelectedUsers] = useState<any[]>([])
  const [selectionMode, setSelectionMode] = useState(false)
  
  // Bulk operation modals
  const [isBulkGrantModalOpen, setIsBulkGrantModalOpen] = useState(false)
  const [isBulkRevokeModalOpen, setIsBulkRevokeModalOpen] = useState(false)
  const [isBulkAssignRolesModalOpen, setIsBulkAssignRolesModalOpen] = useState(false)
  const [isBulkApplyTemplateModalOpen, setIsBulkApplyTemplateModalOpen] = useState(false)
  const [isBulkOperationLoading, setIsBulkOperationLoading] = useState(false)
  
  // Client-side hydration effect
  useEffect(() => {
    setIsClient(true)
  }, [])

  const handleEditUser = (user: any) => {
    setSelectedUser(user)
    setIsEditModalOpen(true)
  }

  const handleDeleteUser = (user: any) => {
    setSelectedUser(user)
    setIsDeleteModalOpen(true)
  }

  const handleUserDeleted = () => {
    // Refresh the page to show the updated user list
    if (searchQuery || activeFilter !== 'all') {
      // If we're in search mode, refresh search results
      performDirectSearch(searchQuery, activeFilter)
    } else {
      // Otherwise refresh the page
      startTransition(() => {
        router.refresh()
      })
    }
  }
  
  const handleUserCreated = () => {
    // Refresh search results or page data
    if (searchQuery || activeFilter !== 'all') {
      performDirectSearch(searchQuery, activeFilter)
    } else {
      startTransition(() => {
        router.refresh()
      })
    }
  }
  
  const handleUserUpdated = () => {
    // Refresh search results or page data  
    if (searchQuery || activeFilter !== 'all') {
      performDirectSearch(searchQuery, activeFilter)
    } else {
      startTransition(() => {
        router.refresh()
      })
    }
  }

  // Multi-select handlers
  const handleToggleSelect = (user: any) => {
    setSelectedUsers(prev => {
      const isSelected = prev.some(u => u.id === user.id)
      if (isSelected) {
        const newSelection = prev.filter(u => u.id !== user.id)
        // Exit selection mode if no users selected
        if (newSelection.length === 0) {
          setSelectionMode(false)
        }
        return newSelection
      } else {
        // Enter selection mode if not already active
        if (!selectionMode) {
          setSelectionMode(true)
        }
        return [...prev, user]
      }
    })
  }

  const handleDeselectAll = () => {
    setSelectedUsers([])
    setSelectionMode(false)
  }

  const handleSelectAll = () => {
    setSelectedUsers(users)
    setSelectionMode(true)
  }

  // Bulk operation handlers
  const handleBulkGrantPermissions = async (data: { userIds: string[], permissions: string[], reason?: string }) => {
    setIsBulkOperationLoading(true)
    try {
      const result = await bulkGrantPermissionsAction(data)
      if (result.success) {
        console.log('Bulk grant permissions successful:', result.data)
        // Refresh data and clear selection
        handleDeselectAll()
        if (searchQuery || activeFilter !== 'all') {
          await performDirectSearch(searchQuery, activeFilter)
        } else {
          startTransition(() => {
            router.refresh()
          })
        }
      } else {
        console.error('Bulk grant permissions failed:', result.error)
      }
    } catch (error) {
      console.error('Bulk grant permissions error:', error)
    } finally {
      setIsBulkOperationLoading(false)
    }
  }

  const handleBulkRevokePermissions = async (data: { userIds: string[], permissions: string[], reason?: string }) => {
    setIsBulkOperationLoading(true)
    try {
      const result = await bulkRevokePermissionsAction(data)
      if (result.success) {
        console.log('Bulk revoke permissions successful:', result.data)
        // Refresh data and clear selection
        handleDeselectAll()
        if (searchQuery || activeFilter !== 'all') {
          await performDirectSearch(searchQuery, activeFilter)
        } else {
          startTransition(() => {
            router.refresh()
          })
        }
      } else {
        console.error('Bulk revoke permissions failed:', result.error)
      }
    } catch (error) {
      console.error('Bulk revoke permissions error:', error)
    } finally {
      setIsBulkOperationLoading(false)
    }
  }

  const handleBulkAssignRoles = async (data: { userIds: string[], role: string, mergePermissions: boolean, reason?: string }) => {
    setIsBulkOperationLoading(true)
    try {
      const result = await bulkAssignRolesAction(data)
      if (result.success) {
        console.log('Bulk assign roles successful:', result.data)
        // Refresh data and clear selection
        handleDeselectAll()
        if (searchQuery || activeFilter !== 'all') {
          await performDirectSearch(searchQuery, activeFilter)
        } else {
          startTransition(() => {
            router.refresh()
          })
        }
      } else {
        console.error('Bulk assign roles failed:', result.error)
      }
    } catch (error) {
      console.error('Bulk assign roles error:', error)
    } finally {
      setIsBulkOperationLoading(false)
    }
  }

  const handleBulkApplyTemplate = async (userIds: string[]) => {
    setIsBulkApplyTemplateModalOpen(true)
  }

  const handleBulkValidatePermissions = async (userIds: string[]) => {
    setIsBulkOperationLoading(true)
    try {
      const result = await bulkValidatePermissionsAction({
        userIds,
        checkExpired: true,
        checkConflicting: true
      })
      if (result.success) {
        console.log('Bulk validation successful:', result.data)
        // TODO: Show validation results modal
      } else {
        console.error('Bulk validation failed:', result.error)
      }
    } catch (error) {
      console.error('Bulk validation error:', error)
    } finally {
      setIsBulkOperationLoading(false)
    }
  }


  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    console.log('🔍 handleSearchChange called with:', value)
    setSearchQuery(value)
    
    // Only trigger search after client hydration
    if (isClient) {
      // Trigger search directly instead of relying on useEffect
      setTimeout(() => {
        console.log('🔍 Direct search triggered for:', value)
        performDirectSearch(value, activeFilter)
      }, 500)
    }
  }
  
  const performDirectSearch = async (query: string, filter: string) => {
    console.log('🔍 performDirectSearch called with:', { query, filter })
    
    // Clear search results if no query and filter is 'all'
    if (!query.trim() && filter === 'all') {
      console.log('🔍 Clearing search results - no query and all filter')
      setSearchResults(null)
      return
    }

    setIsSearching(true)
    
    try {
      const searchParams: any = {
        page: 1,
        per_page: 50
      }
      
      if (query.trim()) {
        searchParams.search = query.trim()
      }
      
      if (filter !== 'all') {
        if (filter === 'active') {
          searchParams.status = 'active'
        } else if (filter === 'premium') {
          searchParams.package_tier = 'premium'
        }
        // 'admins' filter applied client-side for now
      }
      
      console.log('🔍 Making search API call with params:', searchParams)
      
      const result = await searchUsersAction(searchParams)
      
      if (result.success) {
        console.log('✅ Search successful:', result.data)
        setSearchResults(result.data)
      } else {
        console.error('❌ Search failed:', result.error)
        setSearchResults(null)
      }
      
    } catch (error) {
      console.error('❌ Search error:', error)
      setSearchResults(null)
    } finally {
      setIsSearching(false)
    }
  }

  const handleFilterClick = async (filter: string) => {
    console.log('🔍 handleFilterClick called with:', filter)
    setActiveFilter(filter)
    
    // Only trigger search after client hydration
    if (isClient) {
      await performDirectSearch(searchQuery, filter)
    }
  }

  // Clear search and return to initial data
  const clearSearch = () => {
    setSearchQuery('')
    setActiveFilter('all')
    setSearchResults(null)
  }

  // Helper to get consistent button classes
  const getFilterButtonClass = (filter: string) => {
    const baseClass = 'font-light text-lg pb-3 whitespace-nowrap transition-all border-b-2'
    
    // During SSR or before client hydration, show consistent default state
    if (!isClient) {
      if (filter === 'all') {
        return `${baseClass} text-gray-900 dark:text-white border-yellow-400`
      }
      return `${baseClass} text-gray-500 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-white`
    }
    
    // After client hydration, use active filter state
    const isActive = activeFilter === filter
    if (isActive) {
      return `${baseClass} text-gray-900 dark:text-white border-yellow-400`
    }
    return `${baseClass} text-gray-500 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-white`
  }

  // Debounced search effect - only run after client hydration
  useEffect(() => {
    // Don't run search effects until client is hydrated
    if (!isClient) return
    
    console.log('🔍 Search effect triggered with:', { searchQuery, activeFilter, isClient })
    
    // Clear search results if no query and filter is 'all'
    if (!searchQuery.trim() && activeFilter === 'all') {
      console.log('🔍 Clearing search results - no query and all filter')
      setSearchResults(null)
      return
    }

    // Debounce the search
    const timeoutId = setTimeout(async () => {
      console.log('⏰ Debounced search triggered for:', { searchQuery, activeFilter })
      setIsSearching(true)
      
      try {
        const searchParams: any = {
          page: 1,
          per_page: 50
        }
        
        if (searchQuery.trim()) {
          searchParams.search = searchQuery.trim()
        }
        
        if (activeFilter !== 'all') {
          if (activeFilter === 'active') {
            searchParams.status = 'active'
          } else if (activeFilter === 'premium') {
            searchParams.package_tier = 'premium'
          }
          // 'admins' filter applied client-side for now
        }
        
        console.log('🔍 Making search API call with params:', searchParams)
        
        const result = await searchUsersAction(searchParams)
        
        if (result.success) {
          console.log('✅ Search successful:', result.data)
          setSearchResults(result.data)
        } else {
          console.error('❌ Search failed:', result.error)
          setSearchResults(null)
        }
        
      } catch (error) {
        console.error('❌ Search error:', error)
        setSearchResults(null)
      } finally {
        setIsSearching(false)
      }
    }, 500)

    return () => clearTimeout(timeoutId)
  }, [searchQuery, activeFilter, isClient])

  // Use search results or initial data
  const displayData = useMemo(() => {
    if (searchResults) {
      let filteredUsers = searchResults.users
      
      // Client-side filtering for features not supported by backend yet
      if (activeFilter === 'admins') {
        filteredUsers = filteredUsers.filter((user: any) => 
          user.permissions && user.permissions.some((p: string) => p.startsWith('admin:'))
        )
      }
      
      return {
        users: filteredUsers,
        total: searchResults.total,
        stats: initialData?.stats || {}
      }
    }
    
    // Apply client-side filtering to initial data
    const { users = [], total = 0, stats = {} } = initialData || {}
    if (activeFilter === 'all') {
      return { users, total, stats }
    }
    
    let filteredUsers = users
    if (activeFilter === 'active') {
      filteredUsers = users.filter((user: any) => user.is_active)
    } else if (activeFilter === 'admins') {
      filteredUsers = users.filter((user: any) => 
        user.permissions && user.permissions.some((p: string) => p.startsWith('admin:'))
      )
    } else if (activeFilter === 'premium') {
      filteredUsers = users.filter((user: any) => user.package_tier === 'premium')
    }
    
    return {
      users: filteredUsers,
      total: filteredUsers.length,
      stats
    }
  }, [searchResults, initialData, activeFilter])

  const { users, total, stats } = displayData

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-black text-gray-900 dark:text-white overflow-x-hidden">
      {/* PancakeSwap x Windows Phone header */}
      <div className="px-6 pt-8 pb-6">
        <div className="flex items-center justify-between mb-4" suppressHydrationWarning>
          <div className="flex items-center gap-4">
            <div className="w-10 h-10 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-lg flex items-center justify-center">
              <Users size={20} className="text-black" />
            </div>
            <div>
              <h1 className="text-4xl font-extralight tracking-wide text-gray-900 dark:text-white">
                users
              </h1>
              <p className="text-yellow-400 font-light text-sm">
                {total.toLocaleString()} registered people
              </p>
            </div>
          </div>
          
          <div className="flex items-center gap-3">
            {/* Select All Button */}
            {users.length > 0 && (
              <button
                onClick={selectedUsers.length === users.length ? handleDeselectAll : handleSelectAll}
                className="flex items-center gap-2 px-3 py-2 bg-gray-200 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-700 transition-all text-sm"
                title={selectedUsers.length === users.length ? 'Deselect all' : 'Select all'}
              >
                {selectedUsers.length === users.length ? <CheckSquare size={14} /> : <Square size={14} />}
                <span className="hidden sm:inline">
                  {selectedUsers.length === users.length ? 'Deselect all' : 'Select all'}
                </span>
              </button>
            )}
            
            {/* Create User Button */}
            <button
              onClick={() => setIsCreateModalOpen(true)}
              className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-yellow-400 to-orange-500 text-black font-medium hover:from-yellow-500 hover:to-orange-600 transition-all disabled:opacity-50"
              disabled={isPending}
            >
              <Plus size={16} />
              <span className="hidden sm:inline">create user</span>
            </button>
          </div>
        </div>
      </div>

      {/* Live Tiles Dashboard */}
      <div className="px-6">
        <LiveTiles stats={stats} />
      </div>

      {/* Pivot Navigation with PancakeSwap accent */}
      <div className="px-6 mb-8">
        <div className="flex overflow-x-auto gap-8 border-b border-gray-300 dark:border-gray-700">
          <button 
            onClick={() => handleFilterClick('all')}
            className={getFilterButtonClass('all')}
            disabled={isSearching}
          >
            all users
          </button>
          <button 
            onClick={() => handleFilterClick('active')}
            className={getFilterButtonClass('active')}
            disabled={isSearching}
          >
            active
          </button>
          <button 
            onClick={() => handleFilterClick('admins')}
            className={getFilterButtonClass('admins')}
            disabled={isSearching}
          >
            admins
          </button>
          <button 
            onClick={() => handleFilterClick('premium')}
            className={getFilterButtonClass('premium')}
            disabled={isSearching}
          >
            premium
          </button>
        </div>
      </div>

      {/* Search with PancakeSwap styling */}
      <div className="px-6 mb-8">
        <div className="relative">
          <Search className={`absolute left-4 top-1/2 transform -translate-y-1/2 ${isSearching ? 'animate-spin text-yellow-400' : 'text-gray-500 dark:text-gray-400'}`} size={18} />
          <input 
            type="text"
            value={searchQuery}
            onChange={(e) => {
              console.log('🔍 Input onChange fired:', e.target.value)
              handleSearchChange(e)
            }}
            placeholder="search users by email or permissions..."
            className="w-full pl-12 pr-4 py-4 bg-gradient-to-r from-white to-gray-50 dark:from-gray-900 dark:to-gray-800 border border-gray-300 dark:border-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 font-light focus:border-yellow-400 focus:outline-none transition-all"
          />
          {(searchQuery || activeFilter !== 'all') && (
            <button
              onClick={clearSearch}
              className="absolute right-4 top-1/2 transform -translate-y-1/2 px-2 py-1 text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-all"
              title="Clear search"
            >
              clear
            </button>
          )}
        </div>
        
        {/* Search info */}
        {(searchResults || activeFilter !== 'all') && (
          <div className="mt-2 text-sm text-gray-600 dark:text-gray-400 font-light">
            {isSearching ? (
              <span>Searching...</span>
            ) : (
              <span>
                {searchQuery && `"${searchQuery}" • `}
                {activeFilter !== 'all' && `${activeFilter} • `}
                {total.toLocaleString()} results
              </span>
            )}
          </div>
        )}
      </div>

      {/* User Tiles Grid - PancakeSwap Live Tiles style */}
      <div className="px-6 pb-10">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          {users.map((user: any) => (
            <UserTile 
              key={user.id} 
              user={user} 
              onEdit={handleEditUser} 
              onDelete={handleDeleteUser}
              isSelected={selectedUsers.some(u => u.id === user.id)}
              onToggleSelect={handleToggleSelect}
              selectionMode={selectionMode || selectedUsers.length > 0}
            />
          ))}
        </div>
      </div>

      {/* PancakeSwap-style Pagination */}
      <div className="px-6 pb-10">
        <div className="flex items-center justify-between border-t border-gray-300 dark:border-gray-700 pt-6">
          <p className="text-gray-600 dark:text-gray-400 font-light text-sm">
            showing {users.length} of {total.toLocaleString()} users
          </p>
          <div className="flex gap-3">
            <button className="px-4 py-2 bg-gray-200 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-300 dark:hover:bg-gray-700 transition-all font-light border border-gray-400 dark:border-gray-600">
              previous
            </button>
            <button className="px-4 py-2 bg-gradient-to-r from-yellow-400 to-orange-500 text-black font-medium">
              1
            </button>
            <button className="px-4 py-2 bg-gray-200 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-300 dark:hover:bg-gray-700 transition-all font-light border border-gray-400 dark:border-gray-600">
              next
            </button>
          </div>
        </div>
      </div>

      {/* Create User Modal */}
      <CreateUserModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onUserCreated={handleUserCreated}
      />

      {/* Edit User Modal */}
      <EditUserModal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        onUserUpdated={handleUserUpdated}
        user={selectedUser}
      />

      {/* Delete User Modal */}
      <DeleteUserModal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        onUserDeleted={handleUserDeleted}
        user={selectedUser}
      />

      {/* Bulk Actions Bar */}
      <BulkActionsBar
        selectedUsers={selectedUsers}
        totalUsers={total}
        onDeselectAll={handleDeselectAll}
        onBulkGrantPermissions={() => setIsBulkGrantModalOpen(true)}
        onBulkRevokePermissions={() => setIsBulkRevokeModalOpen(true)}
        onBulkAssignRoles={() => setIsBulkAssignRolesModalOpen(true)}
        onBulkValidatePermissions={handleBulkValidatePermissions}
        onBulkApplyTemplate={handleBulkApplyTemplate}
        isLoading={isBulkOperationLoading}
      />

      {/* Bulk Grant Permissions Modal */}
      <BulkGrantPermissionsModal
        isOpen={isBulkGrantModalOpen}
        onClose={() => setIsBulkGrantModalOpen(false)}
        selectedUsers={selectedUsers}
        onConfirm={handleBulkGrantPermissions}
      />

      {/* Bulk Revoke Permissions Modal */}
      <BulkRevokePermissionsModal
        isOpen={isBulkRevokeModalOpen}
        onClose={() => setIsBulkRevokeModalOpen(false)}
        selectedUsers={selectedUsers}
        onConfirm={handleBulkRevokePermissions}
      />

      {/* Bulk Assign Roles Modal */}
      <BulkAssignRolesModal
        isOpen={isBulkAssignRolesModalOpen}
        onClose={() => setIsBulkAssignRolesModalOpen(false)}
        selectedUsers={selectedUsers}
        onConfirm={handleBulkAssignRoles}
      />
    </div>
  )
}