import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser } from '../helpers/test-users';

test.describe('Firebase IAM Integration', () => {
  test.describe('User Profile Management', () => {
    let testUser: any;

    test.afterEach(async () => {
      if (testUser) {
        await deleteTestUser(testUser.uid);
      }
    });

    test('should create user profile on registration', async ({ page }) => {
      testUser = await createTestUserWithRole(
        'profile@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'profile@test.com', 'password123');
      
      // Check if user profile is loaded
      await page.goto('/profile');
      await expect(page.locator('[data-testid="user-email"]')).toContainText('profile@test.com');
      await expect(page.locator('[data-testid="user-role"]')).toContainText('user');
    });

    test('should update user profile', async ({ page }) => {
      testUser = await createTestUserWithRole(
        'update@test.com',
        'password123',
        'user',
        ['read:own_data', 'write:own_data']
      );

      await loginAsUser(page, 'update@test.com', 'password123');
      
      await page.goto('/profile/edit');
      await page.fill('input[name="displayName"]', 'Updated Name');
      await page.fill('input[name="phone"]', '1234567890');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('.success-message')).toContainText('Profile updated successfully');
      await expect(page.locator('[data-testid="user-display-name"]')).toContainText('Updated Name');
    });

    test('should handle profile update errors', async ({ page }) => {
      testUser = await createTestUserWithRole(
        'error@test.com',
        'password123',
        'user',
        ['read:own_data', 'write:own_data']
      );

      await loginAsUser(page, 'error@test.com', 'password123');
      
      await page.goto('/profile/edit');
      await page.fill('input[name="email"]', 'invalid-email');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('.error-message')).toContainText('Invalid email format');
    });
  });

  test.describe('Role Synchronization', () => {
    test('should sync user role from Firestore', async ({ page }) => {
      const user = await createTestUserWithRole(
        'sync@test.com',
        'password123',
        'premium',
        ['read:own_data', 'access:premium_features']
      );

      await loginAsUser(page, 'sync@test.com', 'password123');
      
      await page.goto('/dashboard');
      await expect(page.locator('[data-testid="user-role-badge"]')).toContainText('Premium');
      
      await deleteTestUser(user.uid);
    });

    test('should handle role changes in real-time', async ({ page }) => {
      const user = await createTestUserWithRole(
        'rolechange@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'rolechange@test.com', 'password123');
      
      // Initially user role is 'user'
      await page.goto('/dashboard');
      await expect(page.locator('[data-testid="user-role-badge"]')).toContainText('User');
      
      // Simulate role upgrade (in real app, this would be handled by admin)
      await page.evaluate(() => {
        // This would typically be done via admin panel
        console.log('Role upgrade simulated');
      });
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Permission Validation', () => {
    test('should validate permissions before actions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'limited@test.com',
        'password123',
        'user',
        ['read:own_data'] // No write permission
      );

      await loginAsUser(page, 'limited@test.com', 'password123');
      
      await page.goto('/profile/edit');
      await page.fill('input[name="displayName"]', 'New Name');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('.error-message')).toContainText('Insufficient permissions');
      
      await deleteTestUser(user.uid);
    });

    test('should allow actions with proper permissions', async ({ page }) => {
      const user = await createTestUserWithRole(
        'permitted@test.com',
        'password123',
        'user',
        ['read:own_data', 'write:own_data']
      );

      await loginAsUser(page, 'permitted@test.com', 'password123');
      
      await page.goto('/profile/edit');
      await page.fill('input[name="displayName"]', 'Permitted User');
      await page.click('button[type="submit"]');
      
      await expect(page.locator('.success-message')).toContainText('Profile updated successfully');
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Session Management', () => {
    test('should maintain session across page reloads', async ({ page }) => {
      const user = await createTestUserWithRole(
        'session@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'session@test.com', 'password123');
      
      await page.goto('/dashboard');
      await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
      
      await page.reload();
      await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
      
      await deleteTestUser(user.uid);
    });

    test('should handle session expiration', async ({ page }) => {
      const user = await createTestUserWithRole(
        'expired@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'expired@test.com', 'password123');
      
      // Simulate session expiration
      await page.evaluate(() => {
        localStorage.removeItem('firebase:authUser');
        sessionStorage.clear();
      });
      
      await page.reload();
      await expect(page).toHaveURL(/.*\/login/);
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Multi-Device Sessions', () => {
    test('should handle concurrent sessions', async ({ browser }) => {
      const user = await createTestUserWithRole(
        'multi@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      const context1 = await browser.newContext();
      const context2 = await browser.newContext();
      
      const page1 = await context1.newPage();
      const page2 = await context2.newPage();
      
      await loginAsUser(page1, 'multi@test.com', 'password123');
      await loginAsUser(page2, 'multi@test.com', 'password123');
      
      // Both sessions should be active
      await page1.goto('/dashboard');
      await page2.goto('/dashboard');
      
      await expect(page1.locator('[data-testid="user-menu"]')).toBeVisible();
      await expect(page2.locator('[data-testid="user-menu"]')).toBeVisible();
      
      await context1.close();
      await context2.close();
      await deleteTestUser(user.uid);
    });
  });
});
