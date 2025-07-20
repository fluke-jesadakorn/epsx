import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser } from '../helpers/test-users';

test.describe('Authentication Flow', () => {
  test.describe('User Registration', () => {
    test('should register new user with basic role', async ({ page }) => {
      await page.goto('/register');
      
      await page.fill('input[name="email"]', 'newuser@test.com');
      await page.fill('input[name="password"]', 'password123');
      await page.fill('input[name="confirmPassword"]', 'password123');
      await page.click('button[type="submit"]');
      
      await page.waitForURL('/dashboard');
      
      // Verify user is logged in
      await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
      
      // Cleanup
      // Note: In real implementation, you'd need to delete this user from Firebase
    });

    test('should prevent duplicate registration', async ({ page }) => {
      const user = await createTestUserWithRole(
        'duplicate@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await page.goto('/register');
      
      await page.fill('input[name="email"]', 'duplicate@test.com');
      await page.fill('input[name="password"]', 'password123');
      await page.fill('input[name="confirmPassword"]', 'password123');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('[data-testid="error-message"]')).toContainText('already exists');
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('User Login', () => {
    test('should login with valid credentials', async ({ page }) => {
      const user = await createTestUserWithRole(
        'login@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'login@test.com', 'password123');
      
      await expect(page).toHaveURL('/dashboard');
      await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should fail login with invalid credentials', async ({ page }) => {
      await page.goto('/login');
      
      await page.fill('input[type="email"]', 'invalid@test.com');
      await page.fill('input[type="password"]', 'wrongpassword');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('[data-testid="error-message"]')).toContainText('Invalid credentials');
      await expect(page).toHaveURL('/login');
    });

    test('should redirect authenticated users away from login', async ({ page }) => {
      const user = await createTestUserWithRole(
        'redirect@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'redirect@test.com', 'password123');
      
      // Try to access login page while authenticated
      await page.goto('/login');
      
      // Should redirect to dashboard
      await expect(page).toHaveURL('/dashboard');
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Password Reset', () => {
    test('should send password reset email', async ({ page }) => {
      const user = await createTestUserWithRole(
        'reset@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await page.goto('/forgot-password');
      
      await page.fill('input[type="email"]', 'reset@test.com');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('[data-testid="success-message"]')).toContainText('reset email sent');
      
      await deleteTestUser(user.uid);
    });

    test('should handle non-existent email gracefully', async ({ page }) => {
      await page.goto('/forgot-password');
      
      await page.fill('input[type="email"]', 'nonexistent@test.com');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('[data-testid="success-message"]')).toContainText('reset email sent');
      // Should not reveal if email exists or not for security
    });
  });

  test.describe('Session Management', () => {
    test('should maintain session on page refresh', async ({ page }) => {
      const user = await createTestUserWithRole(
        'session@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'session@test.com', 'password123');
      
      await page.reload();
      
      // Should still be logged in
      await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should logout successfully', async ({ page }) => {
      const user = await createTestUserWithRole(
        'logout@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'logout@test.com', 'password123');
      
      await page.click('[data-testid="user-menu"]');
      await page.click('[data-testid="logout-button"]');
      
      await expect(page).toHaveURL('/login');
      await expect(page.locator('[data-testid="login-form"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should redirect to login after logout when accessing protected route', async ({ page }) => {
      const user = await createTestUserWithRole(
        'redirect-after-logout@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'redirect-after-logout@test.com', 'password123');
      
      // Logout
      await page.click('[data-testid="user-menu"]');
      await page.click('[data-testid="logout-button"]');
      
      // Try to access protected route
      await page.goto('/dashboard');
      
      // Should redirect to login
      await expect(page).toHaveURL('/login?redirect=/dashboard');
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Remember Me', () => {
    test('should persist session with remember me', async ({ page }) => {
      const user = await createTestUserWithRole(
        'remember@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await page.goto('/login');
      
      await page.fill('input[type="email"]', 'remember@test.com');
      await page.fill('input[type="password"]', 'password123');
      await page.check('input[name="rememberMe"]');
      await page.click('button[type="submit"]');
      
      await page.waitForURL('/dashboard');
      
      // Simulate browser restart by clearing session storage
      await page.evaluate(() => sessionStorage.clear());
      await page.reload();
      
      // Should still be logged in due to persistent session
      await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });
  });
});
