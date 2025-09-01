'use client'

import React, { useState, useTransition, useEffect, useMemo } from 'react'
import { Search, Plus, Upload, RefreshCw, Edit, Trash2, Settings, Users, TrendingUp, Crown, Shield, DollarSign, CheckSquare, Square } from 'lucide-react'
import { PancakePhoneTheme } from '@/design-system/pancake-phone-theme'
import Pagination, { PaginationInfo } from '@/components/ui/Pagination'
import { useRouter } from 'next/navigation'

/**
 * PancakeSwap x Windows Phone Users Hub
 * Modern tile-based user management with DeFi aesthetics and real backend data
 */

// Windows Phone style user tile
interface UserTileProps {
  user: any
  isSelected: boolean
  onToggleSelect: (user: any) => void
  selectionMode: boolean
}

function UserTile({ user, isSelected, onToggleSelect, selectionMode }: UserTileProps) {
  const isActive = user.is_active
  
  // Parse permissions with embedded timestamps
  const parsePermission = (permission: string) => {
    const parts = permission.split(':')
    if (parts.length >= 4 && /^\d+$/.test(parts[parts.length - 1])) {
      const timestamp = parseInt(parts[parts.length - 1])
      const permissionWithoutTimestamp = parts.slice(0, -1).join(':')
      const expiresAt = new Date(timestamp * 1000)
      const isExpired = expiresAt < new Date()
      return { permission: permissionWithoutTimestamp, expiresAt, isExpired, hasTimestamp: true }
    }
    return { permission, expiresAt: null, isExpired: false, hasTimestamp: false }
  }

  const permissions = user.permissions?.map(parsePermission) || []
  const activePermissions = permissions.filter(p => !p.isExpired).length
  const hasAdminPerms = permissions.some((p: any) => p.permission.startsWith('admin:') && !p.isExpired)
  const expiredPermissions = permissions.filter(p => p.isExpired).length
  const expiringPermissions = permissions.filter(p => {
    if (!p.hasTimestamp || p.isExpired) return false
    const hoursUntilExpiry = (p.expiresAt!.getTime() - Date.now()) / (1000 * 60 * 60)
    return hoursUntilExpiry <= 24
  }).length
  
  const isPremium = user.subscription_tier === 'premium'
  
  // PancakeSwap + Windows Phone color scheme
  const getTileColor = () => {
    if (hasAdminPerms) return 'bg-gradient-to-br from-red-600 to-red-800'
    if (isPremium) return 'bg-gradient-to-br from-yellow-500 to-orange-600'
    return 'bg-gradient-to-br from-blue-600 to-blue-800'
  }
  
  const handleEditClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    window.location.href = `/users/${user.id}/edit`
  }
  
  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    window.location.href = `/users/${user.id}/delete`
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

      {/* PancakeSwap-style metrics bar with expiry indicators */}
      <div className="text-xs opacity-90 font-light">
        <div className="flex justify-between items-center mb-2">
          <div className="flex items-center gap-2">
            <span>{activePermissions} active</span>
            {expiredPermissions > 0 && (
              <div className="bg-red-400/20 text-red-300 px-1 py-0.5 rounded text-xs">
                {expiredPermissions} expired
              </div>
            )}
            {expiringPermissions > 0 && (
              <div className="bg-yellow-400/20 text-yellow-300 px-1 py-0.5 rounded text-xs">
                {expiringPermissions} expiring
              </div>
            )}
          </div>
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

