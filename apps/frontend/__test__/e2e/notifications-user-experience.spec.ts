/**
 * User-Side Notification System E2E Tests
 * 
 * Tests notification functionality from the user perspective:
 * - Receiving notifications sent by admins
 * - Notification display in user frontend
 * - Notification interactions and management
 * - Cross-platform notification experience
 */
import { test, expect, Page } from '@playwright/test';

// Test configuration
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:3000';
const ADMIN_FRONTEND_URL = process.env.ADMIN_FRONTEND_URL || 'http://localhost:3001';
const TEST_USER_EMAIL = process.env.TEST_USER_EMAIL || 'user@example.com';
const TEST_USER_PASSWORD = process.env.TEST_USER_PASSWORD || 'user123';
const ADMIN_EMAIL = process.env.ADMIN_EMAIL || 'admin@example.com';
const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || 'admin123';

interface NotificationData {
  title: string;
  message: string;
  type: 'broadcast' | 'direct' | 'system';
  priority?: 'low' | 'normal' | 'high' | 'critical';
}

// Test helper for user notification experience
class UserNotificationHelper {
  constructor(private page: Page) {}

  async loginAsUser() {
    await this.page.goto(`${FRONTEND_URL}/login`);
    await this.page.fill('[data-testid="email-input"]', TEST_USER_EMAIL);
    await this.page.fill('[data-testid="password-input"]', TEST_USER_PASSWORD);
    await this.page.click('[data-testid="login-button"]');
    
    // Wait for successful login
    await this.page.waitForURL(/\/dashboard/);
    await expect(this.page.locator('text=Dashboard')).toBeVisible();
  }

  async checkNotificationBell() {
    const bell = this.page.locator('[data-testid="notification-bell"]');
    await expect(bell).toBeVisible();
    return bell;
  }

  async getUnreadNotificationCount(): Promise<number> {
    const badge = this.page.locator('[data-testid="unread-count-badge"]');
    if (await badge.isVisible()) {
      const count = await badge.textContent();
      return parseInt(count || '0');
    }
    return 0;
  }

  async openNotificationDropdown() {
    const bell = await this.checkNotificationBell();
    await bell.click();
    
    // Wait for dropdown to appear
    await expect(this.page.locator('[data-testid="notification-dropdown"]')).toBeVisible();
  }

  async verifyNotificationInDropdown(notification: NotificationData) {
    await this.openNotificationDropdown();
    
    const notificationItem = this.page.locator(
      `[data-testid="notification-item"]:has-text("${notification.title}")`
    );
    
    await expect(notificationItem).toBeVisible();
    await expect(notificationItem.locator(`text=${notification.message}`)).toBeVisible();
    
    // Check priority indicator if applicable
    if (notification.priority && notification.priority !== 'normal') {
      const priorityIndicator = notificationItem.locator(`[data-priority="${notification.priority}"]`);
      await expect(priorityIndicator).toBeVisible();
    }
    
    return notificationItem;
  }

  async markNotificationAsRead(title: string) {
    await this.openNotificationDropdown();
    
    const notificationItem = this.page.locator(`[data-testid="notification-item"]:has-text("${title}")`);
    const markReadButton = notificationItem.locator('[data-testid="mark-read-button"]');
    
    if (await markReadButton.isVisible()) {
      await markReadButton.click();
      
      // Wait for read status to update
      await this.page.waitForTimeout(1000);
    }
  }

  async markAllNotificationsAsRead() {
    await this.openNotificationDropdown();
    
    const markAllButton = this.page.locator('[data-testid="mark-all-read-button"]');
    if (await markAllButton.isVisible()) {
      await markAllButton.click();
      
      // Wait for status update
      await this.page.waitForTimeout(1000);
    }
  }

  async navigateToNotificationsPage() {
    await this.page.click('[data-testid="view-all-notifications-link"]');
    await this.page.waitForURL(/\/notifications/);
    await expect(this.page.locator('text=Notifications')).toBeVisible();
  }

  async verifyNotificationInPage(notification: NotificationData) {
    await this.navigateToNotificationsPage();
    
    const notificationCard = this.page.locator(
      `[data-testid="notification-card"]:has-text("${notification.title}")`
    );
    
    await expect(notificationCard).toBeVisible();
    await expect(notificationCard.locator(`text=${notification.message}`)).toBeVisible();
    
    return notificationCard;
  }
}

