/**
 * E2E Tests for Advanced User Filtering and Search
 * Tests comprehensive filtering functionality with URL parameters
 */

import { test, expect, type Page } from '@playwright/test'
import { APIMocks } from './utils/api-mocks'

test.describe('Advanced User Filtering and Search', () => {
  let apiMocks: APIMocks

  const mockUsers = [
    {
      user_id: 'user-001',
      email: 'alice.johnson@company.com',
      display_name: 'Alice Johnson',
      role: 'user-basic-001',
      status: 'active',
      email_verified: true,
      created_at: '2024-01-15T10:30:00Z',
      last_login: '2024-01-20T14:22:00Z',
      subscription_tier: 'basic'
    },
    {
      user_id: 'user-002', 
      email: 'bob.smith@example.org',
      display_name: 'Bob Smith',
      role: 'user-premium-002',
      status: 'active',
      email_verified: true,
      created_at: '2024-01-10T08:45:00Z',
      last_login: '2024-01-19T16:30:00Z',
      subscription_tier: 'premium'
    },
    {
      user_id: 'user-003',
      email: 'charlie.brown@test.com',
      display_name: 'Charlie Brown',
      role: 'moderator-standard-003',
      status: 'inactive',
      email_verified: false,
      created_at: '2024-01-05T12:15:00Z',
      last_login: '2024-01-18T11:00:00Z',
      subscription_tier: 'premium'
    },
    {
      user_id: 'user-004',
      email: 'diana.prince@hero.com',
      display_name: 'Diana Prince',
      role: 'admin-full-004',
      status: 'active',
      email_verified: true,
      created_at: '2023-12-28T09:00:00Z',
      last_login: '2024-01-21T07:45:00Z',
      subscription_tier: 'premium'
    },
    {
      user_id: 'user-005',
      email: 'edward.norton@film.com',
      display_name: 'Edward Norton',
      role: 'user-basic-001',
      status: 'suspended',
      email_verified: true,
      created_at: '2024-01-12T14:20:00Z',
      last_login: '2024-01-17T09:30:00Z',
      subscription_tier: 'basic'
    }
  ]

  test.beforeEach(async ({ page }) => {
    apiMocks = new APIMocks(page)
    await apiMocks.mockSuccessfulAuth('admin@epsx.com', 'admin')
    
    // Mock advanced user list API with full filtering support
    await page.route('**/api/admin/users**', async (route) => {
      const url = new URL(route.request().url())
      const searchParams = url.searchParams
      
      let filteredUsers = [...mockUsers]
      
      // Apply search filter
      const search = searchParams.get('search')?.toLowerCase()
      if (search) {
        filteredUsers = filteredUsers.filter(user => 
          user.email.toLowerCase().includes(search) ||
          user.display_name.toLowerCase().includes(search) ||
          user.user_id.toLowerCase().includes(search)
        )
      }
      
      // Apply status filter
      const status = searchParams.get('status')
      if (status && status !== 'all') {
        filteredUsers = filteredUsers.filter(user => user.status === status)
      }
      
      // Apply role filter
      const role = searchParams.get('role')
      if (role && role !== 'all') {
        filteredUsers = filteredUsers.filter(user => user.role === role)
      }
      
      // Apply subscription tier filter
      const tier = searchParams.get('tier')
      if (tier && tier !== 'all') {
        filteredUsers = filteredUsers.filter(user => user.subscription_tier === tier)
      }
      
      // Apply email verification filter
      const emailVerified = searchParams.get('emailVerified')
      if (emailVerified && emailVerified !== 'all') {
        const isVerified = emailVerified === 'true'
        filteredUsers = filteredUsers.filter(user => user.email_verified === isVerified)
      }
      
      // Apply sorting
      const sortBy = searchParams.get('sortBy') || 'created_at'
      const sortOrder = searchParams.get('sortOrder') || 'desc'
      
      filteredUsers.sort((a, b) => {
        const aValue = a[sortBy as keyof typeof a] as string
        const bValue = b[sortBy as keyof typeof b] as string
        
        let comparison: number
        if (sortBy.includes('_at')) {
          // Date comparison
          comparison = new Date(aValue).getTime() - new Date(bValue).getTime()
        } else {
          // String comparison
          comparison = aValue.localeCompare(bValue)
        }
        
        return sortOrder === 'asc' ? comparison : -comparison
      })
      
      // Apply pagination
      const page = parseInt(searchParams.get('page') || '1')
      const limit = parseInt(searchParams.get('limit') || '10')
      const startIndex = (page - 1) * limit
      const endIndex = startIndex + limit
      const paginatedUsers = filteredUsers.slice(startIndex, endIndex)
      
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          users: paginatedUsers,
          total: filteredUsers.length,
          page,
          totalPages: Math.ceil(filteredUsers.length / limit)
        })
      })
    })
  })

  test('should perform basic text search', async ({ page }) => {
    await page.goto('/users')
    
    // Wait for initial load
    await expect(page.getByText('Alice Johnson')).toBeVisible()
    
    // Search by name
    await page.getByPlaceholder('Search users...').fill('alice')
    
    // Should show only matching user
    await expect(page.getByText('Alice Johnson')).toBeVisible()
    await expect(page.getByText('Bob Smith')).not.toBeVisible()
    
    // Clear and search by email domain
    await page.getByPlaceholder('Search users...').fill('')
    await page.getByPlaceholder('Search users...').fill('@example.org')
    
    await expect(page.getByText('Bob Smith')).toBeVisible()
    await expect(page.getByText('Alice Johnson')).not.toBeVisible()
  })

  test('should filter by multiple criteria simultaneously', async ({ page }) => {
    await page.goto('/users')
    
    // Apply status filter
    await page.getByRole('combobox', { name: /status/i }).click()
    await page.getByRole('option', { name: 'Active' }).click()
    
    // Apply role filter
    await page.getByRole('combobox', { name: /role/i }).click()
    await page.getByRole('option', { name: /premium/i }).click()
    
    // Should show only active premium users
    await expect(page.getByText('Bob Smith')).toBeVisible()
    await expect(page.getByText('Alice Johnson')).not.toBeVisible() // basic role
    await expect(page.getByText('Charlie Brown')).not.toBeVisible() // inactive
    
    // Add search term
    await page.getByPlaceholder('Search users...').fill('diana')
    
    // Should show Diana if she matches all criteria
    await expect(page.getByText('Diana Prince')).toBeVisible()
    await expect(page.getByText('Bob Smith')).not.toBeVisible()
  })

  test('should maintain all filter state in URL parameters', async ({ page }) => {
    await page.goto('/users')
    
    // Apply multiple filters
    await page.getByPlaceholder('Search users...').fill('test search')
    
    await page.getByRole('combobox', { name: /status/i }).click()
    await page.getByRole('option', { name: 'Inactive' }).click()
    
    await page.getByRole('combobox', { name: /role/i }).click()
    await page.getByRole('option', { name: /moderator/i }).click()
    
    // URL should contain all parameters
    await expect(page.url()).toContain('search=test+search')
    await expect(page.url()).toContain('status=inactive')
    await expect(page.url()).toContain('role=moderator')
    
    // Refresh page
    await page.reload()
    
    // All filters should be restored
    await expect(page.getByPlaceholder('Search users...')).toHaveValue('test search')
    // Note: Checking combobox selected values would require specific data-testid attributes
  })

  test('should sort users by different columns', async ({ page }) => {
    await page.goto('/users')
    
    // Sort by email ascending
    await page.getByRole('button', { name: /sort.*email/i }).click()
    
    // URL should contain sort parameters
    await expect(page.url()).toContain('sortBy=email')
    await expect(page.url()).toContain('sortOrder=asc')
    
    // Click again to sort descending
    await page.getByRole('button', { name: /sort.*email/i }).click()
    await expect(page.url()).toContain('sortOrder=desc')
    
    // Sort by creation date
    await page.getByRole('button', { name: /sort.*created/i }).click()
    await expect(page.url()).toContain('sortBy=created_at')
    
    // Sort by last login
    await page.getByRole('button', { name: /sort.*login/i }).click()
    await expect(page.url()).toContain('sortBy=last_login')
  })

  test('should handle pagination with filters', async ({ page }) => {
    await page.goto('/users')
    
    // Apply a filter first
    await page.getByRole('combobox', { name: /status/i }).click()
    await page.getByRole('option', { name: 'Active' }).click()
    
    // Change page size
    await page.getByRole('combobox', { name: /per page/i }).click()
    await page.getByRole('option', { name: '5' }).click()
    
    await expect(page.url()).toContain('limit=5')
    await expect(page.url()).toContain('status=active')
    
    // Navigate to next page
    if (await page.getByRole('button', { name: /next page/i }).isVisible()) {
      await page.getByRole('button', { name: /next page/i }).click()
      await expect(page.url()).toContain('page=2')
      
      // Filters should be maintained across pages
      await expect(page.url()).toContain('status=active')
      await expect(page.url()).toContain('limit=5')
    }
  })

  test('should reset all filters', async ({ page }) => {
    await page.goto('/users')
    
    // Apply multiple filters
    await page.getByPlaceholder('Search users...').fill('test')
    await page.getByRole('combobox', { name: /status/i }).click()
    await page.getByRole('option', { name: 'Inactive' }).click()
    
    // Verify filters applied
    await expect(page.url()).toContain('search=test')
    await expect(page.url()).toContain('status=inactive')
    
    // Reset filters
    await page.getByRole('button', { name: /reset.*filters/i }).click()
    
    // URL should be clean
    await expect(page.url()).not.toContain('search=')
    await expect(page.url()).not.toContain('status=')
    
    // All users should be visible again
    await expect(page.getByText('Alice Johnson')).toBeVisible()
    await expect(page.getByText('Bob Smith')).toBeVisible()
    
    // Search field should be empty
    await expect(page.getByPlaceholder('Search users...')).toHaveValue('')
  })

  test('should handle advanced filter combinations', async ({ page }) => {
    await page.goto('/users')
    
    // Complex filter: Active premium users created this year
    await page.getByRole('combobox', { name: /status/i }).click()
    await page.getByRole('option', { name: 'Active' }).click()
    
    // If tier filter exists
    if (await page.getByRole('combobox', { name: /tier/i }).isVisible()) {
      await page.getByRole('combobox', { name: /tier/i }).click()
      await page.getByRole('option', { name: 'Premium' }).click()
      
      await expect(page.url()).toContain('tier=premium')
    }
    
    // Should show only users matching all criteria
    await expect(page.getByText('Bob Smith')).toBeVisible()
    await expect(page.getByText('Diana Prince')).toBeVisible()
    await expect(page.getByText('Charlie Brown')).not.toBeVisible() // inactive
    await expect(page.getByText('Alice Johnson')).not.toBeVisible() // basic tier
  })

  test('should handle no results gracefully', async ({ page }) => {
    await page.goto('/users')
    
    // Search for non-existent user
    await page.getByPlaceholder('Search users...').fill('nonexistent@nowhere.com')
    
    // Should show no results message
    await expect(page.getByText('No users found')).toBeVisible()
    await expect(page.getByText('Try adjusting your search criteria')).toBeVisible()
    
    // Clear search to show results again
    await page.getByPlaceholder('Search users...').fill('')
    await expect(page.getByText('Alice Johnson')).toBeVisible()
  })

  test('should handle filter state on direct URL navigation', async ({ page }) => {
    // Navigate directly with filter parameters
    await page.goto('/users?search=alice&status=active&sortBy=email&sortOrder=asc&page=1&limit=10')
    
    // Filters should be applied from URL
    await expect(page.getByPlaceholder('Search users...')).toHaveValue('alice')
    await expect(page.getByText('Alice Johnson')).toBeVisible()
    await expect(page.getByText('Bob Smith')).not.toBeVisible()
    
    // Sort and status indicators should reflect URL state
    // (This would depend on UI implementation details)
  })

  test('should maintain filter state when navigating away and back', async ({ page }) => {
    await page.goto('/users')
    
    // Apply filters
    await page.getByPlaceholder('Search users...').fill('premium')
    await page.getByRole('combobox', { name: /status/i }).click()
    await page.getByRole('option', { name: 'Active' }).click()
    
    // Navigate to a user profile
    if (await page.getByText('Bob Smith').isVisible()) {
      await page.getByText('Bob Smith').click()
      await expect(page.url()).toContain('/users/user-002')
    }
    
    // Navigate back to user list
    await page.getByRole('button', { name: /back/i }).click()
    
    // Filters should be maintained
    await expect(page.getByPlaceholder('Search users...')).toHaveValue('premium')
    await expect(page.url()).toContain('search=premium')
    await expect(page.url()).toContain('status=active')
  })

  test('should handle real-time search with debouncing', async ({ page }) => {
    await page.goto('/users')
    
    const searchInput = page.getByPlaceholder('Search users...')
    
    // Type search query character by character
    await searchInput.type('ali', { delay: 100 })
    
    // Should eventually show Alice Johnson
    await expect(page.getByText('Alice Johnson')).toBeVisible()
    
    // Continue typing to refine search
    await searchInput.type('ce', { delay: 100 })
    
    // Should still show Alice Johnson
    await expect(page.getByText('Alice Johnson')).toBeVisible()
    await expect(page.getByText('Bob Smith')).not.toBeVisible()
    
    // Clear search quickly
    await searchInput.clear()
    
    // Should show all users again
    await expect(page.getByText('Bob Smith')).toBeVisible()
  })
})