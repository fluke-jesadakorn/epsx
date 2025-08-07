'use client'

import { useState, useEffect } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { Search, Filter } from 'lucide-react'
import { UserListFilters } from '@/lib/actions/unified-user-actions'

interface UserListClientProps {
  currentFilters: UserListFilters
}

export default function UserListClient({ currentFilters }: UserListClientProps) {
  const router = useRouter()
  const searchParams = useSearchParams()
  const [search, setSearch] = useState(currentFilters.search)
  const [role, setRole] = useState(currentFilters.role)
  const [showFilters, setShowFilters] = useState(false)

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      if (search !== currentFilters.search) {
        updateFilters({ search, page: 1 })
      }
    }, 500)

    return () => clearTimeout(timer)
  }, [search])

  const updateFilters = (newFilters: Partial<UserListFilters>) => {
    const params = new URLSearchParams(searchParams.toString())
    
    Object.entries(newFilters).forEach(([key, value]) => {
      if (value && value !== 'all' && value !== '') {
        params.set(key, value.toString())
      } else {
        params.delete(key)
      }
    })

    router.push(`/users?${params.toString()}`)
  }

  const handleRoleChange = (newRole: string) => {
    setRole(newRole)
    updateFilters({ role: newRole, page: 1 })
  }

  return (
    <div className="flex flex-col md:flex-row gap-4 mb-6">
      <div className="flex-1 relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
        <input
          type="text"
          placeholder="Search users by email..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        />
      </div>
      
      <div className="relative">
        <button 
          onClick={() => setShowFilters(!showFilters)}
          className="bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-4 py-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center gap-2"
        >
          <Filter className="h-4 w-4" />
          Filter
        </button>
        
        {showFilters && (
          <div className="absolute right-0 top-12 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4 min-w-48 z-10">
            <div className="space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Role
                </label>
                <select
                  value={role}
                  onChange={(e) => handleRoleChange(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                >
                  <option value="all">All Roles</option>
                  <option value="user">User</option>
                  <option value="moderator">Moderator</option>
                  <option value="admin">Admin</option>
                  <option value="super_admin">Super Admin</option>
                </select>
              </div>
              
              <div className="pt-2 border-t border-gray-200 dark:border-gray-600">
                <button
                  onClick={() => {
                    setSearch('')
                    setRole('all')
                    updateFilters({ search: '', role: 'all', page: 1 })
                    setShowFilters(false)
                  }}
                  className="w-full text-sm text-blue-600 hover:text-blue-800"
                >
                  Clear Filters
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}