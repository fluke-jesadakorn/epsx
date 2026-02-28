import type { Page } from '@playwright/test';
import { test, expect } from '@playwright/test';
import { performance } from 'node:perf_hooks';

// Configuration for load testing
const LOAD_TEST_CONFIG = {
  concurrent_users: 10,
  requests_per_user: 20,
  timeout_ms: 30000,
  max_response_time_ms: 5000,
  success_rate_threshold: 95,
};

// Mock API endpoints for load testing
async function setupMockApiEndpoints(page: Page) {
  // Mock challenge endpoint with realistic delay
  await page.route('**/api/auth/web3/challenge', async (route) => {
    // Simulate database lookup time
    await new Promise(resolve => setTimeout(resolve, Math.random() * 100 + 50));
    
    const request = route.request();
    const body = await request.postDataJSON();
    
    if (!body?.wallet_address) {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'wallet_address required' }),
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        nonce: `nonce_${Date.now()}_${Math.random()}`,
        message: `epsx.io wants you to sign in with your Ethereum account:\n${body.wallet_address}\n\nSign in to EPSX analytics platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: nonce_${Date.now()}\nIssued At: ${new Date().toISOString()}`,
        expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
      }),
    });
  });

  // Mock verify endpoint with realistic processing time
  await page.route('**/api/auth/web3/verify', async (route) => {
    // Simulate signature verification time
    await new Promise(resolve => setTimeout(resolve, Math.random() * 200 + 100));
    
    const request = route.request();
    const body = await request.postDataJSON();
    
    if (!body?.message || !body?.signature || !body?.wallet_address) {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Missing required fields' }),
      });
      return;
    }

    // Simulate occasional verification failures (5% failure rate)
    if (Math.random() < 0.05) {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Invalid signature' }),
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        access_token: `token_${Date.now()}_${Math.random()}`,
        id_token: `id_${Date.now()}_${Math.random()}`,
        refresh_token: `refresh_${Date.now()}_${Math.random()}`,
        user_id: `user_${Date.now()}`,
        wallet_address: body.wallet_address,
        permissions: ['user:profile:view', 'user:analytics:access'],
        expires_in: 3600
      }),
    });
  });

  // Mock permissions endpoint
  await page.route('**/api/auth/web3/permissions*', async (route) => {
    await new Promise(resolve => setTimeout(resolve, Math.random() * 50 + 25));
    
    const url = new URL(route.request().url());
    const walletAddress = url.searchParams.get('wallet_address');
    
    if (!walletAddress) {
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'wallet_address parameter required' }),
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        wallet_address: walletAddress,
        permissions: [
          { permission: 'user:profile:view', permission_type: 'manual', granted_at: new Date().toISOString(), is_active: true }
        ],
        automatic_grants: []
      }),
    });
  });

  // Mock session endpoint
  await page.route('**/api/auth/session', async (route) => {
    await new Promise(resolve => setTimeout(resolve, Math.random() * 30 + 10));
    
    await route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'No active session' }),
    });
  });

  // Mock status endpoint
  await page.route('**/api/auth/web3/status*', async (route) => {
    await new Promise(resolve => setTimeout(resolve, Math.random() * 40 + 20));
    
    const url = new URL(route.request().url());
    const walletAddress = url.searchParams.get('wallet_address');
    
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        wallet_address: walletAddress,
        is_registered: Math.random() < 0.3, // 30% registered
        is_available: Math.random() < 0.7, // 70% available
        user_id: Math.random() < 0.3 ? `user_${Date.now()}` : null,
        status: Math.random() < 0.3 ? 'registered' : 'available'
      }),
    });
  });
}

// Helper to generate unique wallet addresses
function generateWalletAddress(index: number): string {
  return `0x${index.toString(16).padStart(40, '0')}`;
}

// Helper to measure response times
async function measureResponseTime<T>(operation: () => Promise<T>): Promise<{ result: T; duration: number }> {
  const start = performance.now();
  const result = await operation();
  const duration = performance.now() - start;
  return { result, duration };
}

