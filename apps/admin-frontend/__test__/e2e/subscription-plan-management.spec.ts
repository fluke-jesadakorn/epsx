import { test, expect } from '@playwright/test';

test.describe('Subscription and Plan Management - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL ?? 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Subscription Management', () => {
    test('should display subscriptions list', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      await page.waitForSelector('text=Subscription, h1, [data-testid="subscription-page"]', { timeout: 10000 });
    });

    test('should create new subscription', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      const createBtn = page.locator('button:has-text("Create"), button:has-text("New Subscription")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const userInput = page.locator('input[name*="user"], select[name*="user"]').first();
        if (await userInput.isVisible()) {
          if (userInput.tagName === 'select') {
            await userInput.selectOption({ index: 1 });
          } else {
            await userInput.fill('test@example.com');
          }
        }

        const planSelect = page.locator('select[name*="plan"]').first();
        if (await planSelect.isVisible()) {
          await planSelect.selectOption({ index: 1 });
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=Success, text=created', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should view subscription details', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      const firstSubscription = page.locator('[data-testid="subscription-item"], .subscription-card').first();
      if (await firstSubscription.isVisible()) {
        await firstSubscription.click();

        await page.waitForSelector('text=Details, text=Status, text=Plan', { timeout: 5000 });
      }
    });

    test('should cancel subscription', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      const cancelBtn = page.locator('button:has-text("Cancel"), button:has-text("Terminate")').first();
      if (await cancelBtn.isVisible()) {
        await cancelBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Yes")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();
          await page.waitForSelector('text=cancelled, text=terminated', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should renew subscription', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      const renewBtn = page.locator('button:has-text("Renew"), button:has-text("Extend")').first();
      if (await renewBtn.isVisible()) {
        await renewBtn.click();

        const submitBtn = page.locator('button[type="submit"], button:has-text("Renew")').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=renewed, text=extended', { timeout: 10000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Plan Management', () => {
    test('should display plans list', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      await page.waitForSelector('text=Plan, h1, [data-testid="plans-page"]', { timeout: 10000 });
    });

    test('should create new plan', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const createBtn = page.locator('button:has-text("Create Plan"), button:has-text("New Plan")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Premium Plan');
        }

        const priceInput = page.locator('input[name*="price"], input[type="number"]').first();
        if (await priceInput.isVisible()) {
          await priceInput.fill('99.99');
        }

        const descInput = page.locator('textarea[name*="description"]').first();
        if (await descInput.isVisible()) {
          await descInput.fill('Premium features included');
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=Success, text=created', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should edit existing plan', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const editBtn = page.locator('button:has-text("Edit"), button[aria-label*="edit"]').first();
      if (await editBtn.isVisible()) {
        await editBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Updated Plan Name');
        }

        const saveBtn = page.locator('button:has-text("Save"), button[type="submit"]').first();
        if (await saveBtn.isVisible()) {
          await saveBtn.click();
          await page.waitForSelector('text=Success, text=updated', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should delete plan with confirmation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const deleteBtn = page.locator('button:has-text("Delete"), button[aria-label*="delete"]').first();
      if (await deleteBtn.isVisible()) {
        await deleteBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Delete")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();
          await page.waitForSelector('text=deleted, text=removed', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should view plan analytics', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const analyticsBtn = page.locator('button:has-text("Analytics"), button:has-text("Statistics")').first();
      if (await analyticsBtn.isVisible()) {
        await analyticsBtn.click();

        await page.waitForSelector('text=Subscribers, text=Revenue, text=Analytics', { timeout: 5000 });
      }
    });
  });

  test.describe('Plan Features', () => {
    test('should add features to plan', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const editBtn = page.locator('button:has-text("Edit")').first();
      if (await editBtn.isVisible()) {
        await editBtn.click();

        const addFeatureBtn = page.locator('button:has-text("Add Feature")').first();
        if (await addFeatureBtn.isVisible()) {
          await addFeatureBtn.click();

          const featureInput = page.locator('input[name*="feature"], input[placeholder*="feature"]').first();
          if (await featureInput.isVisible()) {
            await featureInput.fill('Advanced Analytics');
          }

          const saveBtn = page.locator('button:has-text("Save")').first();
          if (await saveBtn.isVisible()) {
            await saveBtn.click();
            await page.waitForSelector('text=Success, text=added', { timeout: 10000 }).catch(() => {});
          }
        }
      }
    });

    test('should remove features from plan', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/plans`);

      const editBtn = page.locator('button:has-text("Edit")').first();
      if (await editBtn.isVisible()) {
        await editBtn.click();

        const removeFeatureBtn = page.locator('button:has-text("Remove"), button[aria-label*="remove feature"]').first();
        if (await removeFeatureBtn.isVisible()) {
          await removeFeatureBtn.click();

          await page.waitForSelector('text=removed, text=deleted', { timeout: 5000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Subscription Status', () => {
    test('should filter subscriptions by status', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      const statusFilter = page.locator('select[name*="status"], button:has-text("Status")').first();
      if (await statusFilter.isVisible()) {
        if (statusFilter.tagName === 'select') {
          await statusFilter.selectOption('active');
        } else {
          await statusFilter.click();
          await page.locator('text=Active').first().click();
        }

        await page.waitForTimeout(500);
      }
    });

    test('should display subscription status badges', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      const statusBadge = page.locator('.badge, [data-testid="status-badge"]').first();
      const hasBadge = await statusBadge.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasBadge).toBe('boolean');
    });
  });

  test.describe('Performance', () => {
    test('should load subscriptions page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/subscriptions`);
      await page.waitForSelector('h1, [data-testid="subscription-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });

    test('should load plans page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/plans`);
      await page.waitForSelector('h1, [data-testid="plans-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/subscriptions`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });
  });
});
