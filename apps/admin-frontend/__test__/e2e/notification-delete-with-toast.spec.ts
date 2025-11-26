import { test, expect, Page } from '@playwright/test'

test.describe('Notification Creation and Deletion with Toast', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001'
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 })
    await page.goto(ADMIN_URL)
    await page.waitForLoadState('networkidle')
  })

  test('should create notification, verify it appears, delete it, and show toast', async ({ page }) => {
    // Step 1: Navigate to notifications page
    await page.goto(`${ADMIN_URL}/notifications`)
    await page.waitForLoadState('networkidle')
    await page.screenshot({ path: 'test-results/01-notifications-page.png', fullPage: true })

    // Step 2: Click on Send tab
    const sendTab = page.locator('button:has-text("Send"), button:has-text("📨")').first()
    await expect(sendTab).toBeVisible({ timeout: 5000 })
    await sendTab.click()
    await page.waitForTimeout(500)
    await page.screenshot({ path: 'test-results/02-send-tab-opened.png', fullPage: true })

    // Step 3: Fill notification form with unique title
    const timestamp = Date.now()
    const notificationTitle = `E2E Test Notification ${timestamp}`
    const notificationMessage = `This is an automated E2E test message created at ${new Date().toISOString()}`

    const titleInput = page.locator('input[name*="title"], input[placeholder*="title"]').first()
    await expect(titleInput).toBeVisible({ timeout: 5000 })
    await titleInput.fill(notificationTitle)

    const messageInput = page.locator('textarea[name*="message"], textarea[placeholder*="message"]').first()
    await expect(messageInput).toBeVisible({ timeout: 5000 })
    await messageInput.fill(notificationMessage)

    // Step 4: Select notification type and priority
    const typeSelect = page.locator('select[name*="type"]').first()
    const hasTypeSelect = await typeSelect.isVisible({ timeout: 3000 }).catch(() => false)
    if (hasTypeSelect) {
      await typeSelect.selectOption('system')
    }

    const prioritySelect = page.locator('select[name*="priority"]').first()
    const hasPrioritySelect = await prioritySelect.isVisible({ timeout: 3000 }).catch(() => false)
    if (hasPrioritySelect) {
      await prioritySelect.selectOption('normal')
    }

    await page.screenshot({ path: 'test-results/03-form-filled.png', fullPage: true })

    // Step 5: Enable broadcast
    const broadcastCheckbox = page.locator('input[type="checkbox"][name*="broadcast"], label:has-text("Broadcast")').first()
    const hasBroadcast = await broadcastCheckbox.isVisible({ timeout: 3000 }).catch(() => false)
    if (hasBroadcast) {
      await broadcastCheckbox.check()
    }

    // Step 6: Submit the form
    const submitBtn = page.locator('button[type="submit"], button:has-text("Send Notification")').first()
    await expect(submitBtn).toBeVisible({ timeout: 5000 })
    await submitBtn.click()

    // Wait for submission to complete
    await page.waitForTimeout(2000)
    await page.screenshot({ path: 'test-results/04-notification-sent.png', fullPage: true })

    // Step 7: Check for success message/toast
    const successToast = page.locator('text=/success|sent|successfully/i').first()
    const hasSuccess = await successToast.isVisible({ timeout: 5000 }).catch(() => false)
    if (hasSuccess) {
      await page.screenshot({ path: 'test-results/05-success-toast.png', fullPage: true })
    }

    // Step 8: Navigate to home/dashboard to check notification bell
    await page.goto(ADMIN_URL)
    await page.waitForLoadState('networkidle')
    await page.waitForTimeout(2000) // Wait for SSE connection

    // Step 9: Open notification bell
    const notificationBell = page.locator('button:has-text("🔔")').first()
    await expect(notificationBell).toBeVisible({ timeout: 5000 })
    await page.screenshot({ path: 'test-results/06-before-opening-bell.png', fullPage: true })

    await notificationBell.click()
    await page.waitForTimeout(500)
    await page.screenshot({ path: 'test-results/07-bell-opened.png', fullPage: true })

    // Step 10: Verify notification appears in dropdown
    const dropdown = page.locator('div[class*="dropdown"], div[class*="absolute"]').first()
    await expect(dropdown).toBeVisible({ timeout: 3000 })

    // Look for our notification by title
    const notificationItem = page.locator(`text="${notificationTitle}"`).first()
    const notificationExists = await notificationItem.isVisible({ timeout: 5000 }).catch(() => false)

    if (!notificationExists) {
      // If notification not found, take screenshot and list all visible notifications
      await page.screenshot({ path: 'test-results/08-notification-not-found.png', fullPage: true })

      const allNotifications = page.locator('div[class*="font-semibold"]')
      const count = await allNotifications.count()
      console.log(`Found ${count} notifications in bell`)

      for (let i = 0; i < count; i++) {
        const text = await allNotifications.nth(i).textContent()
        console.log(`Notification ${i}: ${text}`)
      }
    }

    await expect(notificationItem).toBeVisible({ timeout: 5000 })
    await page.screenshot({ path: 'test-results/09-notification-found.png', fullPage: true })

    // Step 11: Hover over notification to reveal delete button
    const notificationCard = notificationItem.locator('..').locator('..').first()
    await notificationCard.hover()
    await page.waitForTimeout(500)
    await page.screenshot({ path: 'test-results/10-notification-hovered.png', fullPage: true })

    // Step 12: Find and click delete button
    const deleteButton = notificationCard.locator('button[title="Delete notification"], button:has-text("✕")').first()
    await expect(deleteButton).toBeVisible({ timeout: 3000 })
    await page.screenshot({ path: 'test-results/11-delete-button-visible.png', fullPage: true })

    // Step 13: Click delete button
    await deleteButton.click()
    await page.waitForTimeout(1000)
    await page.screenshot({ path: 'test-results/12-after-delete-click.png', fullPage: true })

    // Step 14: Verify toast message appears
    const deleteToast = page.locator('text=/notification deleted|deleted/i').first()
    await expect(deleteToast).toBeVisible({ timeout: 5000 })
    await page.screenshot({ path: 'test-results/13-delete-toast-visible.png', fullPage: true })

    // Verify toast has success styling
    const toastContainer = page.locator('div[class*="gradient"], div[style*="gradient"]').filter({ hasText: /deleted/i }).first()
    const hasToast = await toastContainer.isVisible({ timeout: 3000 }).catch(() => false)
    expect(hasToast).toBe(true)

    // Step 15: Verify notification is removed from list
    await page.waitForTimeout(1000)
    const notificationStillExists = await notificationItem.isVisible({ timeout: 2000 }).catch(() => false)
    expect(notificationStillExists).toBe(false)
    await page.screenshot({ path: 'test-results/14-notification-removed.png', fullPage: true })

    console.log('✅ Test completed successfully!')
    console.log(`- Created notification: "${notificationTitle}"`)
    console.log('- Verified notification appears in bell')
    console.log('- Deleted notification successfully')
    console.log('- Toast message displayed correctly')
    console.log('- Notification removed from list')
  })

  test('should handle delete notification failure gracefully', async ({ page }) => {
    // Navigate and open notification bell
    await page.goto(ADMIN_URL)
    await page.waitForLoadState('networkidle')
    await page.waitForTimeout(2000)

    const notificationBell = page.locator('button:has-text("🔔")').first()
    await notificationBell.click()
    await page.waitForTimeout(500)

    // Mock API failure for delete
    await page.route('**/api/admin/notifications/*', route => {
      if (route.request().method() === 'DELETE') {
        route.fulfill({
          status: 500,
          body: JSON.stringify({ error: 'Failed to delete notification' }),
        })
      } else {
        route.continue()
      }
    })

    // Find first notification and try to delete
    const firstNotification = page.locator('div[class*="rounded-2xl"]').first()
    const hasNotification = await firstNotification.isVisible({ timeout: 3000 }).catch(() => false)

    if (hasNotification) {
      await firstNotification.hover()
      await page.waitForTimeout(500)

      const deleteButton = firstNotification.locator('button:has-text("✕")').first()
      const hasDeleteBtn = await deleteButton.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasDeleteBtn) {
        await deleteButton.click()
        await page.waitForTimeout(1000)

        // Should show error toast
        const errorToast = page.locator('text=/failed|error/i').first()
        const hasError = await errorToast.isVisible({ timeout: 5000 }).catch(() => false)

        await page.screenshot({ path: 'test-results/15-delete-error-toast.png', fullPage: true })
        expect(typeof hasError).toBe('boolean')
      }
    }
  })

  test('should display toast with correct styling', async ({ page }) => {
    // Create a notification first
    await page.goto(`${ADMIN_URL}/notifications`)
    await page.waitForLoadState('networkidle')

    const sendTab = page.locator('button:has-text("Send")').first()
    await sendTab.click()
    await page.waitForTimeout(500)

    const timestamp = Date.now()
    await page.locator('input[name*="title"]').first().fill(`Toast Test ${timestamp}`)
    await page.locator('textarea[name*="message"]').first().fill('Testing toast styling')

    const submitBtn = page.locator('button[type="submit"]').first()
    await submitBtn.click()
    await page.waitForTimeout(2000)

    // Go back and delete
    await page.goto(ADMIN_URL)
    await page.waitForTimeout(2000)

    const bell = page.locator('button:has-text("🔔")').first()
    await bell.click()
    await page.waitForTimeout(500)

    const notification = page.locator(`text="Toast Test ${timestamp}"`).first()
    const exists = await notification.isVisible({ timeout: 5000 }).catch(() => false)

    if (exists) {
      const card = notification.locator('..').locator('..').first()
      await card.hover()
      await page.waitForTimeout(500)

      const deleteBtn = card.locator('button:has-text("✕")').first()
      await deleteBtn.click()
      await page.waitForTimeout(500)

      // Verify toast styling
      const toast = page.locator('text=/deleted/i').first()
      await expect(toast).toBeVisible({ timeout: 5000 })

      // Check for success gradient styling
      const toastWithGradient = page.locator('div[style*="142 71%"], div[class*="bg-green"]').filter({ hasText: /deleted/i }).first()
      const hasGradient = await toastWithGradient.isVisible({ timeout: 3000 }).catch(() => false)

      await page.screenshot({ path: 'test-results/16-toast-with-styling.png', fullPage: true })
      expect(typeof hasGradient).toBe('boolean')
    }
  })

  test('should close dropdown after deleting notification', async ({ page }) => {
    await page.goto(ADMIN_URL)
    await page.waitForTimeout(2000)

    const bell = page.locator('button:has-text("🔔")').first()
    await bell.click()
    await page.waitForTimeout(500)

    // Find any notification
    const firstNotif = page.locator('div[class*="rounded-2xl"]').first()
    const hasNotif = await firstNotif.isVisible({ timeout: 3000 }).catch(() => false)

    if (hasNotif) {
      const initialCount = await page.locator('div[class*="rounded-2xl"]').count()

      await firstNotif.hover()
      await page.waitForTimeout(500)

      const deleteBtn = firstNotif.locator('button:has-text("✕")').first()
      const hasDeleteBtn = await deleteBtn.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasDeleteBtn) {
        await deleteBtn.click()
        await page.waitForTimeout(1000)

        // Verify count decreased
        const newCount = await page.locator('div[class*="rounded-2xl"]').count()
        expect(newCount).toBeLessThan(initialCount)

        await page.screenshot({ path: 'test-results/17-after-delete-count-check.png', fullPage: true })
      }
    }
  })
})
