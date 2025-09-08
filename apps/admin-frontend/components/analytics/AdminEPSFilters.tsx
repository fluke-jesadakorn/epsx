'use client'

import { useState, useCallback } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { Search, Filter, X, ChevronDown } from 'lucide-react'

interface EPSQueryParams {
  page: number
  limit: number
  country?: string
  sector?: string
  sort_by?: string
  min_eps?: number
  min_growth?: number
  search?: string
}

interface AdminEPSFiltersProps {
  currentParams: EPSQueryParams
}

// Mock filter options (in real implementation, these would come from API)
const COUNTRIES = [
  { value: '', label: 'All Countries' },
  { value: 'US', label: 'United States' },
  { value: 'CA', label: 'Canada' },
  { value: 'DE', label: 'Germany' },
  { value: 'JP', label: 'Japan' },
  { value: 'GB', label: 'United Kingdom' }
]

const SECTORS = [
  { value: '', label: 'All Sectors' },
  { value: 'Technology', label: 'Technology' },
  { value: 'Healthcare', label: 'Healthcare' },
  { value: 'Financial Services', label: 'Financial Services' },
  { value: 'Consumer Cyclical', label: 'Consumer Cyclical' },
  { value: 'Communication Services', label: 'Communication Services' },
  { value: 'Industrials', label: 'Industrials' },
  { value: 'Consumer Defensive', label: 'Consumer Defensive' },
  { value: 'Energy', label: 'Energy' },
  { value: 'Utilities', label: 'Utilities' }
]

const SORT_OPTIONS = [
  { value: 'growth_factor', label: '📈 EPS Growth' },
  { value: 'price_growth', label: '💰 Price Growth' },
  { value: 'rank', label: '🏆 Rank' },
  { value: 'symbol', label: '📊 Symbol' },
  { value: 'user_views', label: '👁️ User Views' },
  { value: 'user_watchlist', label: '⭐ Watchlist Count' },
  { value: 'admin_priority', label: '🔴 Admin Priority' }
]

const ADMIN_PRIORITY_OPTIONS = [
  { value: '', label: 'All Priorities' },
  { value: 'high', label: '🔴 High Priority' },
  { value: 'medium', label: '🟡 Medium Priority' },
  { value: 'low', label: '🟢 Low Priority' }
]

const ADMIN_STATUS_OPTIONS = [
  { value: '', label: 'All Statuses' },
  { value: 'alerts', label: '⚠️ Has Alerts' },
  { value: 'clean', label: '✅ Clean Status' },
  { value: 'high_engagement', label: '🔥 High User Engagement' }
]