// Enhanced Windows Phone Live Tile with advanced animations
function StatsTile({ title, value, subtitle, icon: Icon, color, onClick }: any) {
  const [isFlipped, setIsFlipped] = React.useState(false)
  
  React.useEffect(() => {
    const flipInterval = setInterval(() => {
      setIsFlipped(prev => !prev)
    }, 4000) // Flip every 4 seconds like Windows Phone live tiles
    
    return () => clearInterval(flipInterval)
  }, [])

  return (
    <div 
      className={`${color} text-white p-5 cursor-pointer shadow-2xl relative overflow-hidden border-2 border-transparent transition-all duration-500 transform hover:scale-110 hover:rotate-1 hover:border-yellow-400 hover:shadow-yellow-400/25 group perspective-1000`}
      onClick={onClick}
      style={{ perspective: '1000px' }}
    >
      {/* Enhanced PancakeSwap corner accent with animation */}
      <div className="absolute top-0 right-0 w-8 h-8 bg-gradient-to-bl from-yellow-400 via-yellow-300 to-transparent opacity-70 group-hover:opacity-100 transition-opacity duration-300"></div>
      
      {/* Windows Phone live tile flip animation */}
      <div className={`transition-transform duration-700 transform-style-preserve-3d ${isFlipped ? 'rotate-y-180' : ''}`}>
        {/* Front face */}
        <div className="backface-hidden">
          <div className="flex items-center justify-between mb-4">
            <div className="p-3 bg-white/15 rounded-xl backdrop-blur-sm group-hover:bg-white/25 transition-all duration-300">
              <Icon size={20} className="group-hover:scale-110 transition-transform duration-300" />
            </div>
            <div className="text-right">
              <div className="text-3xl font-extralight tracking-tight group-hover:text-4xl transition-all duration-300 counter-animation">
                {value.toLocaleString()}
              </div>
            </div>
          </div>
          
          <div className="space-y-1">
            <div className="text-xs font-medium opacity-90 uppercase tracking-widest group-hover:opacity-100 transition-opacity">
              {title}
            </div>
            <div className="text-xs opacity-75 font-light group-hover:opacity-90 transition-opacity">
              {subtitle}
            </div>
          </div>
        </div>
        
        {/* Back face with additional info */}
        <div className="absolute inset-0 backface-hidden rotate-y-180 p-5">
          <div className="h-full flex flex-col justify-center items-center text-center">
            <div className="w-12 h-12 bg-white/20 rounded-xl flex items-center justify-center mb-3">
              <Icon size={24} />
            </div>
            <div className="text-lg font-extralight mb-2">{title}</div>
            <div className="text-sm opacity-90">{subtitle}</div>
            <div className="text-xs opacity-75 mt-2">tap for details</div>
          </div>
        </div>
      </div>
      
      {/* Windows Phone accent line */}
      <div className="absolute left-0 top-0 h-full w-1 bg-gradient-to-b from-yellow-400 via-orange-400 to-transparent opacity-60 group-hover:opacity-100 transition-opacity"></div>
      
      {/* Hover glow effect */}
      <div className="absolute inset-0 bg-gradient-to-br from-white/0 via-white/5 to-white/0 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
      
      {/* Bottom right accent dot */}
      <div className="absolute bottom-2 right-2 w-2 h-2 bg-yellow-400 rounded-full opacity-0 group-hover:opacity-100 transition-all duration-300 group-hover:scale-125"></div>
    </div>
  )
}

