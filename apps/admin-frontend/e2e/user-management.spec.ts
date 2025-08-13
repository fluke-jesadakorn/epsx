import { test, expect } from '@playwright/test'

test.describe('Admin User Management', () => {
  test.beforeEach(async ({ page }) => {
    // Mock authentication for admin user
    await page.goto('/login')
    
    // Fill in admin credentials
    await page.fill('[data-testid="email-input"]', 'admin@epsx.com')
    await page.fill('[data-testid="password-input"]', 'admin123')
    await page.click('[data-testid="login-button"]')
    
    // Wait for redirect to admin dashboard
    await page.waitForURL('/dashboard')
    await expect(page.locator('[data-testid="admin-dashboard"]')).toBeVisible()
  })

  test('should display user management interface', async ({ page }) => {
    await page.goto('/users')
    
    // Check if user management page loads
    await expect(page.locator('h1')).toContainText('User Management')
    await expect(page.locator('[data-testid="user-list"]')).toBeVisible()
    
    // Check if search functionality is present
    await expect(page.locator('[data-testid="user-search"]')).toBeVisible()
    
    // Check if create user button is present
    await expect(page.locator('[data-testid="create-user-button"]')).toBeVisible()
  })

  test('should create a new user', async ({ page }) => {
    await page.goto('/users')
    
    // Click create user button
    await page.click('[data-testid="create-user-button"]')
    
    // Wait for modal to open
    await expect(page.locator('[data-testid="create-user-modal"]')).toBeVisible()
    
    // Fill in user details
    await page.fill('[data-testid="user-email-input"]', 'newuser@example.com')
    await page.fill('[data-testid="user-name-input"]', 'New Test User')
    await page.selectOption('[data-testid="user-role-select"]', 'user-basic-001')
    
    // Submit form
    await page.click('[data-testid="submit-user-button"]')
    
    // Verify success message
    await expect(page.locator('[data-testid="success-toast"]')).toContainText('User created successfully')
    
    // Verify user appears in list
    await expect(page.locator('[data-testid="user-list"]')).toContainText('newuser@example.com')
  })

  test('should search users by email', async ({ page }) => {
    await page.goto('/users')
    
    // Wait for user list to load
    await page.waitForSelector('[data-testid="user-list"]')
    
    // Search for specific user
    await page.fill('[data-testid="user-search"]', 'admin@epsx.com')
    await page.press('[data-testid="user-search"]', 'Enter')
    
    // Wait for search results
    await page.waitForTimeout(1000)
    
    // Verify search results
    await expect(page.locator('[data-testid="user-list"]')).toContainText('admin@epsx.com')
  })

  test('should edit user permissions', async ({ page }) => {
    await page.goto('/users')
    
    // Click on first user's edit button
    await page.click('[data-testid="user-row"]:first-child [data-testid="edit-user-button"]')
    
    // Wait for user detail page
    await page.waitForURL('/users/*')
    
    // Navigate to permissions tab
    await page.click('[data-testid="permissions-tab"]')
    
    // Wait for permissions interface to load
    await expect(page.locator('[data-testid="permission-assignment"]')).toBeVisible()
    
    // Add a new permission
    await page.click('[data-testid="add-permission-button"]')
    await page.selectOption('[data-testid="permission-select"]', 'user-premium-002')
    await page.click('[data-testid="confirm-permission-button"]')
    
    // Verify success
    await expect(page.locator('[data-testid="success-toast"]')).toContainText('Permission assigned successfully')
  })

  test('should handle bulk user operations', async ({ page }) => {
    await page.goto('/users')
    
    // Select multiple users
    await page.check('[data-testid="user-row"]:first-child [data-testid="user-checkbox"]')
    await page.check('[data-testid="user-row"]:nth-child(2) [data-testid="user-checkbox"]')
    
    // Verify bulk actions panel appears
    await expect(page.locator('[data-testid="bulk-actions-panel"]')).toBeVisible()
    
    // Perform bulk permission assignment
    await page.click('[data-testid="bulk-assign-permission"]')
    await page.selectOption('[data-testid="bulk-permission-select"]', 'user-premium-002')
    await page.click('[data-testid="confirm-bulk-assignment"]')
    
    // Verify success
    await expect(page.locator('[data-testid="success-toast"]')).toContainText('Bulk permission assignment completed')
  })

  test('should validate admin access only', async ({ page }) => {
    // Logout and login as regular user
    await page.goto('/logout')
    await page.goto('/login')
    
    await page.fill('[data-testid="email-input"]', 'user@example.com')
    await page.fill('[data-testid="password-input"]', 'user123')
    await page.click('[data-testid="login-button"]')
    
    // Try to access admin area
    await page.goto('/users')
    
    // Should be redirected to access denied or dashboard
    await expect(page.locator('[data-testid="access-denied"]')).toBeVisible()
  })
})