// Admin helper to send notifications for testing
class AdminTestHelper {
  constructor(private page: Page) {}

  async loginAsAdmin() {
    await this.page.goto(`${ADMIN_FRONTEND_URL}/login`);
    await this.page.fill('[data-testid="email-input"]', ADMIN_EMAIL);
    await this.page.fill('[data-testid="password-input"]', ADMIN_PASSWORD);
    await this.page.click('[data-testid="login-button"]');
    
    await this.page.waitForURL(`${ADMIN_FRONTEND_URL}/dashboard`);
    await expect(this.page.locator('text=Admin Dashboard')).toBeVisible();
  }

  async sendNotificationToUser(notification: NotificationData, targetUserId?: string) {
    await this.page.goto(`${ADMIN_FRONTEND_URL}/notifications`);
    
    await this.page.click('[data-testid="create-notification-button"]');
    
    await this.page.fill('[data-testid="notification-title"]', notification.title);
    await this.page.fill('[data-testid="notification-message"]', notification.message);
    await this.page.selectOption('[data-testid="notification-type"]', notification.type);
    
    if (notification.priority) {
      await this.page.selectOption('[data-testid="notification-priority"]', notification.priority);
    }
    
    if (notification.type === 'direct' && targetUserId) {
      await this.page.fill('[data-testid="target-user-id"]', targetUserId);
    }
    
    await this.page.click('[data-testid="send-notification-button"]');
    
    // Wait for success confirmation
    await expect(this.page.locator('[data-testid="success-toast"]')).toBeVisible();
  }
}

