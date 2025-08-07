/**
 * E2E Tests for User Profile Tab Navigation
 * Tests the unified user profile interface and tab functionality
 */

import { test, expect, type Page } from '@playwright/test'

test.describe('User Profile Tab Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Mock authentication
    await page.route('**/api/auth/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user: {
            id: 'test-admin',
            email: 'admin@test.com',
            role: 'admin'
          }
        })
      })
    })

    // Mock user data API
    await page.route('**/api/v1/admin/users/*/unified', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'user-123',
          email: 'testuser@example.com',
          displayName: 'Test User',
          status: 'active',
          emailVerified: true,
          twoFactorEnabled: false,
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-15T00:00:00Z',
          lastLogin: '2024-01-15T10:00:00Z',
          roles: [
            {
              id: 'role-1',
              name: 'user',
              description: 'Basic user role',
              isActive: true,
              assignedAt: '2024-01-01T00:00:00Z'
            }
          ],
          permissionProfiles: [
            {
              id: 'profile-1',
              name: 'user-basic-001',
              description: 'Basic user permissions',
              permissions: ['trading:basic'],
              isActive: true,
              assignedAt: '2024-01-01T00:00:00Z'
            }
          ],
          moduleAccess: [
            {
              id: 'module-1',
              moduleName: 'Trading Platform',
              description: 'Basic trading access',
              isActive: true,
              accessLevel: 'basic',
              assignedAt: '2024-01-01T00:00:00Z',
              lastUsed: '2024-01-15T09:00:00Z'
            }
          ],
          stockRankingPackages: [
            {
              id: 'package-1',
              name: 'Basic Package',
              tier: 'basic',
              isActive: true,
              features: ['Basic Analytics'],
              monthlyPrice: 29.99,
              startDate: '2024-01-01T00:00:00Z',
              expirationDate: '2024-12-31T23:59:59Z',
              autoRenew: true
            }
          ],
          billing: {
            tier: 'basic',
            paymentStatus: 'current',
            nextBillingDate: '2024-02-01T00:00:00Z'
          },
          recentActivity: [
            {
              id: 'activity-1',
              type: 'login',
              description: 'User logged in successfully',
              timestamp: '2024-01-15T10:00:00Z',
              category: 'security',
              severity: 'info'
            }
          ],
          usageMetrics: {
            sessionsThisMonth: 15,
            apiCallsThisMonth: 150,
            apiCallsToday: 5,
            avgSessionDuration: 30
          }
        })
      })
    })

    // Mock user list API for search functionality
    await page.route('**/api/v1/admin/users/search**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          users: [
            {
              id: 'user-123',
              email: 'testuser@example.com',
              displayName: 'Test User',
              status: 'active'
            }
          ],
          total: 1,
          page: 1,
          totalPages: 1
        })
      })
    })

    // Navigate to user profile
    await page.goto('/users/user-123')
  })

  test('should display user profile overview by default', async ({ page }) => {
    await expect(page.getByTestId('user-profile-overview')).toBeVisible()
    await expect(page.getByText('Test User')).toBeVisible()
    await expect(page.getByText('testuser@example.com')).toBeVisible()
    
    // Check for key metrics
    await expect(page.getByText('15')).toBeVisible() // Sessions this month
    await expect(page.getByText('150')).toBeVisible() // API calls this month
  })

  test('should navigate to permissions tab', async ({ page }) => {
    await page.click('[data-testid="permissions-tab"]')
    
    await expect(page.url()).toContain('/users/user-123/permissions')
    await expect(page.getByText('User Permissions')).toBeVisible()
    await expect(page.getByText('user-basic-001')).toBeVisible()
    await expect(page.getByText('Basic user role')).toBeVisible()
  })

  test('should navigate to modules tab', async ({ page }) => {
    await page.click('[data-testid="modules-tab"]')
    
    await expect(page.url()).toContain('/users/user-123/modules')
    await expect(page.getByText('Module Access')).toBeVisible()
    await expect(page.getByText('Trading Platform')).toBeVisible()
    await expect(page.getByText('Basic trading access')).toBeVisible()
  })

  test('should navigate to packages tab', async ({ page }) => {
    await page.click('[data-testid="packages-tab"]')
    
    await expect(page.url()).toContain('/users/user-123/packages')
    await expect(page.getByText('Stock Ranking Packages')).toBeVisible()
    await expect(page.getByText('Basic Package')).toBeVisible()
    await expect(page.getByText('$29.99/month')).toBeVisible()
  })

  test('should navigate to activity tab', async ({ page }) => {
    await page.click('[data-testid="activity-tab"]')
    
    await expect(page.url()).toContain('/users/user-123/activity')
    await expect(page.getByText('Recent Activity')).toBeVisible()
    await expect(page.getByText('User logged in successfully')).toBeVisible()
  })

  test('should maintain tab state in URL', async ({ page }) => {
    // Navigate to permissions tab
    await page.click('[data-testid="permissions-tab"]')
    await expect(page.url()).toContain('/permissions')
    
    // Refresh page
    await page.reload()
    
    // Should still be on permissions tab
    await expect(page.url()).toContain('/permissions')
    await expect(page.getByText('User Permissions')).toBeVisible()
  })

  test('should handle direct navigation to tabs', async ({ page }) => {
    // Direct navigation to modules tab
    await page.goto('/users/user-123/modules')
    
    await expect(page.getByText('Module Access')).toBeVisible()
    await expect(page.getByText('Trading Platform')).toBeVisible()
    
    // Tab should be active
    const modulesTab = page.getByTestId('modules-tab')
    await expect(modulesTab).toHaveAttribute('aria-selected', 'true')
  })
})


