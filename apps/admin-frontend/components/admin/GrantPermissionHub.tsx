'use client'

import { useState, useMemo } from 'react'
import { useRouter } from 'next/navigation'
import { Search, Users, Shield, Clock, CheckCircle, AlertCircle, UserPlus } from 'lucide-react'
import { User } from '@/types/core'

interface GrantPermissionHubProps {
  users: User[]
  currentUser: any
}

export function GrantPermissionHub({ users, currentUser }: GrantPermissionHubProps) {
  const router = useRouter()
  const [searchQuery, setSearchQuery] = useState('')

  // Filter and search users
  const filteredUsers = useMemo(() => {
    if (!searchQuery.trim()) return users
    
    const query = searchQuery.toLowerCase()
    return users.filter(user => 
      user.email.toLowerCase().includes(query) ||
      user.name?.toLowerCase().includes(query) ||
      user.displayName?.toLowerCase().includes(query)
    )
  }, [users, searchQuery])

  const stats = useMemo(() => {
    return {
      totalUsers: users.length,
      activeUsers: users.filter(u => u.status === 'active').length,
      adminUsers: users.filter(u => u.role === 'admin').length,
      premiumUsers: users.filter(u => u.role === 'premium_user').length
    }
  }, [users])

  const handleUserSelect = (user: User) => {
    const params = new URLSearchParams({
      userId: user.id
    })
    router.push(`/permissions/grant?${params.toString()}`)
  }

  const getRoleBadgeColor = (role: string) => {
    switch (role) {
      case 'admin': return 'bg-gradient-to-r from-red-500 to-pink-500'
      case 'premium_user': return 'bg-gradient-to-r from-yellow-500 to-orange-500'
      default: return 'bg-gradient-to-r from-gray-400 to-gray-500'
    }
  }

  const getStatusBadgeColor = (status: string) => {
    switch (status) {
      case 'active': return 'bg-gradient-to-r from-green-400 to-green-500'
      case 'inactive': return 'bg-gradient-to-r from-yellow-400 to-yellow-500'
      case 'suspended': return 'bg-gradient-to-r from-red-400 to-red-500'
      default: return 'bg-gradient-to-r from-gray-400 to-gray-500'
    }
  }

  return (
    <div className="min-h-screen">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-7xl mx-auto">
        {/* Hero Header */}
        <div className="text-center mb-12">
          <div className="inline-flex items-center justify-center p-4 mb-6 rounded-3xl bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 shadow-2xl">
            <Shield className="h-12 w-12 text-white mr-4" />
            <div className="text-left">
              <h1 className="text-4xl font-bold text-white tracking-tight">Grant Permissions</h1>
              <p className="text-yellow-100 text-lg">Assign permissions to users securely</p>
            </div>
          </div>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto leading-relaxed">
            Select a user and grant them specific permissions with optional expiry times for enhanced security control.
          </p>
        </div>

        {/* Stats Overview */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 to-cyan-400/20 p-0.5">
            <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-blue-600 dark:text-blue-400">Total Users</p>
                  <p className="text-3xl font-bold text-gray-900 dark:text-white">{stats.totalUsers}</p>
                </div>
                <div className="p-3 bg-gradient-to-r from-blue-500 to-cyan-500 rounded-2xl">
                  <Users className="h-8 w-8 text-white" />
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 to-emerald-400/20 p-0.5">
            <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-green-600 dark:text-green-400">Active Users</p>
                  <p className="text-3xl font-bold text-gray-900 dark:text-white">{stats.activeUsers}</p>
                </div>
                <div className="p-3 bg-gradient-to-r from-green-500 to-emerald-500 rounded-2xl">
                  <CheckCircle className="h-8 w-8 text-white" />
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-red-400/20 to-pink-400/20 p-0.5">
            <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-red-600 dark:text-red-400">Admin Users</p>
                  <p className="text-3xl font-bold text-gray-900 dark:text-white">{stats.adminUsers}</p>
                </div>
                <div className="p-3 bg-gradient-to-r from-red-500 to-pink-500 rounded-2xl">
                  <Shield className="h-8 w-8 text-white" />
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-yellow-400/20 to-orange-400/20 p-0.5">
            <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-yellow-600 dark:text-yellow-400">Premium Users</p>
                  <p className="text-3xl font-bold text-gray-900 dark:text-white">{stats.premiumUsers}</p>
                </div>
                <div className="p-3 bg-gradient-to-r from-yellow-500 to-orange-500 rounded-2xl">
                  <Clock className="h-8 w-8 text-white" />
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Search Bar */}
        <div className="mb-8">
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/10 to-pink-400/10 p-0.5">
            <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-4">
              <div className="relative">
                <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 h-6 w-6 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search users by name, email..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-12 pr-4 py-4 bg-transparent border-0 focus:outline-none focus:ring-2 focus:ring-purple-500/20 rounded-2xl text-lg placeholder-gray-400"
                />
              </div>
            </div>
          </div>
        </div>

        {/* User Selection Grid */}
        <div className="mb-8">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6 flex items-center">
            <UserPlus className="h-7 w-7 mr-3 text-purple-600" />
            Select User to Grant Permissions
          </h2>
          
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {filteredUsers.map((user) => (
              <div
                key={user.id}
                onClick={() => handleUserSelect(user)}
                className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-red-400/20 p-0.5 group cursor-pointer hover:scale-105 transition-all duration-300"
              >
                <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6 h-48">
                  {/* Status Indicator */}
                  <div className="absolute top-3 left-3">
                    <div className={`w-3 h-3 rounded-full ${getStatusBadgeColor(user.status || 'active')}`}></div>
                  </div>

                  {/* Action Icons */}
                  <div className="absolute top-3 right-3 flex gap-2">
                    <div className="w-6 h-6 bg-gradient-to-r from-purple-400 to-pink-500 rounded-lg flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                      <Shield className="h-4 w-4 text-white" />
                    </div>
                  </div>

                  {/* User Info */}
                  <div className="mt-8 space-y-3">
                    <div>
                      <h3 className="font-semibold text-lg text-gray-900 dark:text-white truncate">
                        {user.displayName || user.name || 'Unknown User'}
                      </h3>
                      <p className="text-sm text-gray-600 dark:text-gray-300 truncate">{user.email}</p>
                    </div>

                    {/* Role Badge */}
                    <div className="flex items-center justify-between">
                      <span className={`px-3 py-1 rounded-full text-xs font-medium text-white ${getRoleBadgeColor(user.role)}`}>
                        {user.role.replace('_', ' ').toUpperCase()}
                      </span>
                      <span className="text-xs text-gray-500 dark:text-gray-400">
                        {user.permissions.length} permissions
                      </span>
                    </div>

                    {/* Last Login */}
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                      {user.lastLoginAt ? (
                        <>Last seen: {new Date(user.lastLoginAt).toLocaleDateString()}</>
                      ) : (
                        'Never logged in'
                      )}
                    </div>
                  </div>

                  {/* Hover Overlay */}
                  <div className="absolute inset-0 bg-gradient-to-r from-purple-600/10 to-pink-600/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 rounded-3xl flex items-center justify-center">
                    <div className="bg-white/90 dark:bg-gray-800/90 px-4 py-2 rounded-2xl shadow-xl">
                      <span className="text-sm font-medium text-gray-900 dark:text-white">Grant Permissions</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {filteredUsers.length === 0 && (
            <div className="text-center py-12">
              <AlertCircle className="h-16 w-16 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">No users found</h3>
              <p className="text-gray-600 dark:text-gray-300">
                {searchQuery ? 'Try adjusting your search criteria' : 'No users available in the system'}
              </p>
            </div>
          )}
        </div>
      </div>

    </div>
  )
}