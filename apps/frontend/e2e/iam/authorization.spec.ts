import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser, logoutUser } from '../helpers/test-users';

test.describe('Authorization Tests', () => {
  test.describe('Role-Based Access Control', () => {
    test('should allow admin to access admin dashboard', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'admin@test.com',
        'password123',
        'admin',
        ['read:all', 'write:all', 'admin:access']
      );

      await loginAsUser(page, 'admin@test.com', 'password123');
      
      await page.goto('/admin');
      await expect(page.locator('[data-testid="admin-dashboard"]')).toBeVisible();
      
      await deleteTestUser(admin.uid);
    });

    test('should prevent user from accessing admin dashboard', async ({ page }) => {
      const user = await createTestUserWithRole(
        'user@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'user@test.com', 'password123');
      
      await page.goto('/admin');
      await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

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

    test('should allow admins to access any user data', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'superadmin@test.com',
        'password123',
        'admin',
        ['read:all', 'write:all']
      );

      await loginAsUser(page, 'superadmin@test.com', 'password123');
      
      await page.goto('/users/any-user-id/data');
      await expect(page.locator('[data-testid="user-data"]')).toBeVisible();
      
      await deleteTestUser(admin.uid);
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
      
      // Simulate permission update (in real app, this would be via admin panel)
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
      
      // Try to access admin API
      const response = await page.request.get('/api/admin/users');
      expect(response.status()).toBe(403);
      
      await deleteTestUser(user.uid);
    });

    test('should allow authorized API access', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'apiadmin@test.com',
        'password123',
        'admin',
        ['read:all']
      );

      await loginAsUser(page, 'apiadmin@test.com', 'password123');
      
      // Should be able to access admin API
      const response = await page.request.get('/api/admin/users');
      expect(response.status()).toBe(200);
      
      await deleteTestUser(admin.uid);
    });
  });
});
