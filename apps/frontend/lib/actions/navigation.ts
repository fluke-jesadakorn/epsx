'use server'

import { redirect } from 'next/navigation'

/**
 * Server Action for analytics filter navigation
 * Uses redirect for optimal serverless performance (no fetch calls)
 */
export async function navigateToAnalyticsFilter(filters: {
  page?: number
  limit?: number
  country?: string
  sector?: string
  sort_by?: string
  min_eps?: number
  min_growth?: number
}) {
  const params = new URLSearchParams()
  
  // Build clean URL params
  Object.entries(filters).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      params.append(key, String(value))
    }
  })
  
  // Use redirect for optimal serverless navigation
  redirect(`/analytics?${params.toString()}`)
}

/**
 * Server Action for paginated navigation  
 * Uses redirect for optimal serverless performance (no fetch calls)
 */
export async function navigateToAnalyticsPage(page: number, currentFilters: Record<string, any>) {
  const params = new URLSearchParams()
  
  // Preserve existing filters
  Object.entries(currentFilters).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      params.append(key, String(value))
    }
  })
  
  // Update page
  params.set('page', String(page))
  
  redirect(`/analytics?${params.toString()}`)
}

/**
 * Server Action for stock detail navigation
 * Uses redirect for optimal serverless performance (no fetch calls)
 */
export async function navigateToStockDetail(symbol: string) {
  redirect(`/stocks/${symbol}`)
}

/**
 * Server Action for dashboard navigation
 * Uses redirect for optimal serverless performance (no fetch calls)
 */
export async function navigateToDashboard() {
  redirect('/dashboard')
}