test.describe('User Profile Responsive Design', () => {
  test('should handle mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 }) // iPhone SE
    
    // Mock APIs
    await page.route('**/api/auth/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ user: { id: 'test-admin', role: 'admin' } })
      })
    })

    await page.route('**/api/v1/admin/users/*/unified', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'user-123',
          email: 'testuser@example.com',
          displayName: 'Test User',
          status: 'active'
        })
      })
    })

    await page.goto('/users/user-123')
    
    // Should still be functional on mobile
    await expect(page.getByText('Test User')).toBeVisible()
    
    // Tabs might be stacked or scrollable on mobile
    await page.click('[data-testid="permissions-tab"]')
    await expect(page.url()).toContain('/permissions')
  })

  test('should handle tablet viewport', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 }) // iPad
    
    // Mock APIs
    await page.route('**/api/auth/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ user: { id: 'test-admin', role: 'admin' } })
      })
    })

    await page.route('**/api/v1/admin/users/*/unified', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'user-123',
          email: 'testuser@example.com',
          displayName: 'Test User',
          status: 'active'
        })
      })
    })

    await page.goto('/users/user-123')
    
    await expect(page.getByText('Test User')).toBeVisible()
    
    // Tab navigation should work smoothly on tablet
    await page.click('[data-testid="modules-tab"]')
    await expect(page.url()).toContain('/modules')
  })
})

test.describe('User Profile Error Handling', () => {
  test('should handle user not found', async ({ page }) => {
    // Mock authentication
    await page.route('**/api/auth/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ user: { id: 'test-admin', role: 'admin' } })
      })
    })

    // Mock 404 response for user data
    await page.route('**/api/v1/admin/users/nonexistent/unified', async (route) => {
      await route.fulfill({ status: 404 })
    })

    await page.goto('/users/nonexistent')
    
    // Should show 404 page or error message
    await expect(page.getByText('User not found')).toBeVisible()
  })

  test('should handle API errors gracefully', async ({ page }) => {
    // Mock authentication
    await page.route('**/api/auth/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ user: { id: 'test-admin', role: 'admin' } })
      })
    })

    // Mock server error
    await page.route('**/api/v1/admin/users/*/unified', async (route) => {
      await route.fulfill({ status: 500 })
    })

    await page.goto('/users/user-123')
    
    // Should show error state
    await expect(page.getByText('Failed to load user data')).toBeVisible()
  })
})