// Enhanced PancakeSwap x Windows Phone Live Tiles Dashboard
function LiveTiles({ stats }: { stats: any }) {
  const router = useRouter()
  
  const handleTileClick = (section: string) => {
    switch (section) {
      case 'users':
        router.push('/users?filter=all')
        break
      case 'growth':
        router.push('/users?filter=active')
        break
      case 'admins':
        router.push('/users?filter=admins')
        break
      case 'security':
        router.push('/analytics')
        break
      default:
        break
    }
  }

  // Calculate admin permissions count more accurately
  const adminPermissionsCount = Object.entries(stats.by_permissions || {})
    .filter(([permission]) => permission.startsWith('admin:'))
    .reduce((sum, [, count]) => sum + (count as number), 0)

  return (
    <div className="mb-10">
      {/* Main stats tiles - large and prominent */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <StatsTile 
          title="TOTAL USERS"
          value={stats.total_users || 0}
          subtitle={`${stats.active_users || 0} active • ${Math.round((stats.active_users || 0) / (stats.total_users || 1) * 100)}% online`}
          icon={Users}
          color="bg-gradient-to-br from-yellow-500 via-yellow-600 to-orange-600"
          onClick={() => handleTileClick('users')}
        />
        
        <StatsTile 
          title="NEW GROWTH"
          value={stats.recent_users_30_days || 0}
          subtitle={`${stats.recent_users_30_days || 0} this month • ${Math.round((stats.recent_users_30_days || 0) / 30)} daily avg`}
          icon={TrendingUp}
          color="bg-gradient-to-br from-green-500 via-green-600 to-emerald-700"
          onClick={() => handleTileClick('growth')}
        />
        
        <StatsTile 
          title="ADMIN ACCESS"
          value={adminPermissionsCount}
          subtitle={`${adminPermissionsCount} privileged • ${Math.round(adminPermissionsCount / (stats.total_users || 1) * 100)}% admin ratio`}
          icon={Crown}
          color="bg-gradient-to-br from-red-600 via-red-700 to-red-800"
          onClick={() => handleTileClick('admins')}
        />
        
        <StatsTile 
          title="SECURITY"
          value={stats.active_users || 0}
          subtitle={`${stats.active_users || 0} protected sessions • 100% secure`}
          icon={Shield}
          color="bg-gradient-to-br from-purple-600 via-purple-700 to-indigo-800"
          onClick={() => handleTileClick('security')}
        />
      </div>
      
      {/* Secondary metrics - smaller tiles for additional insights */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        {Object.entries(stats.by_tier || {}).map(([tier, count]) => (
          <div 
            key={tier}
            className="bg-gradient-to-br from-gray-700 via-gray-800 to-gray-900 text-white p-4 transition-all duration-300 hover:scale-105 cursor-pointer shadow-lg relative overflow-hidden group"
            onClick={() => router.push(`/users?status=${tier}`)}
          >
            {/* Mini PancakeSwap accent */}
            <div className="absolute top-0 right-0 w-3 h-3 bg-gradient-to-bl from-yellow-400 to-transparent opacity-50 group-hover:opacity-100 transition-opacity"></div>
            
            <div className="text-center">
              <div className="text-lg font-extralight">{count as number}</div>
              <div className="text-xs uppercase tracking-wider opacity-90">{tier}</div>
            </div>
            
            {/* Windows Phone accent dot */}
            <div className="absolute bottom-1 right-1 w-1 h-1 bg-yellow-400 rounded-full opacity-0 group-hover:opacity-100 transition-opacity"></div>
          </div>
        ))}
      </div>
    </div>
  )
}

interface UsersHubProps {
  initialData?: {
    users: any[]
    total: number
    page?: number
    totalPages?: number
    stats: any
  }
  searchParams?: {
    page?: string
    search?: string
    filter?: string
    limit?: string
  }
}

export default function UsersHub({ initialData, searchParams }: UsersHubProps) {
  const router = useRouter()
  const [isPending, startTransition] = useTransition()
  
  // Get current values from URL search params
  const currentPage = parseInt(searchParams?.page || '1', 10)
  const searchQuery = searchParams?.search || ''
  const activeFilter = searchParams?.filter || 'all'
  const itemsPerPage = parseInt(searchParams?.limit || '20', 10)
  
  // Client-side state for UI interactions only
  const [isSearching, setIsSearching] = useState(false)
  const [isClient, setIsClient] = useState(false)

  // Multi-select state
  const [selectedUsers, setSelectedUsers] = useState<any[]>([])
  const [selectionMode, setSelectionMode] = useState(false)
  
  
  // URL navigation helpers
  const updateURL = (updates: Record<string, string | number | undefined>) => {
    const params = new URLSearchParams(window.location.search)
    
    Object.entries(updates).forEach(([key, value]) => {
      if (value === undefined || value === null || value === '' || value === 'all' || (key === 'page' && value === 1)) {
        params.delete(key)
      } else {
        params.set(key, value.toString())
      }
    })
    
    const newUrl = `/users${params.toString() ? `?${params.toString()}` : ''}`
    router.push(newUrl)
  }

  // Client-side hydration effect
  useEffect(() => {
    setIsClient(true)
  }, [])

  // Page navigation handlers (replace modal handlers)
  const handleCreateUser = () => {
    router.push('/users/create')
  }

  // Simple pagination handler using URL navigation
  const handlePageChange = (page: number) => {
    updateURL({ page })
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

  // Bulk operation navigation handlers (replace modal handlers)
  const handleBulkOperations = () => {
    const userIds = selectedUsers.map(u => u.id)
    if (userIds.length > 0) {
      router.push(`/users/bulk?users=${userIds.join(',')}`)
    }
  }


  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    
    // Debounce search updates
    if (isClient) {
      setTimeout(() => {
        updateURL({ search: value, page: 1 }) // Reset to page 1 when searching
      }, 500)
    }
  }
  
  const handleFilterClick = (filter: string) => {
    updateURL({ filter, page: 1 }) // Reset to page 1 when filtering
  }

  // Clear search and return to initial data
  const clearSearch = () => {
    updateURL({ search: undefined, filter: undefined, page: undefined })
  }

  // Helper to get consistent button classes
  const getFilterButtonClass = (filter: string) => {
    const baseClass = 'font-light text-lg pb-3 whitespace-nowrap transition-all border-b-2'
    
    const isActive = activeFilter === filter
    if (isActive) {
      return `${baseClass} text-gray-900 dark:text-white border-yellow-400`
    }
    return `${baseClass} text-gray-500 dark:text-gray-400 border-transparent hover:text-gray-900 dark:hover:text-white`
  }

  // Use server-fetched data directly
  const { users = [], total = 0, page = 1, totalPages = 1, stats = {} } = initialData || {}

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
              onClick={handleCreateUser}
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
          >
            all users
          </button>
          <button 
            onClick={() => handleFilterClick('active')}
            className={getFilterButtonClass('active')}
          >
            active
          </button>
          <button 
            onClick={() => handleFilterClick('admins')}
            className={getFilterButtonClass('admins')}
          >
            admins
          </button>
          <button 
            onClick={() => handleFilterClick('premium')}
            className={getFilterButtonClass('premium')}
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
            defaultValue={searchQuery}
            onChange={handleSearchChange}
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
        {(searchQuery || activeFilter !== 'all') && (
          <div className="mt-2 text-sm text-gray-600 dark:text-gray-400 font-light">
            <span>
              {searchQuery && `"${searchQuery}" • `}
              {activeFilter !== 'all' && `${activeFilter} • `}
              {total.toLocaleString()} results
            </span>
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
              isSelected={selectedUsers.some(u => u.id === user.id)}
              onToggleSelect={handleToggleSelect}
              selectionMode={selectionMode || selectedUsers.length > 0}
            />
          ))}
        </div>
      </div>

      {/* Server-side Pagination */}
      <div className="px-6 pb-6">
        <div className="mb-4">
          <PaginationInfo
            currentPage={currentPage}
            totalPages={totalPages}
            totalItems={total}
            itemsPerPage={itemsPerPage}
            itemName="users"
          />
        </div>
        
        <Pagination
          currentPage={currentPage}
          totalPages={totalPages}
          onPageChange={handlePageChange}
          disabled={isPending}
        />
      </div>

      {/* Bulk Actions Bar - Updated for page navigation */}
      {selectedUsers.length > 0 && (
        <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50">
          <div className="bg-[#FFC107] text-black px-6 py-3 rounded-lg shadow-lg flex items-center gap-4">
            <span className="font-medium">
              {selectedUsers.length} user{selectedUsers.length !== 1 ? 's' : ''} selected
            </span>
            <button
              onClick={handleBulkOperations}
              className="px-4 py-2 bg-black text-white rounded hover:bg-gray-800 transition-colors text-sm font-medium"
            >
              Bulk Operations
            </button>
            <button
              onClick={handleDeselectAll}
              className="px-3 py-1 text-black/70 hover:text-black transition-colors text-sm"
            >
              Clear
            </button>
          </div>
        </div>
      )}
    </div>
  )
}