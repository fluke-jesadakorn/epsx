import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser, logoutUser } from '../helpers/test-users';

test.describe('Authorization Tests', () => {
  test.describe('Role-Based Access Control', () => {
    test('should allow moderator to access moderation tools', async ({ page }) => {
      const moderator = await createTestUserWithRole(
        'moderator@test.com',
        'password123',
        'moderator',
        ['read:all', 'write:moderated', 'moderate:content']
      );

      await loginAsUser(page, 'moderator@test.com', 'password123');
      
      await page.goto('/moderate');
      await expect(page.locator('[data-testid="moderation-panel"]')).toBeVisible();
      
      await deleteTestUser(moderator.uid);
    });
  });

  test.describe('Permission-Based Access Control', () => {
    test('should allow access based on specific permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'analytics@test.com',
        'password123',
        'user',
        ['read:analytics', 'read:own_data']
      );

      await loginAsUser(page, 'analytics@test.com', 'password123');
      
      await page.goto('/analytics');
      await expect(page.locator('[data-testid="analytics-dashboard"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should deny access without required permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'basic@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'basic@test.com', 'password123');
      
      await page.goto('/analytics');
      await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should allow write operations with write permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'writer@test.com',
        'password123',
        'user',
        ['read:own_data', 'write:own_data']
      );

      await loginAsUser(page, 'writer@test.com', 'password123');
      
      await page.goto('/profile');
      await page.fill('input[name="displayName"]', 'Updated Name');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should prevent write operations without write permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'readonly@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'readonly@test.com', 'password123');
      
      await page.goto('/profile');
      await expect(page.locator('input[name="displayName"]')).toBeDisabled();
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Resource-Level Permissions', () => {
    test('should allow users to access their own data', async ({ page }) => {
      const user = await createTestUserWithRole(
        'owner@test.com',
        'password123',
        'user',
        ['read:own_data', 'write:own_data']
      );

      await loginAsUser(page, 'owner@test.com', 'password123');
      
      await page.goto('/my-data');
      await expect(page.locator('[data-testid="own-data"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should prevent users from accessing others data', async ({ page }) => {
      const user = await createTestUserWithRole(
        'user1@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'user1@test.com', 'password123');
      
      await page.goto('/users/other-user-id/data');
      await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Dynamic Permission Updates', () => {
    test('should reflect permission changes immediately', async ({ page }) => {
      const user = await createTestUserWithRole(
        'dynamic@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'dynamic@test.com', 'password123');
      
      // Initially cannot access analytics
      await page.goto('/analytics');
      await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
      
      // Simulate permission update (in real app, this would be via system panel)
      await page.evaluate(() => {
        sessionStorage.setItem('updated-permissions', JSON.stringify(['read:own_data', 'read:analytics']));
      });
      
      await page.reload();
      
      // Now should have access
      await page.goto('/analytics');
      await expect(page.locator('[data-testid="analytics-dashboard"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Permission Inheritance', () => {
    test('should inherit permissions from role', async ({ page }) => {
      const user = await createTestUserWithRole(
        'inherit@test.com',
        'password123',
        'premium_user',
        ['read:own_data', 'read:premium_content']
      );

      await loginAsUser(page, 'inherit@test.com', 'password123');
      
      await page.goto('/premium');
      await expect(page.locator('[data-testid="premium-content"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('API Endpoint Authorization', () => {
    test('should protect API endpoints based on permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'api@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'api@test.com', 'password123');
      
      // Try to access protected API
      const response = await page.request.get('/api/protected/users');
      expect(response.status()).toBe(403);
      
      await deleteTestUser(user.uid);
    });

    test('should allow authorized API access', async ({ page }) => {
      const moderator = await createTestUserWithRole(
        'apimoderator@test.com',
        'password123',
        'moderator',
        ['read:all', 'moderate:content']
      );

      await loginAsUser(page, 'apimoderator@test.com', 'password123');
      
      // Should be able to access moderator API
      const response = await page.request.get('/api/moderator/content');
      expect(response.status()).toBe(200);
      
      await deleteTestUser(moderator.uid);
    });
  });
});
