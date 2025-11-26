import { Page, expect } from '@playwright/test'

export interface TestNotification {
  title: string
  message: string
  type: 'system' | 'security' | 'permission' | 'wallet' | 'payment' | 'general'
  priority: 'low' | 'normal' | 'high' | 'critical'
  recipient?: string // wallet address or 'all'
  scheduleAt?: string
}

export interface NotificationElement {
  id: string
  title: string
  message: string
  type: string
  priority: string
  read: boolean
}

/**
 * Generate test notification data
 */
export function genTestNotif(overrides?: Partial<TestNotification>): TestNotification {
  return {
    title: 'Test Notification',
    message: 'This is a test notification message',
    type: 'system',
    priority: 'normal',
    ...overrides,
  }
}

/**
 * Generate broadcast notification
 */
export function genBroadcastNotif(overrides?: Partial<TestNotification>): TestNotification {
  return genTestNotif({
    title: 'System Broadcast',
    message: 'Important system announcement',
    recipient: 'all',
    ...overrides,
  })
}

/**
 * Generate high priority notification
 */
export function genHighPriorityNotif(overrides?: Partial<TestNotification>): TestNotification {
  return genTestNotif({
    title: 'Urgent Alert',
    message: 'This requires immediate attention',
    priority: 'high',
    type: 'security',
    ...overrides,
  })
}

/**
 * Wait for notification to appear in bell dropdown
 */
export async function waitForNotifInBell(page: Page, title: string, timeout = 10000) {
  await page.waitForSelector(`text=${title}`, { timeout, state: 'visible' })
}

/**
 * Open notification bell dropdown
 */
export async function openNotifBell(page: Page) {
  const bell = page.locator('button:has(svg), button[aria-label*="notification"]').first()
  await bell.click()
  await page.waitForTimeout(300) // Wait for dropdown animation
}

/**
 * Close notification bell dropdown
 */
export async function closeNotifBell(page: Page) {
  // Click outside the dropdown
  await page.click('body', { position: { x: 0, y: 0 } })
  await page.waitForTimeout(300)
}

/**
 * Get unread count from bell badge
 */
export async function getUnreadCount(page: Page): Promise<number> {
  const badge = page.locator('[data-testid="notification-badge"], .notification-badge, button:has(svg) span').first()

  try {
    const isVisible = await badge.isVisible({ timeout: 2000 })
    if (!isVisible) return 0

    const text = await badge.textContent()
    if (!text) return 0

    const match = text.match(/(\d+)/)
    return match ? parseInt(match[1], 10) : 0
  } catch {
    return 0
  }
}

/**
 * Count notifications in dropdown
 */
export async function countNotifsInDropdown(page: Page): Promise<number> {
  const items = page.locator('[data-testid="notification-item"], .notification-item, [role="menu"] > div, .notification-dropdown > div').all()
  return (await items).length
}

/**
 * Mark notification as read in dropdown
 */
export async function markNotifAsRead(page: Page, title: string) {
  const notif = page.locator(`text=${title}`).first()
  await notif.click()
  await page.waitForTimeout(500)
}

/**
 * Mark all notifications as read
 */
export async function markAllAsRead(page: Page) {
  const btn = page.locator('button:has-text("Mark All as Read"), button:has-text("Mark all")').first()
  await btn.click()
  await page.waitForTimeout(500)
}

/**
 * Navigate to notifications page
 */
export async function goToNotifsPage(page: Page, baseUrl: string) {
  await page.goto(`${baseUrl}/notifications`)
  await page.waitForLoadState('networkidle')
}

/**
 * Apply status filter on notifications page
 */
export async function applyStatusFilter(page: Page, status: 'all' | 'unread' | 'read') {
  const btn = page.locator(`button:has-text("${status}")`, { hasText: new RegExp(status, 'i') }).first()
  await btn.click()
  await page.waitForTimeout(500)
}

/**
 * Apply type filter on notifications page
 */
export async function applyTypeFilter(page: Page, type: string) {
  const select = page.locator('select[name*="type"], select:has(option:has-text("System"))').first()
  await select.selectOption(type)
  await page.waitForTimeout(500)
}

/**
 * Apply priority filter on notifications page
 */
export async function applyPriorityFilter(page: Page, priority: string) {
  const select = page.locator('select[name*="priority"], select:has(option:has-text("Normal"))').first()
  await select.selectOption(priority)
  await page.waitForTimeout(500)
}

