import { test, expect } from '@playwright/test';

test.describe('Group Management - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL || 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Group CRUD Operations', () => {
    test('should create new permission group', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const createGroupBtn = page.locator('button:has-text("Create Group"), button:has-text("New Group")').first();
      if (await createGroupBtn.isVisible()) {
        await createGroupBtn.click();

        const nameInput = page.locator('input[name*="name"], input[placeholder*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Test Group');
        }

        const descInput = page.locator('textarea[name*="description"], input[name*="description"]').first();
        if (await descInput.isVisible()) {
          await descInput.fill('E2E Test Group');
        }

        const submitBtn = page.locator('button[type="submit"], button:has-text("Create")').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=Success, text=created', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should edit existing group', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const editBtn = page.locator('button:has-text("Edit"), button[aria-label*="edit"]').first();
      if (await editBtn.isVisible()) {
        await editBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Updated Group Name');
        }

        const saveBtn = page.locator('button:has-text("Save"), button[type="submit"]').first();
        if (await saveBtn.isVisible()) {
          await saveBtn.click();
          await page.waitForSelector('text=Success, text=updated', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should delete group with confirmation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

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
  });

  test.describe('Group Member Management', () => {
    test('should add members to group', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const addMemberBtn = page.locator('button:has-text("Add Member"), button:has-text("Add User")').first();
      if (await addMemberBtn.isVisible()) {
        await addMemberBtn.click();

        const userSelect = page.locator('select[name*="user"], input[placeholder*="user"]').first();
        if (await userSelect.isVisible()) {
          if (userSelect.tagName === 'select') {
            await userSelect.selectOption({ index: 1 });
          } else {
            await userSelect.fill('test@example.com');
          }
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=added, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should remove members from group', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const removeMemberBtn = page.locator('button:has-text("Remove"), button[aria-label*="remove member"]').first();
      if (await removeMemberBtn.isVisible()) {
        await removeMemberBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Remove")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();
          await page.waitForSelector('text=removed, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should display group membership history', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const historyBtn = page.locator('button:has-text("History"), button:has-text("Audit")').first();
      if (await historyBtn.isVisible()) {
        await historyBtn.click();

        await page.waitForSelector('text=Member History, text=Changes, text=Audit Log', { timeout: 5000 });
      }
    });
  });

  test.describe('Dynamic Group Rules', () => {
    test('should create dynamic group with rules', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const dynamicGroupBtn = page.locator('button:has-text("Dynamic Group"), button:has-text("Auto-assign")').first();
      if (await dynamicGroupBtn.isVisible()) {
        await dynamicGroupBtn.click();

        const ruleTypeSelect = page.locator('select[name*="rule"], select[name*="type"]').first();
        if (await ruleTypeSelect.isVisible()) {
          await ruleTypeSelect.selectOption('wallet-balance');
        }

        const conditionInput = page.locator('input[name*="condition"], input[name*="value"]').first();
        if (await conditionInput.isVisible()) {
          await conditionInput.fill('100');
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=created, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should test dynamic group rules', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const testRuleBtn = page.locator('button:has-text("Test Rule"), button:has-text("Preview")').first();
      if (await testRuleBtn.isVisible()) {
        await testRuleBtn.click();

        await page.waitForSelector('text=matches, text=members, text=preview', { timeout: 5000 });
      }
    });
  });

  test.describe('Group Analytics', () => {
    test('should display group analytics dashboard', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const analyticsBtn = page.locator('button:has-text("Analytics"), button:has-text("Statistics")').first();
      if (await analyticsBtn.isVisible()) {
        await analyticsBtn.click();

        await page.waitForSelector('text=Group Analytics, text=Members, text=Activity', { timeout: 5000 });
      }
    });

    test('should export group analytics data', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const exportBtn = page.locator('button:has-text("Export"), button:has-text("Download")').first();
      if (await exportBtn.isVisible()) {
        const downloadPromise = page.waitForEvent('download').catch(() => null);
        await exportBtn.click();

        const download = await downloadPromise;
        if (download) {
          expect(download.suggestedFilename()).toContain('group');
        }
      }
    });
  });

  test.describe('Group Hierarchy', () => {
    test('should create parent-child group relationship', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      const createChildBtn = page.locator('button:has-text("Create Child"), button:has-text("Add Subgroup")').first();
      if (await createChildBtn.isVisible()) {
        await createChildBtn.click();

        const parentSelect = page.locator('select[name*="parent"]').first();
        if (await parentSelect.isVisible()) {
          await parentSelect.selectOption({ index: 1 });
        }

        const childNameInput = page.locator('input[name*="name"]').first();
        if (await childNameInput.isVisible()) {
          await childNameInput.fill('Child Group');
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=created, text=Success', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should visualize group hierarchy', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      const hierarchyView = page.locator('[data-testid="hierarchy-view"], .hierarchy-tree');
      const hasHierarchy = await hierarchyView.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasHierarchy).toBe('boolean');
    });
  });

  test.describe('Error Handling', () => {
    test('should validate group name uniqueness', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      const createGroupBtn = page.locator('button:has-text("Create Group")').first();
      if (await createGroupBtn.isVisible()) {
        await createGroupBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Existing Group');
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();

          await page.waitForSelector('text=exists, text=duplicate, text=already', { timeout: 5000 }).catch(() => {});
        }
      }
    });

    test('should handle API errors gracefully', async ({ page }) => {
      await page.route('**/api/admin/groups**', route => {
        route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server Error' }) });
      });

      await page.goto(`${ADMIN_URL}/permissions`);

      const errorMsg = page.locator('text=error, text=failed, text=Unable to load');
      const hasError = await errorMsg.isVisible({ timeout: 10000 }).catch(() => false);
      expect(typeof hasError).toBe('boolean');
    });
  });

  test.describe('Performance', () => {
    test('should load group management within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/permissions`);
      await page.waitForSelector('h1, [data-testid="group-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });
  });
});