export default function AdminEPSFilters({ currentParams }: AdminEPSFiltersProps) {
  const router = useRouter()
  const searchParams = useSearchParams()
  
  const [showAdvanced, setShowAdvanced] = useState(false)
  const [searchQuery, setSearchQuery] = useState(currentParams.search || '')
  const [minEPS, setMinEPS] = useState(currentParams.min_eps?.toString() || '')
  const [minGrowth, setMinGrowth] = useState(currentParams.min_growth?.toString() || '')
  
  // Debounced search function
  const updateFilters = useCallback((newParams: Partial<EPSQueryParams>) => {
    const params = new URLSearchParams(searchParams.toString())
    
    // Reset to page 1 when filters change
    params.set('page', '1')
    
    // Update each parameter
    Object.entries(newParams).forEach(([key, value]) => {
      if (value === undefined || value === '' || value === null) {
        params.delete(key)
      } else {
        params.set(key, String(value))
      }
    })
    
    // Navigate to updated URL
    router.push(`?${params.toString()}`)
  }, [router, searchParams])

  const handleSearchSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault()
    updateFilters({ search: searchQuery })
  }, [searchQuery, updateFilters])

  const handleSelectChange = useCallback((key: string, value: string) => {
    updateFilters({ [key]: value })
  }, [updateFilters])

  const handleRangeSubmit = useCallback(() => {
    updateFilters({ 
      min_eps: minEPS ? parseFloat(minEPS) : undefined,
      min_growth: minGrowth ? parseFloat(minGrowth) : undefined
    })
  }, [minEPS, minGrowth, updateFilters])

  const clearAllFilters = useCallback(() => {
    setSearchQuery('')
    setMinEPS('')
    setMinGrowth('')
    router.push('?page=1')
  }, [router])

  const hasActiveFilters = !!(
    currentParams.search || 
    currentParams.country || 
    currentParams.sector || 
    currentParams.min_eps || 
    currentParams.min_growth ||
    currentParams.sort_by !== 'growth_factor'
  )

  return (
    <div className="mb-8 space-y-4">
      {/* Main filter bar */}
      <div className="rounded-3xl border border-pink-200/50 bg-gradient-to-r from-pink-50 via-orange-50 to-yellow-50 p-6 shadow-xl dark:from-pink-900/20 dark:via-orange-900/20 dark:to-yellow-900/20">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:gap-6">
          {/* Search */}
          <form onSubmit={handleSearchSubmit} className="flex flex-1 items-center gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search stocks... (e.g., AAPL, NVDA)"
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-3 pl-10 pr-4 text-sm backdrop-blur-sm transition-all focus:border-orange-400 focus:outline-none focus:ring-2 focus:ring-orange-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              />
            </div>
            <button
              type="submit"
              className="rounded-2xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 text-sm font-semibold text-white shadow-lg transition-all hover:from-orange-600 hover:to-yellow-600 hover:scale-105"
            >
              🔍 Search
            </button>
          </form>

          {/* Advanced filters toggle */}
          <button
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="flex items-center gap-2 rounded-2xl bg-white/60 px-4 py-3 text-sm font-medium text-gray-700 backdrop-blur-sm transition-all hover:bg-white/80 dark:bg-gray-800/60 dark:text-gray-300 dark:hover:bg-gray-800/80"
          >
            <Filter className="h-4 w-4" />
            Advanced Filters
            <ChevronDown className={`h-4 w-4 transition-transform ${showAdvanced ? 'rotate-180' : ''}`} />
          </button>

          {/* Clear filters */}
          {hasActiveFilters && (
            <button
              onClick={clearAllFilters}
              className="flex items-center gap-2 rounded-2xl bg-red-100 px-4 py-3 text-sm font-medium text-red-700 transition-all hover:bg-red-200 dark:bg-red-900/20 dark:text-red-300 dark:hover:bg-red-900/40"
            >
              <X className="h-4 w-4" />
              Clear All
            </button>
          )}
        </div>
      </div>

      {/* Advanced filters */}
      {showAdvanced && (
        <div className="rounded-3xl border border-blue-200/50 bg-gradient-to-r from-blue-50 via-indigo-50 to-purple-50 p-6 shadow-xl dark:from-blue-900/20 dark:via-indigo-900/20 dark:to-purple-900/20">
          <div className="mb-4 flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-2xl bg-gradient-to-br from-blue-500 to-indigo-600 text-lg text-white shadow-lg">
              🎛️
            </div>
            <h3 className="text-lg font-bold text-blue-800 dark:text-blue-200">
              Admin Advanced Filters
            </h3>
          </div>

          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {/* Country Filter */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                🌍 Country
              </label>
              <select
                value={currentParams.country || ''}
                onChange={(e) => handleSelectChange('country', e.target.value)}
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              >
                {COUNTRIES.map(option => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Sector Filter */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                🏭 Sector
              </label>
              <select
                value={currentParams.sector || ''}
                onChange={(e) => handleSelectChange('sector', e.target.value)}
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              >
                {SECTORS.map(option => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Sort By */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                🔄 Sort By
              </label>
              <select
                value={currentParams.sort_by || 'growth_factor'}
                onChange={(e) => handleSelectChange('sort_by', e.target.value)}
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              >
                {SORT_OPTIONS.map(option => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Admin Priority Filter */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                🔴 Admin Priority
              </label>
              <select
                value={searchParams.get('admin_priority') || ''}
                onChange={(e) => handleSelectChange('admin_priority', e.target.value)}
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              >
                {ADMIN_PRIORITY_OPTIONS.map(option => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Admin Status Filter */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                ⚠️ Admin Status
              </label>
              <select
                value={searchParams.get('admin_status') || ''}
                onChange={(e) => handleSelectChange('admin_status', e.target.value)}
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              >
                {ADMIN_STATUS_OPTIONS.map(option => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Min EPS */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                📈 Min EPS
              </label>
              <input
                type="number"
                step="0.01"
                value={minEPS}
                onChange={(e) => setMinEPS(e.target.value)}
                placeholder="e.g., 1.50"
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              />
            </div>

            {/* Min Growth */}
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                📊 Min Growth %
              </label>
              <input
                type="number"
                step="0.1"
                value={minGrowth}
                onChange={(e) => setMinGrowth(e.target.value)}
                placeholder="e.g., 10.0"
                className="w-full rounded-2xl border border-gray-200 bg-white/80 py-2.5 px-3 text-sm backdrop-blur-sm transition-all focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/20 dark:border-gray-600 dark:bg-gray-800/80"
              />
            </div>

            {/* Apply ranges button */}
            <div className="flex items-end">
              <button
                onClick={handleRangeSubmit}
                className="w-full rounded-2xl bg-gradient-to-r from-blue-500 to-indigo-600 py-2.5 px-4 text-sm font-semibold text-white shadow-lg transition-all hover:from-blue-600 hover:to-indigo-700 hover:scale-105"
              >
                Apply Ranges
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Active filters display */}
      {hasActiveFilters && (
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-medium text-gray-600 dark:text-gray-400">Active filters:</span>
          
          {currentParams.search && (
            <div className="flex items-center gap-1 rounded-full bg-orange-100 px-3 py-1 text-sm text-orange-800 dark:bg-orange-900/20 dark:text-orange-200">
              <Search className="h-3 w-3" />
              <span>"{currentParams.search}"</span>
              <button
                onClick={() => updateFilters({ search: undefined })}
                className="ml-1 hover:text-orange-600"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          )}
          
          {currentParams.country && (
            <div className="flex items-center gap-1 rounded-full bg-blue-100 px-3 py-1 text-sm text-blue-800 dark:bg-blue-900/20 dark:text-blue-200">
              <span>🌍 {COUNTRIES.find(c => c.value === currentParams.country)?.label}</span>
              <button
                onClick={() => updateFilters({ country: undefined })}
                className="ml-1 hover:text-blue-600"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          )}

          {currentParams.sector && (
            <div className="flex items-center gap-1 rounded-full bg-green-100 px-3 py-1 text-sm text-green-800 dark:bg-green-900/20 dark:text-green-200">
              <span>🏭 {SECTORS.find(s => s.value === currentParams.sector)?.label}</span>
              <button
                onClick={() => updateFilters({ sector: undefined })}
                className="ml-1 hover:text-green-600"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          )}

          {currentParams.min_eps && (
            <div className="flex items-center gap-1 rounded-full bg-purple-100 px-3 py-1 text-sm text-purple-800 dark:bg-purple-900/20 dark:text-purple-200">
              <span>📈 Min EPS: {currentParams.min_eps}</span>
              <button
                onClick={() => updateFilters({ min_eps: undefined })}
                className="ml-1 hover:text-purple-600"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          )}

          {currentParams.min_growth && (
            <div className="flex items-center gap-1 rounded-full bg-pink-100 px-3 py-1 text-sm text-pink-800 dark:bg-pink-900/20 dark:text-pink-200">
              <span>📊 Min Growth: {currentParams.min_growth}%</span>
              <button
                onClick={() => updateFilters({ min_growth: undefined })}
                className="ml-1 hover:text-pink-600"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}