import { test, expect } from '@playwright/test';
import { createTestUserWithRole, deleteTestUser, loginAsUser } from '../helpers/test-users';

test.describe('IAM Performance Tests', () => {
  test.describe('Authentication Performance', () => {
    test('should complete login within acceptable time', async ({ page }) => {
      const user = await createTestUserWithRole(
        'perf@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await page.goto('/login');
      
      const startTime = Date.now();
      await page.fill('input[type="email"]', 'perf@test.com');
      await page.fill('input[type="password"]', 'password123');
      await page.click('button[type="submit"]');
      
      await page.waitForURL('/dashboard');
      const endTime = Date.now();
      
      const loginTime = endTime - startTime;
      expect(loginTime).toBeLessThan(5000); // Should complete within 5 seconds
      
      await deleteTestUser(user.uid);
    });

    test('should load user permissions quickly', async ({ page }) => {
      const user = await createTestUserWithRole(
        'perms@test.com',
        'password123',
        'user',
        ['read:own_data', 'write:own_data', 'read:analytics']
      );

      await loginAsUser(page, 'perms@test.com', 'password123');
      
      const startTime = Date.now();
      await page.goto('/dashboard');
      await page.waitForSelector('[data-testid="user-permissions"]');
      const endTime = Date.now();
      
      const permissionsLoadTime = endTime - startTime;
      expect(permissionsLoadTime).toBeLessThan(2000); // Should load within 2 seconds
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Authorization Performance', () => {
    test('should check permissions quickly', async ({ page }) => {
      const user = await createTestUserWithRole(
        'authz@test.com',
        'password123',
        'user',
        ['read:own_data', 'read:analytics']
      );

      await loginAsUser(page, 'authz@test.com', 'password123');
      
      const startTime = Date.now();
      await page.goto('/analytics');
      const endTime = Date.now();
      
      const authCheckTime = endTime - startTime;
      expect(authCheckTime).toBeLessThan(1000); // Should check within 1 second
      
      await deleteTestUser(user.uid);
    });

    test('should handle permission denial efficiently', async ({ page }) => {
      const user = await createTestUserWithRole(
        'deny@test.com',
        'password123',
        'user',
        ['read:own_data'] // No admin access
      );

      await loginAsUser(page, 'deny@test.com', 'password123');
      
      const startTime = Date.now();
      await page.goto('/admin');
      const endTime = Date.now();
      
      const denialTime = endTime - startTime;
      expect(denialTime).toBeLessThan(1000); // Should deny within 1 second
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Session Management Performance', () => {
    test('should restore session quickly on page reload', async ({ page }) => {
      const user = await createTestUserWithRole(
        'session@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'session@test.com', 'password123');
      
      const startTime = Date.now();
      await page.reload();
      await page.waitForSelector('[data-testid="user-menu"]');
      const endTime = Date.now();
      
      const sessionRestoreTime = endTime - startTime;
      expect(sessionRestoreTime).toBeLessThan(2000); // Should restore within 2 seconds
      
      await deleteTestUser(user.uid);
    });

    test('should validate token efficiently', async ({ page }) => {
      const user = await createTestUserWithRole(
        'token@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'token@test.com', 'password123');
      
      const startTime = Date.now();
      await page.goto('/dashboard');
      await page.waitForLoadState('networkidle');
      const endTime = Date.now();
      
      const tokenValidationTime = endTime - startTime;
      expect(tokenValidationTime).toBeLessThan(1500); // Should validate within 1.5 seconds
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Concurrent User Performance', () => {
    test('should handle multiple concurrent users', async ({ browser }) => {
      const users = [];
      const contexts = [];
      
      // Create multiple users
      for (let i = 0; i < 5; i++) {
        const user = await createTestUserWithRole(
          `concurrent${i}@test.com`,
          'password123',
          'user',
          ['read:own_data']
        );
        users.push(user);
        
        const context = await browser.newContext();
        contexts.push(context);
        
        const page = await context.newPage();
        await loginAsUser(page, `concurrent${i}@test.com`, 'password123');
      }
      
      // All users should be able to access dashboard
      for (const context of contexts) {
        const page = await context.newPage();
        const startTime = Date.now();
        await page.goto('/dashboard');
        await page.waitForSelector('[data-testid="user-menu"]');
        const endTime = Date.now();
        
        const loadTime = endTime - startTime;
        expect(loadTime).toBeLessThan(3000); // Each user should load within 3 seconds
      }
      
      // Cleanup
      for (const context of contexts) {
        await context.close();
      }
      for (const user of users) {
        await deleteTestUser(user.uid);
      }
    });
  });

  test.describe('Memory Usage', () => {
    test('should not leak memory during authentication', async ({ page }) => {
      const user = await createTestUserWithRole(
        'memory@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      // Skip memory tests in environments where performance.memory is not available
      const hasMemoryAPI = await page.evaluate(() => 'memory' in performance);
      if (!hasMemoryAPI) {
        test.skip();
        return;
      }

      // Measure memory before login
      const initialMemory = await page.evaluate(() => (performance as any).memory.usedJSHeapSize);
      
      await loginAsUser(page, 'memory@test.com', 'password123');
      
      // Navigate through various pages
      await page.goto('/dashboard');
      await page.goto('/profile');
      await page.goto('/settings');
      
      // Measure memory after navigation
      const finalMemory = await page.evaluate(() => (performance as any).memory.usedJSHeapSize);
      
      const memoryIncrease = finalMemory - initialMemory;
      expect(memoryIncrease).toBeLessThan(50 * 1024 * 1024); // Less than 50MB increase
      
      await deleteTestUser(user.uid);
    });
  });

  test.describe('Network Performance', () => {
    test('should minimize authentication requests', async ({ page }) => {
      const user = await createTestUserWithRole(
        'network@test.com',
        'password123',
        'user',
        ['read:own_data']
      );

      await loginAsUser(page, 'network@test.com', 'password123');
      
      // Monitor network requests
      const requests: string[] = [];
      page.on('request', (request) => {
        if (request.url().includes('firebase') || request.url().includes('auth')) {
          requests.push(request.url());
        }
      });
      
      await page.goto('/dashboard');
      await page.goto('/profile');
      
      // Should not make excessive auth requests
      expect(requests.length).toBeLessThan(10);
      
      await deleteTestUser(user.uid);
    });

    test('should cache permissions effectively', async ({ page }) => {
      const user = await createTestUserWithRole(
        'cache@test.com',
        'password123',
        'user',
        ['read:own_data', 'read:analytics']
      );

      await loginAsUser(page, 'cache@test.com', 'password123');
      
      const requests: string[] = [];
      page.on('request', (request) => {
        if (request.url().includes('permissions') || request.url().includes('user')) {
          requests.push(request.url());
        }
      });
      
      // Access same protected resource multiple times
      await page.goto('/analytics');
      await page.goto('/dashboard');
      await page.goto('/analytics');
      
      // Should cache permissions and not make duplicate requests
      const uniqueRequests = [...new Set(requests)];
      expect(uniqueRequests.length).toBeLessThan(5);
      
      await deleteTestUser(user.uid);
    });
  });
});