test.describe('User Notification Experience Tests', () => {
  let userHelper: UserNotificationHelper;
  let adminHelper: AdminTestHelper;
  let userPage: Page;
  let adminPage: Page;

  test.beforeEach(async ({ browser }) => {
    // Create separate contexts for user and admin
    const userContext = await browser.newContext();
    const adminContext = await browser.newContext();
    
    userPage = await userContext.newPage();
    adminPage = await adminContext.newPage();
    
    userHelper = new UserNotificationHelper(userPage);
    adminHelper = new AdminTestHelper(adminPage);
    
    // Login both users
    await userHelper.loginAsUser();
    await adminHelper.loginAsAdmin();
  });

  test('User receives and displays broadcast notifications', async () => {
    const notification: NotificationData = {
      title: 'System Update',
      message: 'Important system update scheduled for maintenance',
      type: 'broadcast',
      priority: 'high'
    };

    // Admin sends broadcast notification
    await adminHelper.sendNotificationToUser(notification);
    
    // Navigate to user dashboard and wait for notification
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000); // Allow time for notification to arrive
    
    // Check notification appears in bell
    const unreadCount = await userHelper.getUnreadNotificationCount();
    expect(unreadCount).toBeGreaterThan(0);
    
    // Verify notification in dropdown
    await userHelper.verifyNotificationInDropdown(notification);
  });

  test('User receives direct notifications targeted specifically to them', async () => {
    const notification: NotificationData = {
      title: 'Personal Message',
      message: 'This message is specifically for you',
      type: 'direct',
      priority: 'normal'
    };

    // Admin sends direct notification to test user
    await adminHelper.sendNotificationToUser(notification, TEST_USER_EMAIL);
    
    // Check user receives the notification
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000);
    
    // Verify notification appears
    await userHelper.verifyNotificationInDropdown(notification);
  });

  test('User can mark notifications as read', async () => {
    const notification: NotificationData = {
      title: 'Mark Read Test',
      message: 'Testing mark as read functionality',
      type: 'broadcast'
    };

    // Send notification
    await adminHelper.sendNotificationToUser(notification);
    
    // User checks notification
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000);
    
    // Get initial unread count
    const initialCount = await userHelper.getUnreadNotificationCount();
    expect(initialCount).toBeGreaterThan(0);
    
    // Mark notification as read
    await userHelper.markNotificationAsRead(notification.title);
    
    // Verify unread count decreased
    const newCount = await userHelper.getUnreadNotificationCount();
    expect(newCount).toBeLessThan(initialCount);
  });

  test('User can mark all notifications as read', async () => {
    // Send multiple notifications
    const notifications: NotificationData[] = [
      { title: 'Notification 1', message: 'First test notification', type: 'broadcast' },
      { title: 'Notification 2', message: 'Second test notification', type: 'broadcast' },
      { title: 'Notification 3', message: 'Third test notification', type: 'broadcast' }
    ];

    for (const notification of notifications) {
      await adminHelper.sendNotificationToUser(notification);
      await userPage.waitForTimeout(1000); // Brief pause between notifications
    }
    
    // Check notifications appear
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(3000);
    
    const initialCount = await userHelper.getUnreadNotificationCount();
    expect(initialCount).toBeGreaterThanOrEqual(3);
    
    // Mark all as read
    await userHelper.markAllNotificationsAsRead();
    
    // Verify all are marked as read
    const finalCount = await userHelper.getUnreadNotificationCount();
    expect(finalCount).toBe(0);
  });

  test('Notifications display correctly in dedicated notifications page', async () => {
    const notification: NotificationData = {
      title: 'Page Display Test',
      message: 'Testing notification display on dedicated page',
      type: 'broadcast',
      priority: 'normal'
    };

    await adminHelper.sendNotificationToUser(notification);
    
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000);
    
    // Navigate to notifications page
    await userHelper.navigateToNotificationsPage();
    
    // Verify notification appears on page
    await userHelper.verifyNotificationInPage(notification);
  });

  test('High priority notifications are visually distinct', async () => {
    const highPriorityNotification: NotificationData = {
      title: 'URGENT: Security Alert',
      message: 'Important security notification requiring immediate attention',
      type: 'broadcast',
      priority: 'critical'
    };

    await adminHelper.sendNotificationToUser(highPriorityNotification);
    
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000);
    
    await userHelper.openNotificationDropdown();
    
    // Check for critical priority styling
    const notificationItem = userPage.locator(
      `[data-testid="notification-item"]:has-text("${highPriorityNotification.title}")`
    );
    
    await expect(notificationItem).toBeVisible();
    await expect(notificationItem.locator('[data-priority="critical"]')).toBeVisible();
  });

  test('Notification interactions work correctly', async () => {
    const notification: NotificationData = {
      title: 'Interactive Test',
      message: 'Testing notification interactions',
      type: 'broadcast'
    };

    await adminHelper.sendNotificationToUser(notification);
    
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000);
    
    await userHelper.openNotificationDropdown();
    
    // Test click to close dropdown
    await userPage.click('body');
    await expect(userPage.locator('[data-testid="notification-dropdown"]')).not.toBeVisible();
    
    // Test reopening dropdown
    await userHelper.openNotificationDropdown();
    await expect(userPage.locator('[data-testid="notification-dropdown"]')).toBeVisible();
  });

  test('Notifications persist across browser sessions', async () => {
    const notification: NotificationData = {
      title: 'Persistence Test',
      message: 'Testing notification persistence',
      type: 'broadcast'
    };

    await adminHelper.sendNotificationToUser(notification);
    
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    await userPage.waitForTimeout(2000);
    
    // Verify notification is visible
    await userHelper.verifyNotificationInDropdown(notification);
    
    // Simulate browser refresh
    await userPage.reload();
    await userPage.waitForLoadState('networkidle');
    
    // Verify notification still appears after refresh
    const unreadCount = await userHelper.getUnreadNotificationCount();
    expect(unreadCount).toBeGreaterThan(0);
    
    await userHelper.verifyNotificationInDropdown(notification);
  });

  test('Empty notification state displays correctly', async () => {
    await userPage.goto(`${FRONTEND_URL}/dashboard`);
    
    // Clear all notifications first
    const initialCount = await userHelper.getUnreadNotificationCount();
    if (initialCount > 0) {
      await userHelper.markAllNotificationsAsRead();
    }
    
    await userHelper.openNotificationDropdown();
    
    // Verify empty state message
    const emptyState = userPage.locator('[data-testid="no-notifications-message"]');
    await expect(emptyState).toBeVisible();
    await expect(emptyState).toContainText('No notifications');
  });
});

