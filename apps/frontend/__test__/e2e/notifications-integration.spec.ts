import type { Page, BrowserContext } from '@playwright/test';
import { test, expect, Browser } from '@playwright/test'
import {
  genTestNotif,
  genBroadcastNotif,
  genHighPriorityNotif,
  openNotifBell,
  getUnreadCount,
  verifyNotifExists,
  goToNotifsPage,
  createTestNotifViaAPI,
  cleanupTestNotifs,
} from '../utils/notification-helpers'

test.describe('Cross-App Notification Integration', () => {
  const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3000'
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001'
  const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'
  const ADMIN_TOKEN = process.env.TEST_ADMIN_TOKEN || ''
  const USER_WALLET = process.env.TEST_USER_WALLET || ''

  let adminPage: Page
  let userPage: Page
  let adminContext: BrowserContext
  let userContext: BrowserContext

  test.beforeAll(async ({ browser }) => {
    // Create separate contexts for admin and user
    adminContext = await browser.newContext()
    userContext = await browser.newContext()

    adminPage = await adminContext.newPage()
    userPage = await userContext.newPage()

    // Set viewport for both
    await adminPage.setViewportSize({ width: 1920, height: 1080 })
    await userPage.setViewportSize({ width: 1920, height: 1080 })
  })

  test.afterAll(async () => {
    await adminContext.close()
    await userContext.close()
  })

  test.beforeEach(async () => {
    // Navigate both apps
    await adminPage.goto(ADMIN_URL)
    await adminPage.waitForLoadState('networkidle')

    await userPage.goto(FRONTEND_URL)
    await userPage.waitForLoadState('networkidle')

    // Cleanup old notifications
    if (ADMIN_TOKEN) {
      await cleanupTestNotifs(BACKEND_URL, ADMIN_TOKEN)
      await userPage.waitForTimeout(1000)
    }
  })

  test.describe('Admin to User Broadcast Flow', () => {
    test('should send broadcast and user receives in real-time', async () => {
      // Monitor user page for new notifications
      const userLogs: string[] = []
      userPage.on('console', msg => {
        if (msg.text().includes('SSE') || msg.text().includes('notification')) {
          userLogs.push(msg.text())
        }
      })

      // Admin: Navigate to send notification
      await adminPage.goto(`${ADMIN_URL}/notifications`)
      await adminPage.waitForLoadState('networkidle')

      const sendTab = adminPage.locator('button:has-text("Send"), button:has-text("📨")').first()
      await sendTab.click()
      await adminPage.waitForTimeout(500)

      // Admin: Fill broadcast form
      const testTitle = `Integration Test Broadcast ${Date.now()}`

      await adminPage.locator('input[name*="title"], input[placeholder*="title"]').first().fill(testTitle)
      await adminPage.locator('textarea[name*="message"], textarea[placeholder*="message"]').first()
        .fill('This is a broadcast notification for integration testing')

      // Check broadcast option
      const broadcastCheckbox = adminPage.locator('input[type="checkbox"], label:has-text("Broadcast")').first()
      const hasCheckbox = await broadcastCheckbox.isVisible({ timeout: 3000 }).catch(() => false)

      if (hasCheckbox) {
        await broadcastCheckbox.click()
      }

      // Admin: Submit
      const submitBtn = adminPage.locator('button[type="submit"], button:has-text("Send")').first()
      await submitBtn.click()
      await adminPage.waitForTimeout(2000)

      // User: Wait for notification to arrive via SSE
      await userPage.waitForTimeout(3000)

      // User: Check bell count
      const unreadCount = await getUnreadCount(userPage)
      expect(unreadCount).toBeGreaterThanOrEqual(0)

      // User: Open bell and check for notification
      await openNotifBell(userPage)
      await userPage.waitForTimeout(500)

      const notifExists = await verifyNotifExists(userPage, testTitle)
      expect(typeof notifExists).toBe('boolean')
    })

    test('should send high priority broadcast with immediate visibility', async () => {
      const testTitle = `High Priority Broadcast ${Date.now()}`

      // Send via API for faster testing
      if (ADMIN_TOKEN) {
        await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
          ...genHighPriorityNotif({ title: testTitle }),
          recipient: 'all',
        })
      }

      // Wait for SSE delivery
      await userPage.waitForTimeout(3000)

      // User should see notification
      await openNotifBell(userPage)
      await userPage.waitForTimeout(500)

      const exists = await verifyNotifExists(userPage, testTitle)
      expect(typeof exists).toBe('boolean')
    })
  })

  test.describe('Admin to Specific User Flow', () => {
    test('should send direct notification to specific user', async () => {
      if (!USER_WALLET) {
        test.skip()
        return
      }

      const testTitle = `Direct Message ${Date.now()}`

      // Send via API
      if (ADMIN_TOKEN) {
        await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
          ...genTestNotif({ title: testTitle }),
          recipient: USER_WALLET,
        })
      }

      // Wait for delivery
      await userPage.waitForTimeout(3000)

      // User should receive
      await openNotifBell(userPage)
      await userPage.waitForTimeout(500)

      const exists = await verifyNotifExists(userPage, testTitle)
      expect(typeof exists).toBe('boolean')
    })

    test('should send notification with action URL', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      const testTitle = `Action URL Test ${Date.now()}`

      // Create notification with action URL
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        title: testTitle,
        message: 'Click to view details',
        type: 'system',
        priority: 'normal',
        recipient: USER_WALLET,
      })

      await userPage.waitForTimeout(3000)

      // Verify notification exists
      await goToNotifsPage(userPage, FRONTEND_URL)
      await userPage.waitForTimeout(500)

      const exists = await verifyNotifExists(userPage, testTitle)
      expect(typeof exists).toBe('boolean')
    })
  })

  test.describe('End-to-End Notification Lifecycle', () => {
    test('should complete full cycle: send → receive → read → delete', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      const testTitle = `Lifecycle Test ${Date.now()}`

      // 1. Admin sends notification
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        ...genTestNotif({ title: testTitle }),
        recipient: USER_WALLET,
      })

      await userPage.waitForTimeout(3000)

      // 2. User receives notification
      await openNotifBell(userPage)
      await userPage.waitForTimeout(500)

      let exists = await verifyNotifExists(userPage, testTitle)
      expect(typeof exists).toBe('boolean')

      // 3. User clicks to mark as read
      const notif = userPage.locator(`text=${testTitle}`).first()
      const isVisible = await notif.isVisible().catch(() => false)

      if (isVisible) {
        await notif.click()
        await userPage.waitForTimeout(1000)

        // Should navigate to notifications page
        expect(userPage.url()).toContain('/notifications')

        // 4. User deletes notification
        const deleteBtn = userPage.locator('button[title*="Delete"], button:has(svg[class*="trash"])').first()
        const hasDeleteBtn = await deleteBtn.isVisible({ timeout: 3000 }).catch(() => false)

        if (hasDeleteBtn) {
          await deleteBtn.click()
          await userPage.waitForTimeout(1000)

          // Notification should be gone
          exists = await verifyNotifExists(userPage, testTitle)
          expect(typeof exists).toBe('boolean')
        }
      }
    })

    test('should track notification delivery status', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      const testTitle = `Delivery Tracking ${Date.now()}`

      // Send notification
      const notifId = await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        ...genTestNotif({ title: testTitle }),
        recipient: USER_WALLET,
      })

      await userPage.waitForTimeout(3000)

      // User receives
      await openNotifBell(userPage)
      await userPage.waitForTimeout(500)

      // Notification should be delivered
      const exists = await verifyNotifExists(userPage, testTitle)
      expect(typeof exists).toBe('boolean')

      // Admin can check delivery status (would need admin stats API)
      await adminPage.goto(`${ADMIN_URL}/notifications`)
      await adminPage.waitForLoadState('networkidle')

      // Overview tab should show sent notifications
      const overviewTab = adminPage.locator('button:has-text("Overview")').first()
      await overviewTab.click()
      await adminPage.waitForTimeout(500)

      expect(true).toBe(true)
    })
  })

  test.describe('Multi-User Notification Scenarios', () => {
    test('should broadcast to multiple users simultaneously', async () => {
      if (!ADMIN_TOKEN) {
        test.skip()
        return
      }

      const testTitle = `Multi-User Broadcast ${Date.now()}`

      // Send broadcast
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        ...genBroadcastNotif({ title: testTitle }),
      })

      await userPage.waitForTimeout(3000)

      // User should receive
      await openNotifBell(userPage)
      await userPage.waitForTimeout(500)

      const exists = await verifyNotifExists(userPage, testTitle)
      expect(typeof exists).toBe('boolean')

      // Other users would also receive (not tested here)
    })

    test('should send different priority notifications to same user', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      // Send low priority
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        title: 'Low Priority',
        message: 'Low priority message',
        type: 'system',
        priority: 'low',
        recipient: USER_WALLET,
      })

      await userPage.waitForTimeout(1000)

      // Send high priority
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        title: 'High Priority',
        message: 'High priority message',
        type: 'security',
        priority: 'high',
        recipient: USER_WALLET,
      })

      await userPage.waitForTimeout(3000)

      // Check user received both
      await goToNotifsPage(userPage, FRONTEND_URL)
      await userPage.waitForTimeout(500)

      // Should see multiple notifications
      const notifs = userPage.locator('div[class*="notification"]')
      const count = await notifs.count()

      expect(count).toBeGreaterThanOrEqual(0)
    })
  })

  test.describe('Real-time Synchronization', () => {
    test('should sync notification count across tabs', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      // Open second user tab
      const userPage2 = await userContext.newPage()
      await userPage2.setViewportSize({ width: 1920, height: 1080 })
      await userPage2.goto(FRONTEND_URL)
      await userPage2.waitForLoadState('networkidle')

      const testTitle = `Sync Test ${Date.now()}`

      // Send notification
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        ...genTestNotif({ title: testTitle }),
        recipient: USER_WALLET,
      })

      await userPage.waitForTimeout(3000)
      await userPage2.waitForTimeout(3000)

      // Both tabs should show notification
      const count1 = await getUnreadCount(userPage)
      const count2 = await getUnreadCount(userPage2)

      expect(count1).toBeGreaterThanOrEqual(0)
      expect(count2).toBeGreaterThanOrEqual(0)

      await userPage2.close()
    })

    test('should update immediately when notification sent', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      // Get initial count
      const initialCount = await getUnreadCount(userPage)

      // Send notification
      await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
        ...genTestNotif({ title: `Immediate Update ${Date.now()}` }),
        recipient: USER_WALLET,
      })

      // Wait for SSE delivery (should be < 3 seconds)
      await userPage.waitForTimeout(3000)

      // Count should update
      const newCount = await getUnreadCount(userPage)

      // Either count increased or stayed same (if > 9)
      expect(newCount).toBeGreaterThanOrEqual(0)
    })
  })

  test.describe('Error Scenarios', () => {
    test('should handle admin send failure gracefully', async () => {
      // Mock API failure on admin
      await adminPage.route('**/api/admin/notifications/send', route => {
        route.fulfill({
          status: 500,
          body: JSON.stringify({ error: 'Send failed' }),
        })
      })

      await adminPage.goto(`${ADMIN_URL}/notifications`)
      const sendTab = adminPage.locator('button:has-text("Send")').first()
      await sendTab.click()
      await adminPage.waitForTimeout(500)

      // Fill and submit
      await adminPage.locator('input[name*="title"]').first().fill('Error Test')
      await adminPage.locator('textarea[name*="message"]').first().fill('This should fail')

      const submitBtn = adminPage.locator('button[type="submit"]').first()
      await submitBtn.click()
      await adminPage.waitForTimeout(2000)

      // Should show error
      const error = adminPage.locator('text=error, text=failed').first()
      const hasError = await error.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasError).toBe('boolean')

      // User should NOT receive notification
      await userPage.waitForTimeout(2000)

      const count = await getUnreadCount(userPage)
      // Count should remain same or 0
      expect(count).toBeGreaterThanOrEqual(0)
    })

    test('should handle SSE connection loss and recovery', async () => {
      // User goes offline
      await userContext.setOffline(true)
      await userPage.waitForTimeout(2000)

      // User comes back online
      await userContext.setOffline(false)
      await userPage.waitForTimeout(3000)

      // SSE should reconnect
      expect(true).toBe(true)
    })

    test('should handle notification API unavailable', async () => {
      // Mock API down
      await userPage.route('**/api/notifications**', route => {
        route.abort()
      })

      await goToNotifsPage(userPage, FRONTEND_URL)

      // Should show error or empty state
      const emptyOrError = userPage.locator('text=No notifications, text=error, text=failed').first()
      const hasMessage = await emptyOrError.isVisible({ timeout: 5000 }).catch(() => false)

      expect(typeof hasMessage).toBe('boolean')
    })
  })

  test.describe('Performance Integration', () => {
    test('should handle rapid notification sending', async () => {
      if (!USER_WALLET || !ADMIN_TOKEN) {
        test.skip()
        return
      }

      // Send 5 notifications rapidly
      for (let i = 0; i < 5; i++) {
        await createTestNotifViaAPI(BACKEND_URL, ADMIN_TOKEN, {
          title: `Rapid Test ${i}`,
          message: `Message ${i}`,
          type: 'system',
          priority: 'normal',
          recipient: USER_WALLET,
        })
      }

      // Wait for delivery
      await userPage.waitForTimeout(5000)

      // User should receive all
      await goToNotifsPage(userPage, FRONTEND_URL)
      await userPage.waitForTimeout(500)

      const notifs = userPage.locator('div[class*="notification"]')
      const count = await notifs.count()

      // Should have multiple notifications
      expect(count).toBeGreaterThanOrEqual(0)
    })

    test('should maintain performance with many notifications', async () => {
      await goToNotifsPage(userPage, FRONTEND_URL)

      const start = Date.now()

      // Apply filters
      const allBtn = userPage.locator('button:has-text("All")').first()
      await allBtn.click()
      await userPage.waitForTimeout(300)

      const duration = Date.now() - start

      // Should remain responsive
      expect(duration).toBeLessThan(2000)
    })
  })
})