test.describe('Web3 Authentication Load Testing', () => {
  test.beforeEach(async ({ page }) => {
    await setupMockApiEndpoints(page);
  });

  test.describe('API Endpoint Performance', () => {
    test('should handle challenge generation under load', async ({ page }) => {
      const results: { success: boolean; duration: number }[] = [];
      const promises: Promise<void>[] = [];
      
      // Create concurrent requests
      for (let i = 0; i < LOAD_TEST_CONFIG.concurrent_users; i++) {
        const walletAddress = generateWalletAddress(i);
        
        const promise = (async () => {
          try {
            const { duration } = await measureResponseTime(async () => {
              return await page.request.post('/api/auth/web3/challenge', {
                data: { wallet_address: walletAddress },
                timeout: LOAD_TEST_CONFIG.timeout_ms
              });
            });
            
            results.push({ success: true, duration });
          } catch (error) {
            results.push({ success: false, duration: LOAD_TEST_CONFIG.timeout_ms });
          }
        })();
        
        promises.push(promise);
      }
      
      // Wait for all requests to complete
      await Promise.all(promises);
      
      // Analyze results
      const successfulRequests = results.filter(r => r.success);
      const successRate = (successfulRequests.length / results.length) * 100;
      const avgResponseTime = successfulRequests.reduce((sum, r) => sum + r.duration, 0) / successfulRequests.length;
      const maxResponseTime = Math.max(...successfulRequests.map(r => r.duration));
      
      console.log(`Challenge Generation Load Test Results:`);
      console.log(`  Concurrent users: ${LOAD_TEST_CONFIG.concurrent_users}`);
      console.log(`  Success rate: ${successRate.toFixed(2)}%`);
      console.log(`  Average response time: ${avgResponseTime.toFixed(2)}ms`);
      console.log(`  Max response time: ${maxResponseTime.toFixed(2)}ms`);
      
      // Performance assertions
      expect(successRate).toBeGreaterThanOrEqual(LOAD_TEST_CONFIG.success_rate_threshold);
      expect(avgResponseTime).toBeLessThan(LOAD_TEST_CONFIG.max_response_time_ms);
      expect(maxResponseTime).toBeLessThan(LOAD_TEST_CONFIG.max_response_time_ms * 2);
    });

    test('should handle signature verification under load', async ({ page }) => {
      const results: { success: boolean; duration: number }[] = [];
      const promises: Promise<void>[] = [];
      
      for (let i = 0; i < LOAD_TEST_CONFIG.concurrent_users; i++) {
        const walletAddress = generateWalletAddress(i);
        
        const promise = (async () => {
          try {
            const { duration } = await measureResponseTime(async () => {
              return await page.request.post('/api/auth/web3/verify', {
                data: {
                  message: `Mock SIWE message for ${walletAddress}`,
                  signature: `0x${  'a'.repeat(130)}`,
                  wallet_address: walletAddress
                },
                timeout: LOAD_TEST_CONFIG.timeout_ms
              });
            });
            
            results.push({ success: true, duration });
          } catch (error) {
            results.push({ success: false, duration: LOAD_TEST_CONFIG.timeout_ms });
          }
        })();
        
        promises.push(promise);
      }
      
      await Promise.all(promises);
      
      const successfulRequests = results.filter(r => r.success);
      const successRate = (successfulRequests.length / results.length) * 100;
      const avgResponseTime = successfulRequests.reduce((sum, r) => sum + r.duration, 0) / successfulRequests.length;
      
      console.log(`Signature Verification Load Test Results:`);
      console.log(`  Success rate: ${successRate.toFixed(2)}%`);
      console.log(`  Average response time: ${avgResponseTime.toFixed(2)}ms`);
      
      // Account for 5% simulated failure rate
      expect(successRate).toBeGreaterThanOrEqual(90);
      expect(avgResponseTime).toBeLessThan(LOAD_TEST_CONFIG.max_response_time_ms);
    });

    test('should handle permission checks under load', async ({ page }) => {
      const results: { success: boolean; duration: number }[] = [];
      const promises: Promise<void>[] = [];
      
      for (let i = 0; i < LOAD_TEST_CONFIG.concurrent_users * 5; i++) { // More permission checks
        const walletAddress = generateWalletAddress(i % LOAD_TEST_CONFIG.concurrent_users);
        
        const promise = (async () => {
          try {
            const { duration } = await measureResponseTime(async () => {
              return await page.request.get(`/api/auth/web3/permissions?wallet_address=${walletAddress}`, {
                timeout: LOAD_TEST_CONFIG.timeout_ms
              });
            });
            
            results.push({ success: true, duration });
          } catch (error) {
            results.push({ success: false, duration: LOAD_TEST_CONFIG.timeout_ms });
          }
        })();
        
        promises.push(promise);
      }
      
      await Promise.all(promises);
      
      const successfulRequests = results.filter(r => r.success);
      const successRate = (successfulRequests.length / results.length) * 100;
      const avgResponseTime = successfulRequests.reduce((sum, r) => sum + r.duration, 0) / successfulRequests.length;
      
      console.log(`Permission Check Load Test Results:`);
      console.log(`  Total requests: ${results.length}`);
      console.log(`  Success rate: ${successRate.toFixed(2)}%`);
      console.log(`  Average response time: ${avgResponseTime.toFixed(2)}ms`);
      
      expect(successRate).toBeGreaterThanOrEqual(LOAD_TEST_CONFIG.success_rate_threshold);
      expect(avgResponseTime).toBeLessThan(1000); // Permission checks should be very fast
    });
  });

  test.describe('Frontend Performance Under Load', () => {
    test('should handle multiple concurrent auth flows', async ({ browser }) => {
      const results: { success: boolean; duration: number }[] = [];
      const promises: Promise<void>[] = [];
      
      for (let i = 0; i < LOAD_TEST_CONFIG.concurrent_users; i++) {
        const promise = (async () => {
          const context = await browser.newContext();
          const page = await context.newPage();
          
          try {
            await setupMockApiEndpoints(page);
            
            // Mock wallet connection
            await page.addInitScript((address) => {
              (window as any).ethereum = {
                isMetaMask: true,
                request: async ({ method }: any) => {
                  switch (method) {
                    case 'eth_requestAccounts':
                      return [address];
                    case 'eth_accounts':
                      return [address];
                    case 'personal_sign':
                      return `0x${  'a'.repeat(130)}`;
                    default:
                      return [];
                  }
                },
                on: () => {},
                removeListener: () => {},
              };
            }, generateWalletAddress(i));
            
            const { duration } = await measureResponseTime(async () => {
              await page.goto('http://localhost:3000/auth');
              await page.getByText('Connect Web3 Wallet').click();
              await page.getByText('Sign In with Wallet').click();
              await page.waitForURL('**/dashboard', { timeout: 10000 });
            });
            
            results.push({ success: true, duration });
          } catch (error) {
            console.error(`Auth flow ${i} failed:`, error);
            results.push({ success: false, duration: 10000 });
          } finally {
            await context.close();
          }
        })();
        
        promises.push(promise);
      }
      
      await Promise.all(promises);
      
      const successfulFlows = results.filter(r => r.success);
      const successRate = (successfulFlows.length / results.length) * 100;
      const avgFlowTime = successfulFlows.reduce((sum, r) => sum + r.duration, 0) / successfulFlows.length;
      
      console.log(`Concurrent Auth Flows Results:`);
      console.log(`  Concurrent flows: ${LOAD_TEST_CONFIG.concurrent_users}`);
      console.log(`  Success rate: ${successRate.toFixed(2)}%`);
      console.log(`  Average flow time: ${avgFlowTime.toFixed(2)}ms`);
      
      expect(successRate).toBeGreaterThanOrEqual(LOAD_TEST_CONFIG.success_rate_threshold);
      expect(avgFlowTime).toBeLessThan(15000); // Full auth flow should complete within 15 seconds
    });

    test('should maintain UI responsiveness under load', async ({ page }) => {
      await page.goto('http://localhost:3000/auth');
      
      // Mock wallet for testing
      await page.addInitScript(() => {
        (window as any).ethereum = {
          isMetaMask: true,
          request: async ({ method }: any) => {
            // Simulate slow wallet responses
            await new Promise(resolve => setTimeout(resolve, 200));
            switch (method) {
              case 'eth_requestAccounts':
                return ['0x1234567890123456789012345678901234567890'];
              case 'eth_accounts':
                return ['0x1234567890123456789012345678901234567890'];
              default:
                return [];
            }
          },
          on: () => {},
          removeListener: () => {},
        };
      });
      
      // Measure UI interaction response times
      const interactions = [
        { action: () => page.getByText('Connect Web3 Wallet').click(), name: 'Connect Button Click' },
        { action: () => page.getByRole('tab', { name: /email/i }).click(), name: 'Tab Switch' },
        { action: () => page.getByRole('tab', { name: /web3/i }).click(), name: 'Tab Switch Back' },
      ];
      
      const interactionResults: { name: string; duration: number }[] = [];
      
      for (const interaction of interactions) {
        const { duration } = await measureResponseTime(interaction.action);
        interactionResults.push({ name: interaction.name, duration });
      }
      
      console.log('UI Interaction Response Times:');
      for (const result of interactionResults) {
        console.log(`  ${result.name}: ${result.duration.toFixed(2)}ms`);
        expect(result.duration).toBeLessThan(500); // UI interactions should be under 500ms
      }
    });
  });

  test.describe('Memory and Resource Usage', () => {
    test('should not leak memory during repeated auth operations', async ({ page }) => {
      await page.goto('http://localhost:3000/auth');
      
      // Mock wallet
      await page.addInitScript(() => {
        (window as any).ethereum = {
          isMetaMask: true,
          request: async ({ method }: any) => {
            switch (method) {
              case 'eth_requestAccounts':
                return ['0x1234567890123456789012345678901234567890'];
              case 'eth_accounts':
                return ['0x1234567890123456789012345678901234567890'];
              default:
                return [];
            }
          },
          on: () => {},
          removeListener: () => {},
        };
      });
      
      // Get initial memory usage
      const initialMemory = await page.evaluate(() => {
        if ('memory' in performance) {
          return (performance as any).memory.usedJSHeapSize;
        }
        return 0;
      });
      
      // Perform repeated operations
      for (let i = 0; i < 50; i++) {
        await page.getByText('Connect Web3 Wallet').click();
        await page.reload();
        await page.waitForLoadState('networkidle');
      }
      
      // Get final memory usage
      const finalMemory = await page.evaluate(() => {
        if ('memory' in performance) {
          return (performance as any).memory.usedJSHeapSize;
        }
        return 0;
      });
      
      if (initialMemory > 0 && finalMemory > 0) {
        const memoryIncrease = finalMemory - initialMemory;
        const memoryIncreasePercent = (memoryIncrease / initialMemory) * 100;
        
        console.log(`Memory Usage:`);
        console.log(`  Initial: ${(initialMemory / 1024 / 1024).toFixed(2)} MB`);
        console.log(`  Final: ${(finalMemory / 1024 / 1024).toFixed(2)} MB`);
        console.log(`  Increase: ${memoryIncreasePercent.toFixed(2)}%`);
        
        // Memory should not increase by more than 50% after repeated operations
        expect(memoryIncreasePercent).toBeLessThan(50);
      }
    });

    test('should handle rapid successive requests', async ({ page }) => {
      const requestTimes: number[] = [];
      const totalRequests = 100;
      
      for (let i = 0; i < totalRequests; i++) {
        const walletAddress = generateWalletAddress(i);
        
        const { duration } = await measureResponseTime(async () => {
          await page.request.post('/api/auth/web3/challenge', {
            data: { wallet_address: walletAddress },
            timeout: 5000
          });
        });
        
        requestTimes.push(duration);
        
        // Small delay to prevent overwhelming
        await page.waitForTimeout(10);
      }
      
      const avgTime = requestTimes.reduce((sum, time) => sum + time, 0) / requestTimes.length;
      const maxTime = Math.max(...requestTimes);
      const minTime = Math.min(...requestTimes);
      
      console.log(`Rapid Successive Requests:`);
      console.log(`  Total requests: ${totalRequests}`);
      console.log(`  Average time: ${avgTime.toFixed(2)}ms`);
      console.log(`  Min time: ${minTime.toFixed(2)}ms`);
      console.log(`  Max time: ${maxTime.toFixed(2)}ms`);
      
      expect(avgTime).toBeLessThan(1000);
      expect(maxTime).toBeLessThan(5000);
    });
  });

  test.describe('Error Handling Under Load', () => {
    test('should gracefully handle partial service failures', async ({ page }) => {
      // Mock intermittent failures
      let requestCount = 0;
      await page.route('**/api/auth/web3/challenge', async (route) => {
        requestCount++;
        
        // Fail every 5th request
        if (requestCount % 5 === 0) {
          await route.fulfill({
            status: 503,
            contentType: 'application/json',
            body: JSON.stringify({ error: 'Service temporarily unavailable' }),
          });
          return;
        }
        
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            nonce: `nonce_${requestCount}`,
            message: 'Mock SIWE message',
            expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
          }),
        });
      });
      
      const results: boolean[] = [];
      
      for (let i = 0; i < 20; i++) {
        try {
          const response = await page.request.post('/api/auth/web3/challenge', {
            data: { wallet_address: generateWalletAddress(i) },
            timeout: 5000
          });
          
          results.push(response.ok());
        } catch (error) {
          results.push(false);
        }
      }
      
      const successCount = results.filter(Boolean).length;
      const successRate = (successCount / results.length) * 100;
      
      console.log(`Partial Failure Handling:`);
      console.log(`  Success rate: ${successRate}% (${successCount}/${results.length})`);
      
      // Should handle 80% success rate (20% failures)
      expect(successRate).toBe(80);
    });

    test('should recover from temporary overload', async ({ page }) => {
      // Simulate temporary overload by introducing delays
      let overloadActive = true;
      setTimeout(() => { overloadActive = false; }, 5000); // Overload for 5 seconds
      
      await page.route('**/api/auth/web3/challenge', async (route) => {
        if (overloadActive) {
          // Simulate overload with long delay or failure
          if (Math.random() < 0.7) {
            await route.fulfill({
              status: 503,
              contentType: 'application/json',
              body: JSON.stringify({ error: 'Server overloaded' }),
            });
            return;
          }
          await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            nonce: `nonce_${Date.now()}`,
            message: 'Mock SIWE message',
            expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString()
          }),
        });
      });
      
      const results: { time: number; success: boolean }[] = [];
      
      // Make requests over 10 seconds (including overload period)
      for (let i = 0; i < 20; i++) {
        const startTime = Date.now();
        
        try {
          const response = await page.request.post('/api/auth/web3/challenge', {
            data: { wallet_address: generateWalletAddress(i) },
            timeout: 8000
          });
          
          results.push({ time: startTime, success: response.ok() });
        } catch (error) {
          results.push({ time: startTime, success: false });
        }
        
        await page.waitForTimeout(500); // Space out requests
      }
      
      // Analyze recovery
      const overloadPeriodResults = results.filter(r => r.time < Date.now() - 10000 + 5000);
      const recoveryPeriodResults = results.filter(r => r.time >= Date.now() - 10000 + 5000);
      
      const overloadSuccessRate = overloadPeriodResults.filter(r => r.success).length / overloadPeriodResults.length * 100;
      const recoverySuccessRate = recoveryPeriodResults.filter(r => r.success).length / recoveryPeriodResults.length * 100;
      
      console.log(`Overload Recovery:`);
      console.log(`  During overload: ${overloadSuccessRate.toFixed(2)}% success`);
      console.log(`  After recovery: ${recoverySuccessRate.toFixed(2)}% success`);
      
      // Should show clear improvement after overload period
      expect(recoverySuccessRate).toBeGreaterThan(overloadSuccessRate);
      expect(recoverySuccessRate).toBeGreaterThanOrEqual(90);
    });
  });
});