// Mobile responsiveness tests
test.describe('Mobile Notification Experience', () => {
  test('Notifications work properly on mobile devices', async ({ browser }) => {
    const context = await browser.newContext({
      viewport: { width: 375, height: 667 }, // iPhone SE dimensions
      userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1'
    });

    const page = await context.newPage();
    const helper = new UserNotificationHelper(page);
    
    // Create admin context for sending notifications
    const adminContext = await browser.newContext();
    const adminPage = await adminContext.newPage();
    const adminHelper = new AdminTestHelper(adminPage);
    
    await helper.loginAsUser();
    await adminHelper.loginAsAdmin();
    
    // Send test notification
    const notification: NotificationData = {
      title: 'Mobile Test',
      message: 'Testing mobile notification experience',
      type: 'broadcast'
    };
    
    await adminHelper.sendNotificationToUser(notification);
    
    // Check mobile notification experience
    await page.goto(`${FRONTEND_URL}/dashboard`);
    await page.waitForTimeout(2000);
    
    // Verify notification bell is visible and accessible on mobile
    const bell = await helper.checkNotificationBell();
    
    // Check touch-friendly size (at least 44x44px)
    const boundingBox = await bell.boundingBox();
    expect(boundingBox?.width).toBeGreaterThanOrEqual(44);
    expect(boundingBox?.height).toBeGreaterThanOrEqual(44);
    
    // Test dropdown functionality on mobile
    await helper.verifyNotificationInDropdown(notification);
    
    await context.close();
    await adminContext.close();
  });
});

// Performance tests for user notifications
test.describe('User Notification Performance', () => {
  test('Notification loading and display performance', async ({ page }) => {
    const helper = new UserNotificationHelper(page);
    await helper.loginAsUser();
    
    // Test dashboard load time with notifications
    const startTime = Date.now();
    await page.goto(`${FRONTEND_URL}/dashboard`);
    await page.waitForLoadState('networkidle');
    const loadTime = Date.now() - startTime;
    
    // Dashboard should load within 5 seconds even with notifications
    expect(loadTime).toBeLessThan(5000);
    
    // Test notification dropdown responsiveness
    const dropdownStart = Date.now();
    await helper.openNotificationDropdown();
    const dropdownTime = Date.now() - dropdownStart;
    
    // Dropdown should open within 1 second
    expect(dropdownTime).toBeLessThan(1000);
  });
});

// Accessibility tests
test.describe('Notification Accessibility', () => {
  test('Notifications are accessible via keyboard navigation', async ({ page }) => {
    const helper = new UserNotificationHelper(page);
    await helper.loginAsUser();
    
    await page.goto(`${FRONTEND_URL}/dashboard`);
    
    // Test keyboard navigation to notification bell
    await page.press('body', 'Tab');
    // Continue tabbing until notification bell is focused
    let focused = await page.locator(':focus').getAttribute('data-testid');
    while (focused !== 'notification-bell' && focused !== null) {
      await page.press('body', 'Tab');
      focused = await page.locator(':focus').getAttribute('data-testid');
      
      // Prevent infinite loop
      if (!focused) break;
    }
    
    // Verify notification bell is focusable
    const focusedElement = page.locator(':focus');
    expect(await focusedElement.getAttribute('data-testid')).toBe('notification-bell');
    
    // Test opening dropdown with keyboard
    await page.press('body', 'Enter');
    await expect(page.locator('[data-testid="notification-dropdown"]')).toBeVisible();
  });

  test('Notifications have proper ARIA labels', async ({ page }) => {
    const helper = new UserNotificationHelper(page);
    await helper.loginAsUser();
    
    await page.goto(`${FRONTEND_URL}/dashboard`);
    
    // Check notification bell ARIA attributes
    const bell = page.locator('[data-testid="notification-bell"]');
    
    await expect(bell).toHaveAttribute('role', 'button');
    await expect(bell).toHaveAttribute('aria-label', /.*(notification|bell).*/i);
    
    // Check unread count accessibility
    const badge = page.locator('[data-testid="unread-count-badge"]');
    if (await badge.isVisible()) {
      await expect(badge).toHaveAttribute('aria-label', /.*(unread|notification).*/i);
    }
  });
});