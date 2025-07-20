import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser, setupAuthState, clearAuthState } from '../helpers/test-users';

test.describe('Middleware Authorization Tests', () => {
  test.describe('Route Protection', () => {
    test('should redirect unauthenticated users to login', async ({ page }) => {
      await clearAuthState(page);
      
      await page.goto('/dashboard');
      
      await expect(page).toHaveURL('/login?redirect=/dashboard');
      await expect(page.locator('[data-testid="login-form"]')).toBeVisible();
    });

    test('should allow authenticated users to access protected routes', async ({ page }) => {
      const user = await createTestUserWithRole(
        'auth@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await setupAuthState(page, user);
      
      await page.goto('/dashboard');
      await expect(page).toHaveURL('/dashboard');
      await expect(page.locator('[data-testid="dashboard-content"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should redirect to intended page after login', async ({ page }) => {
      const user = await createTestUserWithRole(
        'redirect@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      // Try to access protected route
      await page.goto('/analytics');
      await expect(page).toHaveURL('/login?redirect=/analytics');
      
      // Login
      await page.fill('input[type="email"]', 'redirect@test.com');
      await page.fill('input[type="password"]', 'password123');
      await page.click('button[type="submit"]');
      
      // Should redirect to originally intended page
      await page.waitForURL('/analytics');
      await expect(page.locator('[data-testid="analytics-dashboard"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Role-Based Route Guards', () => {
    test('should allow admin to access admin routes', async ({ page }) => {
      const admin = await createTestUserWithRole(
        'admin@test.com',
        'password123',
        'admin',
        ['read:all', 'write:all', 'admin:access']
      );

      await setupAuthState(page, admin);
      
      await page.goto('/admin');
      await expect(page).toHaveURL('/admin');
      await expect(page.locator('[data-testid="admin-dashboard"]')).toBeVisible();
      
      await deleteTestUser(admin.uid);
    });

    test('should prevent non-admin from accessing admin routes', async ({ page }) => {
      const user = await createTestUserWithRole(
        'regular@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await setupAuthState(page, user);
      
      await page.goto('/admin');
      await expect(page).toHaveURL('/unauthorized');
      await expect(page.locator('[data-testid="unauthorized-message"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should allow moderator to access moderation routes', async ({ page }) => {
      const moderator = await createTestUserWithRole(
        'moderator@test.com',
        'password123',
        'moderator',
        ['read:all', 'write:moderated', 'moderate:content']
      );

      await setupAuthState(page, moderator);
      
      await page.goto('/moderate');
      await expect(page).toHaveURL('/moderate');
      await expect(page.locator('[data-testid="moderation-panel"]')).toBeVisible();
      
      await deleteTestUser(moderator.uid);
    });
  });

  test.describe('Permission-Based Route Guards', () => {
    test('should allow access with required permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'analytics@test.com',
        'password123',
        'user',
        ['read:analytics', 'read:own_data']
      );

      await setupAuthState(page, user);
      
      await page.goto('/analytics');
      await expect(page).toHaveURL('/analytics');
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

      await setupAuthState(page, user);
      
      await page.goto('/analytics');
      await expect(page).toHaveURL('/unauthorized');
      await expect(page.locator('[data-testid="unauthorized-message"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Token Validation', () => {
    test('should reject expired tokens', async ({ page }) => {
      const user = await createTestUserWithRole(
        'expired@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      // Set expired token
      await page.evaluate(() => {
        sessionStorage.setItem('mock-auth', JSON.stringify({
          uid: 'expired-user',
          email: 'expired@test.com',
          role: 'user',
          permissions: ['read:own_data'],
          idToken: 'expired-token',
          expiresAt: Date.now() - 1000 // Expired 1 second ago
        }));
      });
      
      await page.goto('/dashboard');
      await expect(page).toHaveURL('/login?redirect=/dashboard');
      
      await deleteTestUser(user.uid);
    });

    test('should validate token on each request', async ({ page }) => {
      const user = await createTestUserWithRole(
        'valid@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await setupAuthState(page, user);
      
      // First request should work
      await page.goto('/dashboard');
      await expect(page).toHaveURL('/dashboard');
      
      // Simulate token invalidation
      await page.evaluate(() => {
        sessionStorage.setItem('mock-auth', JSON.stringify({
          uid: 'invalidated-user',
          email: 'invalidated@test.com',
          role: 'user',
          permissions: ['read:own_data'],
          idToken: 'invalidated-token',
          expiresAt: Date.now() + 3600000
        }));
      });
      
      // Next request should redirect to login
      await page.goto('/profile');
      await expect(page).toHaveURL('/login?redirect=/profile');
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('API Route Protection', () => {
    test('should protect API routes with authentication', async ({ page }) => {
      await clearAuthState(page);
      
      const response = await page.request.get('/api/user/profile');
      expect(response.status()).toBe(401);
    });

    test('should allow authenticated API access', async ({ page }) => {
      const user = await createTestUserWithRole(
        'api@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await setupAuthState(page, user);
      
      const response = await page.request.get('/api/user/profile');
      expect(response.status()).toBe(200);
      
      await deleteTestUser(user.uid);
    });

    test('should protect admin API routes', async ({ page }) => {
      const user = await createTestUserWithRole(
        'regular@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await setupAuthState(page, user);
      
      const response = await page.request.get('/api/admin/users');
      expect(response.status()).toBe(403);
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Public Route Access', () => {
    test('should allow access to public routes without authentication', async ({ page }) => {
      await clearAuthState(page);
      
      await page.goto('/');
      await expect(page).toHaveURL('/');
      
      await page.goto('/about');
      await expect(page).toHaveURL('/about');
      
      await page.goto('/login');
      await expect(page).toHaveURL('/login');
    });

    test('should allow access to registration page', async ({ page }) => {
      await clearAuthState(page);
      
      await page.goto('/register');
      await expect(page).toHaveURL('/register');
      await expect(page.locator('[data-testid="registration-form"]')).toBeVisible();
    });
  });

  test.describe('Error Handling', () => {
    test('should handle missing authentication gracefully', async ({ page }) => {
      await page.goto('/dashboard', { waitUntil: 'networkidle' });
      
      // Should redirect to login
      await expect(page).toHaveURL(/\/login/);
    });

    test('should handle invalid role gracefully', async ({ page }) => {
      await page.evaluate(() => {
        sessionStorage.setItem('mock-auth', JSON.stringify({
          uid: 'invalid-role-user',
          email: 'invalid@test.com',
          role: 'invalid_role',
          permissions: [],
          idToken: 'invalid-token',
          expiresAt: Date.now() + 3600000
        }));
      });
      
      await page.goto('/admin');
      await expect(page).toHaveURL('/unauthorized');
    });
  });
});
