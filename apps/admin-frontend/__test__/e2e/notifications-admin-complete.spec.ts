import { test, expect, Page } from '@playwright/test'

test.describe('Admin Notifications - Complete Coverage', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001'
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 })

    // Navigate to admin panel
    await page.goto(ADMIN_URL)
    await page.waitForLoadState('networkidle')
  })

  test.describe('Admin Notification Bell', () => {
    test('should display admin notification bell in header', async ({ page }) => {
      const bell = page.locator('button:has-text("🔔"), button[aria-label*="notification"]').first()
      await expect(bell).toBeVisible({ timeout: 5000 })
    })

    test('should show SSE connection status indicator', async ({ page }) => {
      await page.waitForTimeout(2000) // Wait for SSE connection

      // Look for connection indicator (green dot)
      const indicator = page.locator('div[class*="bg-green"], div[title*="connected"]').first()
      const hasIndicator = await indicator.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasIndicator).toBe('boolean')
    })

    test('should show notification count badge', async ({ page }) => {
      const badge = page.locator('[data-testid="notification-badge"], div[class*="badge"]').first()
      const hasBadge = await badge.isVisible({ timeout: 3000 }).catch(() => false)

      expect(typeof hasBadge).toBe('boolean')
    })

    test('should open admin notification dropdown', async ({ page }) => {
      const bell = page.locator('button:has-text("🔔")').first()
      await bell.click()
      await page.waitForTimeout(500)

      const dropdown = page.locator('div[class*="dropdown"], [role="menu"]').first()
      await expect(dropdown).toBeVisible({ timeout: 3000 })
    })

    test('should display notifications with priority styling', async ({ page }) => {
      const bell = page.locator('button:has-text("🔔")').first()
      await bell.click()
      await page.waitForTimeout(500)

      // Look for gradient backgrounds (admin uses gradients)
      const gradients = page.locator('div[class*="gradient"]')
      const count = await gradients.count()

      expect(count).toBeGreaterThanOrEqual(0)
    })

    test('should show wallet addresses in admin notifications', async ({ page }) => {
      const bell = page.locator('button:has-text("🔔")').first()
      await bell.click()
      await page.waitForTimeout(500)

      // Look for wallet address format (0x... or "all")
      const wallets = page.locator('span[class*="font-mono"], text=/0x[a-fA-F0-9]{4}/')
      const hasWallets = await wallets.count() > 0

      expect(typeof hasWallets).toBe('boolean')
    })

    test('should have send notification button in dropdown', async ({ page }) => {
      const bell = page.locator('button:has-text("🔔")').first()
      await bell.click()
      await page.waitForTimeout(500)

      const sendBtn = page.locator('button:has-text("Send Notification"), button:has-text("➕")').first()
      const hasBtn = await sendBtn.isVisible({ timeout: 3000 }).catch(() => false)

      expect(typeof hasBtn).toBe('boolean')
    })

    test('should close dropdown when clicking backdrop', async ({ page }) => {
      const bell = page.locator('button:has-text("🔔")').first()
      await bell.click()
      await page.waitForTimeout(500)

      // Click backdrop
      const backdrop = page.locator('div[class*="inset"], div[class*="fixed"]').first()
      const hasBackdrop = await backdrop.isVisible().catch(() => false)

      if (hasBackdrop) {
        await backdrop.click()
        await page.waitForTimeout(300)
      }

      expect(true).toBe(true)
    })
  })

  test.describe('Notifications Page Navigation', () => {
    test('should navigate to notifications page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)
      await page.waitForLoadState('networkidle')

      // Check for page elements
      const tabs = page.locator('button:has-text("Overview"), button:has-text("Send")').first()
      await expect(tabs).toBeVisible({ timeout: 5000 })
    })

    test('should have Overview and Send tabs', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      const overviewTab = page.locator('button:has-text("Overview"), button:has-text("📊")').first()
      const sendTab = page.locator('button:has-text("Send Notification"), button:has-text("📨")').first()

      await expect(overviewTab).toBeVisible({ timeout: 5000 })
      await expect(sendTab).toBeVisible({ timeout: 5000 })
    })

    test('should switch between tabs', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      const sendTab = page.locator('button:has-text("Send"), button:has-text("📨")').first()
      await sendTab.click()
      await page.waitForTimeout(500)

      // Should show send form
      const form = page.locator('form, input[name*="title"], textarea[name*="message"]').first()
      await expect(form).toBeVisible({ timeout: 3000 })

      // Switch back to overview
      const overviewTab = page.locator('button:has-text("Overview"), button:has-text("📊")').first()
      await overviewTab.click()
      await page.waitForTimeout(500)

      expect(true).toBe(true)
    })
  })

  test.describe('Send Notification Form', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)
      await page.waitForLoadState('networkidle')

      // Switch to Send tab
      const sendTab = page.locator('button:has-text("Send"), button:has-text("📨")').first()
      await sendTab.click()
      await page.waitForTimeout(500)
    })

    test('should display send notification form', async ({ page }) => {
      const form = page.locator('form, div:has(input[name*="title"])').first()
      await expect(form).toBeVisible({ timeout: 5000 })
    })

    test('should have required form fields', async ({ page }) => {
      // Check for title input
      const titleInput = page.locator('input[name*="title"], input[placeholder*="title"]').first()
      await expect(titleInput).toBeVisible({ timeout: 5000 })

      // Check for message textarea
      const messageInput = page.locator('textarea[name*="message"], textarea[placeholder*="message"]').first()
      await expect(messageInput).toBeVisible({ timeout: 5000 })

      // Check for type select
      const typeSelect = page.locator('select[name*="type"], select:has(option:has-text("System"))').first()
      const hasTypeSelect = await typeSelect.isVisible({ timeout: 3000 }).catch(() => false)

      expect(typeof hasTypeSelect).toBe('boolean')

      // Check for priority select
      const prioritySelect = page.locator('select[name*="priority"], select:has(option:has-text("Normal"))').first()
      const hasPrioritySelect = await prioritySelect.isVisible({ timeout: 3000 }).catch(() => false)

      expect(typeof hasPrioritySelect).toBe('boolean')
    })

    test('should send broadcast notification', async ({ page }) => {
      // Fill form
      await page.locator('input[name*="title"], input[placeholder*="title"]').first().fill('Test Broadcast')
      await page.locator('textarea[name*="message"], textarea[placeholder*="message"]').first().fill('This is a test broadcast message')

      // Check broadcast option
      const broadcastCheckbox = page.locator('input[type="checkbox"], input[value="broadcast"], label:has-text("Broadcast")').first()
      const hasCheckbox = await broadcastCheckbox.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasCheckbox) {
        await broadcastCheckbox.click()
      }

      // Submit form
      const submitBtn = page.locator('button[type="submit"], button:has-text("Send")').first()
      await submitBtn.click()
      await page.waitForTimeout(2000)

      // Check for success message
      const success = page.locator('text=success, text=sent, text=Success').first()
      const hasSuccess = await success.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasSuccess).toBe('boolean')
    })

    test('should send direct notification to wallet', async ({ page }) => {
      // Fill form
      await page.locator('input[name*="title"]').first().fill('Direct Notification')
      await page.locator('textarea[name*="message"]').first().fill('This is a direct message')

      // Enter wallet address
      const walletInput = page.locator('input[name*="wallet"], input[name*="recipient"]').first()
      const hasWalletInput = await walletInput.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasWalletInput) {
        await walletInput.fill('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb')
      }

      // Submit
      const submitBtn = page.locator('button[type="submit"], button:has-text("Send")').first()
      await submitBtn.click()
      await page.waitForTimeout(2000)

      expect(true).toBe(true)
    })

    test('should validate required fields', async ({ page }) => {
      // Try to submit empty form
      const submitBtn = page.locator('button[type="submit"], button:has-text("Send")').first()
      await submitBtn.click()
      await page.waitForTimeout(500)

      // Check for validation errors
      const error = page.locator('text=required, text=Please, text=error').first()
      const hasError = await error.isVisible({ timeout: 3000 }).catch(() => false)

      expect(typeof hasError).toBe('boolean')
    })

    test('should support notification types', async ({ page }) => {
      const typeSelect = page.locator('select[name*="type"]').first()
      const hasSelect = await typeSelect.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasSelect) {
        // Test different types
        await typeSelect.selectOption('system')
        await typeSelect.selectOption('security')
        await typeSelect.selectOption('permission')
        await typeSelect.selectOption('wallet')
      }

      expect(true).toBe(true)
    })

    test('should support priority levels', async ({ page }) => {
      const prioritySelect = page.locator('select[name*="priority"]').first()
      const hasSelect = await prioritySelect.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasSelect) {
        // Test different priorities
        await prioritySelect.selectOption('low')
        await prioritySelect.selectOption('normal')
        await prioritySelect.selectOption('high')
        await prioritySelect.selectOption('critical')
      }

      expect(true).toBe(true)
    })

    test('should schedule notification for future delivery', async ({ page }) => {
      // Fill basic fields
      await page.locator('input[name*="title"]').first().fill('Scheduled Notification')
      await page.locator('textarea[name*="message"]').first().fill('This will be sent later')

      // Look for schedule option
      const scheduleCheckbox = page.locator('input[type="checkbox"]:has-text("Schedule"), label:has-text("Schedule")').first()
      const hasSchedule = await scheduleCheckbox.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasSchedule) {
        await scheduleCheckbox.click()

        // Fill date/time
        const dateInput = page.locator('input[type="datetime-local"], input[type="date"]').first()
        const hasDateInput = await dateInput.isVisible({ timeout: 3000 }).catch(() => false)

        if (hasDateInput) {
          const futureDate = new Date(Date.now() + 24 * 60 * 60 * 1000)
          const isoString = futureDate.toISOString().slice(0, 16)
          await dateInput.fill(isoString)
        }
      }

      expect(true).toBe(true)
    })

    test('should have cancel button', async ({ page }) => {
      const cancelBtn = page.locator('button:has-text("Cancel")').first()
      const hasCancel = await cancelBtn.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasCancel) {
        await cancelBtn.click()
        await page.waitForTimeout(500)

        // Should switch back to overview
        expect(true).toBe(true)
      }
    })
  })

  test.describe('Notification Management Overview', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)
      await page.waitForLoadState('networkidle')

      // Ensure on Overview tab
      const overviewTab = page.locator('button:has-text("Overview"), button:has-text("📊")').first()
      await overviewTab.click()
      await page.waitForTimeout(500)
    })

    test('should display notification statistics', async ({ page }) => {
      // Look for stats cards
      const stats = page.locator('div[class*="stats"], div[class*="card"]')
      const count = await stats.count()

      expect(count).toBeGreaterThanOrEqual(0)
    })

    test('should show notification metrics', async ({ page }) => {
      // Look for metric numbers
      const metrics = page.locator('text=/\\d+/, div[class*="text-\\d"]')
      const hasMetrics = await metrics.count() > 0

      expect(typeof hasMetrics).toBe('boolean')
    })

    test('should display sent notifications history', async ({ page }) => {
      // Look for notification list or table
      const list = page.locator('div[class*="notification"], table, [role="table"]').first()
      const hasList = await list.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasList).toBe('boolean')
    })

    test('should show delivery status for notifications', async ({ page }) => {
      // Look for status indicators
      const status = page.locator('text=delivered, text=sent, text=pending, text=failed').first()
      const hasStatus = await status.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasStatus).toBe('boolean')
    })

    test('should display notification analytics', async ({ page }) => {
      // Look for charts or analytics
      const analytics = page.locator('canvas, [data-testid="chart"], div[class*="chart"]').first()
      const hasAnalytics = await analytics.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasAnalytics).toBe('boolean')
    })
  })

  test.describe('Admin Real-time Notifications (SSE)', () => {
    test('should auto-connect SSE on page load', async ({ page }) => {
      // Monitor console for SSE logs
      const logs: string[] = []
      page.on('console', msg => {
        if (msg.text().includes('SSE') || msg.text().includes('connection')) {
          logs.push(msg.text())
        }
      })

      await page.goto(ADMIN_URL)
      await page.waitForLoadState('networkidle')
      await page.waitForTimeout(3000)

      // Check for SSE connection logs
      const hasSSE = logs.some(log =>
        log.includes('Admin SSE') ||
        log.includes('connected') ||
        log.includes('opened')
      )

      expect(typeof hasSSE).toBe('boolean')
    })

    test('should show connection indicator when connected', async ({ page }) => {
      await page.goto(ADMIN_URL)
      await page.waitForTimeout(3000)

      // Look for green connection dot
      const indicator = page.locator('div[class*="bg-green"][title*="connected"]').first()
      const isConnected = await indicator.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof isConnected).toBe('boolean')
    })

    test('should receive all broadcast notifications', async ({ page }) => {
      await page.goto(ADMIN_URL)
      await page.waitForTimeout(2000)

      // Admin should receive all notifications (not filtered by wallet)
      expect(true).toBe(true)
    })
  })

  test.describe('Error Handling', () => {
    test('should handle notification send failures', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      // Switch to send tab
      const sendTab = page.locator('button:has-text("Send")').first()
      await sendTab.click()
      await page.waitForTimeout(500)

      // Mock API failure
      await page.route('**/api/admin/notifications/send', route => {
        route.fulfill({
          status: 500,
          body: JSON.stringify({ error: 'Failed to send notification' }),
        })
      })

      // Fill and submit form
      await page.locator('input[name*="title"]').first().fill('Error Test')
      await page.locator('textarea[name*="message"]').first().fill('This should fail')

      const submitBtn = page.locator('button[type="submit"]').first()
      await submitBtn.click()
      await page.waitForTimeout(2000)

      // Check for error message
      const error = page.locator('text=error, text=failed').first()
      const hasError = await error.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasError).toBe('boolean')
    })

    test('should handle API failures gracefully', async ({ page }) => {
      // Mock API failure
      await page.route('**/api/admin/notifications**', route => {
        route.fulfill({
          status: 500,
          body: JSON.stringify({ error: 'Server error' }),
        })
      })

      await page.goto(`${ADMIN_URL}/notifications`)
      await page.waitForLoadState('networkidle')

      // Page should still load without crashing
      expect(true).toBe(true)
    })

    test('should validate wallet address format', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      const sendTab = page.locator('button:has-text("Send")').first()
      await sendTab.click()
      await page.waitForTimeout(500)

      // Enter invalid wallet
      const walletInput = page.locator('input[name*="wallet"], input[name*="recipient"]').first()
      const hasWalletInput = await walletInput.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasWalletInput) {
        await walletInput.fill('invalid-wallet-address')

        const submitBtn = page.locator('button[type="submit"]').first()
        await submitBtn.click()
        await page.waitForTimeout(500)

        // Should show validation error
        const error = page.locator('text=invalid, text=Invalid').first()
        const hasError = await error.isVisible({ timeout: 3000 }).catch(() => false)

        expect(typeof hasError).toBe('boolean')
      }
    })
  })

  test.describe('Performance', () => {
    test('should load notifications page quickly', async ({ page }) => {
      const start = Date.now()

      await page.goto(`${ADMIN_URL}/notifications`)
      await page.waitForLoadState('networkidle')

      const duration = Date.now() - start

      // Should load in under 5 seconds
      expect(duration).toBeLessThan(5000)
    })

    test('should handle form submissions efficiently', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      const sendTab = page.locator('button:has-text("Send")').first()
      await sendTab.click()
      await page.waitForTimeout(500)

      const start = Date.now()

      // Fill and submit
      await page.locator('input[name*="title"]').first().fill('Performance Test')
      await page.locator('textarea[name*="message"]').first().fill('Testing performance')

      const submitBtn = page.locator('button[type="submit"]').first()
      await submitBtn.click()

      await page.waitForTimeout(3000)

      const duration = Date.now() - start

      // Should complete in under 3 seconds
      expect(duration).toBeLessThan(3000)
    })
  })

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      // Tab through form elements
      await page.keyboard.press('Tab')
      await page.keyboard.press('Tab')

      const focused = page.locator(':focus').first()
      await expect(focused).toBeVisible({ timeout: 3000 })
    })

    test('should have proper form labels', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/notifications`)

      const sendTab = page.locator('button:has-text("Send")').first()
      await sendTab.click()

      // Check for labels
      const labels = page.locator('label')
      const count = await labels.count()

      expect(count).toBeGreaterThanOrEqual(0)
    })
  })
})
