import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser, setupAuthState } from '../helpers/test-users';

test.describe('Admin Authorization Tests', () => {
  test.describe('Admin Dashboard Access', () => {
    test('should allow admin to access admin dashboard', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'admin@test.com',
        'password123',
        'admin',
        ['read:all', 'write:all', 'admin:access']
      );

      await loginAsUser(page, 'admin@test.com', 'password123');
      
      await page.goto('/'); // Admin dashboard is at root of admin-frontend
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
      
      await page.goto('/');
      await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should allow admin to access user management', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'admin-users@test.com',
        'password123',
        'admin',
        ['read:all', 'write:all', 'admin:access', 'manage:users']
      );

      await loginAsUser(page, 'admin-users@test.com', 'password123');
      
      await page.goto('/users');
      await expect(page.locator('[data-testid="user-management"]')).toBeVisible();
      
      await deleteTestUser(admin.uid);
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

  test.describe('Admin API Access', () => {
    test('should protect admin API routes', async ({ page }) => {
      // Test that admin API routes require authentication
      const response = await page.request.get('/api/admin/users');
      expect(response.status()).toBe(401);
    });

    test('should allow admin to access admin API', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'api-admin@test.com',
        'password123',
        'admin',
        ['admin:access', 'manage:users']
      );

      await setupAuthState(page, admin);
      
      const response = await page.request.get('/api/admin/users');
      expect(response.status()).toBe(200);
      
      await deleteTestUser(admin.uid);
    });
  });

  test.describe('Admin Middleware', () => {
    test('should allow admin to access admin routes', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'middleware-admin@test.com',
        'password123',
        'admin',
        ['read:all', 'write:all', 'admin:access']
      );

      await setupAuthState(page, admin);
      
      await page.goto('/admin');
      await expect(page).toHaveURL('/'); // Redirects to main dashboard
      await expect(page.locator('[data-testid="admin-dashboard"]')).toBeVisible();
      
      await deleteTestUser(admin.uid);
    });

    test('should prevent non-admin from accessing admin routes', async ({ page }) => {
      const user = await createTestUserWithRole(
        'regular-user@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await setupAuthState(page, user);
      
      await page.goto('/');
      await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });
});