/**
 * Delete notification on notifications page
 */
export async function deleteNotif(page: Page, title: string) {
  const notif = page.locator(`text=${title}`).first().locator('..')
  const deleteBtn = notif.locator('button:has(svg[class*="trash"]), button[title*="Delete"]').first()
  await deleteBtn.click()
  await page.waitForTimeout(500)
}

/**
 * Clear all notifications
 */
export async function clearAllNotifs(page: Page) {
  const btn = page.locator('button:has-text("Clear All")').first()
  await btn.click()

  // Handle confirmation dialog
  page.on('dialog', async dialog => {
    await dialog.accept()
  })

  await page.waitForTimeout(500)
}

/**
 * Get notification count on page
 */
export async function getNotifsOnPage(page: Page): Promise<number> {
  const items = page.locator('[data-testid="notification-item"], div[class*="notification"]').all()
  return (await items).length
}

/**
 * Navigate to next page
 */
export async function goToNextPage(page: Page) {
  const btn = page.locator('button:has-text("Next")').first()
  await btn.click()
  await page.waitForLoadState('networkidle')
}

/**
 * Navigate to previous page
 */
export async function goToPrevPage(page: Page) {
  const btn = page.locator('button:has-text("Previous"), button:has-text("Prev")').first()
  await btn.click()
  await page.waitForLoadState('networkidle')
}

/**
 * Verify notification exists on page
 */
export async function verifyNotifExists(page: Page, title: string): Promise<boolean> {
  return await page.locator(`text=${title}`).first().isVisible({ timeout: 5000 }).catch(() => false)
}

/**
 * Verify notification is marked as read
 */
export async function verifyNotifRead(page: Page, title: string): Promise<boolean> {
  const notif = page.locator(`text=${title}`).first().locator('..')
  // Check if read indicator dot is NOT visible
  const unreadDot = notif.locator('div[class*="bg-orange-500"], div[class*="unread"]').first()
  return !(await unreadDot.isVisible({ timeout: 1000 }).catch(() => false))
}

/**
 * Wait for SSE connection
 */
export async function waitForSSEConnection(page: Page, timeout = 10000): Promise<boolean> {
  // Wait for SSE console logs
  return await page.waitForFunction(
    () => {
      return (window as any).__sseConnected === true
    },
    { timeout }
  ).then(() => true).catch(() => false)
}

/**
 * Cleanup test notifications via API
 */
export async function cleanupTestNotifs(apiUrl: string, token: string) {
  try {
    await fetch(`${apiUrl}/api/notifications/clear-all`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
  } catch (error) {
    console.warn('Failed to cleanup notifications:', error)
  }
}

/**
 * Create test notification via admin API
 */
export async function createTestNotifViaAPI(
  apiUrl: string,
  token: string,
  notif: TestNotification
): Promise<string | null> {
  try {
    const response = await fetch(`${apiUrl}/api/admin/notifications/send`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        'X-Admin-Context': 'true',
      },
      body: JSON.stringify({
        title: notif.title,
        message: notif.message,
        notification_type: notif.type,
        priority: notif.priority,
        recipient_wallet_address: notif.recipient !== 'all' ? notif.recipient : undefined,
        broadcast: notif.recipient === 'all',
        schedule_at: notif.scheduleAt,
      }),
    })

    if (!response.ok) {
      console.error('Failed to create notification:', await response.text())
      return null
    }

    const data = await response.json()
    return data.data?.notification_id || null
  } catch (error) {
    console.error('Error creating notification:', error)
    return null
  }
}

/**
 * Assert notification bell count
 */
export async function assertBellCount(page: Page, expected: number) {
  const actual = await getUnreadCount(page)
  expect(actual).toBe(expected)
}

/**
 * Assert notification exists
 */
export async function assertNotifExists(page: Page, title: string) {
  const exists = await verifyNotifExists(page, title)
  expect(exists).toBe(true)
}

/**
 * Assert notification does not exist
 */
export async function assertNotifNotExists(page: Page, title: string) {
  const exists = await verifyNotifExists(page, title)
  expect(exists).toBe(false)
}

/**
 * Assert empty state
 */
export async function assertEmptyState(page: Page) {
  const emptyMsg = page.locator('text=No notifications, text=all caught up').first()
  await expect(emptyMsg).toBeVisible({ timeout: 5000 })
}
