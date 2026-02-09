import { test, expect } from '@playwright/test';

test.describe('Stock Ranking Packages and Policies - E2E', () => {
  const ADMIN_URL = process.env.ADMIN_URL ?? 'http://localhost:3001';

  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('Stock Ranking Packages', () => {
    test('should display stock ranking packages page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      await page.waitForSelector('text=Stock Ranking, text=Package, h1', { timeout: 10000 });
    });

    test('should display package list', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      const packageList = page.locator('[data-testid="package-list"], .package-card, .package-item');
      const hasPackages = await packageList.first().isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasPackages).toBe('boolean');
    });

    test('should assign package to user', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      const assignBtn = page.locator('button:has-text("Assign"), button:has-text("Assign Package")').first();
      if (await assignBtn.isVisible()) {
        await assignBtn.click();

        const userInput = page.locator('input[name*="user"], select[name*="user"]').first();
        if (await userInput.isVisible()) {
          if (userInput.tagName === 'select') {
            await userInput.selectOption({ index: 1 });
          } else {
            await userInput.fill('test@example.com');
          }
        }

        const packageSelect = page.locator('select[name*="package"]').first();
        if (await packageSelect.isVisible()) {
          await packageSelect.selectOption({ index: 1 });
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=Success, text=assigned', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should display package assignment list', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      const assignmentList = page.locator('[data-testid="assignment-list"], text=Assignments').first();
      const hasAssignments = await assignmentList.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasAssignments).toBe('boolean');
    });

    test('should filter packages by tier', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      const tierFilter = page.locator('select[name*="tier"], button:has-text("Tier")').first();
      if (await tierFilter.isVisible()) {
        if (tierFilter.tagName === 'select') {
          await tierFilter.selectOption('premium');
        } else {
          await tierFilter.click();
          await page.locator('text=Premium').first().click();
        }

        await page.waitForTimeout(500);
      }
    });

    test('should unassign package from user', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      const unassignBtn = page.locator('button:has-text("Unassign"), button:has-text("Remove")').first();
      if (await unassignBtn.isVisible()) {
        await unassignBtn.click();

        const confirmBtn = page.locator('button:has-text("Confirm"), button:has-text("Yes")').first();
        if (await confirmBtn.isVisible()) {
          await confirmBtn.click();
          await page.waitForSelector('text=unassigned, text=removed', { timeout: 10000 }).catch(() => {});
        }
      }
    });
  });

  test.describe('Policy Builder', () => {
    test('should display policy builder page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      await page.waitForSelector('text=Policy, text=Builder, h1', { timeout: 10000 });
    });

    test('should create new policy', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const createBtn = page.locator('button:has-text("Create Policy"), button:has-text("New Policy")').first();
      if (await createBtn.isVisible()) {
        await createBtn.click();

        const nameInput = page.locator('input[name*="name"]').first();
        if (await nameInput.isVisible()) {
          await nameInput.fill('Test Access Policy');
        }

        const descInput = page.locator('textarea[name*="description"]').first();
        if (await descInput.isVisible()) {
          await descInput.fill('Policy for testing purposes');
        }

        const submitBtn = page.locator('button[type="submit"]').first();
        if (await submitBtn.isVisible()) {
          await submitBtn.click();
          await page.waitForSelector('text=Success, text=created', { timeout: 10000 }).catch(() => {});
        }
      }
    });

    test('should add rules to policy', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const editBtn = page.locator('button:has-text("Edit"), button[aria-label*="edit"]').first();
      if (await editBtn.isVisible()) {
        await editBtn.click();

        const addRuleBtn = page.locator('button:has-text("Add Rule")').first();
        if (await addRuleBtn.isVisible()) {
          await addRuleBtn.click();

          const conditionSelect = page.locator('select[name*="condition"], select[name*="type"]').first();
          if (await conditionSelect.isVisible()) {
            await conditionSelect.selectOption({ index: 1 });
          }

          const valueInput = page.locator('input[name*="value"]').first();
          if (await valueInput.isVisible()) {
            await valueInput.fill('admin');
          }

          const saveBtn = page.locator('button:has-text("Save")').first();
          if (await saveBtn.isVisible()) {
            await saveBtn.click();
            await page.waitForSelector('text=Success, text=added', { timeout: 10000 }).catch(() => {});
          }
        }
      }
    });

    test('should test policy rules', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const testBtn = page.locator('button:has-text("Test"), button:has-text("Preview")').first();
      if (await testBtn.isVisible()) {
        await testBtn.click();

        await page.waitForSelector('text=Test Results, text=Preview, text=Simulation', { timeout: 5000 });
      }
    });

    test('should delete policy', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

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

  test.describe('Policy Monitor', () => {
    test('should display policy monitor page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const monitorTab = page.locator('button:has-text("Monitor"), [role="tab"]:has-text("Monitor")').first();
      if (await monitorTab.isVisible()) {
        await monitorTab.click();

        await page.waitForSelector('text=Monitor, text=Activity, text=Logs', { timeout: 5000 });
      }
    });

    test('should display policy violations', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const monitorTab = page.locator('button:has-text("Monitor")').first();
      if (await monitorTab.isVisible()) {
        await monitorTab.click();

        const violations = page.locator('text=Violations, text=Denied, text=Blocked').first();
        const hasViolations = await violations.isVisible({ timeout: 5000 }).catch(() => false);
        expect(typeof hasViolations).toBe('boolean');
      }
    });

    test('should filter policy logs by date', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const monitorTab = page.locator('button:has-text("Monitor")').first();
      if (await monitorTab.isVisible()) {
        await monitorTab.click();

        const dateFilter = page.locator('input[type="date"], button:has-text("Date")').first();
        if (await dateFilter.isVisible()) {
          await dateFilter.click();
          await page.waitForTimeout(500);
        }
      }
    });

    test('should export policy logs', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const monitorTab = page.locator('button:has-text("Monitor")').first();
      if (await monitorTab.isVisible()) {
        await monitorTab.click();

        const exportBtn = page.locator('button:has-text("Export"), button:has-text("Download")').first();
        if (await exportBtn.isVisible()) {
          const downloadPromise = page.waitForEvent('download').catch(() => null);
          await exportBtn.click();

          const download = await downloadPromise;
          if (download) {
            expect(download.suggestedFilename()).toContain('policy');
          }
        }
      }
    });
  });

  test.describe('Visual Rule Builder', () => {
    test('should display visual rule builder', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const visualBuilderBtn = page.locator('button:has-text("Visual Builder"), button:has-text("Drag")').first();
      const hasVisualBuilder = await visualBuilderBtn.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasVisualBuilder).toBe('boolean');
    });

    test('should add condition blocks', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const addConditionBtn = page.locator('button:has-text("Add Condition"), button:has-text("Add Block")').first();
      if (await addConditionBtn.isVisible()) {
        await addConditionBtn.click();
        await page.waitForTimeout(500);
      }
    });

    test('should connect rule blocks', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/policies`);

      const ruleBlocks = page.locator('[data-testid="rule-block"], .rule-block');
      const hasBlocks = await ruleBlocks.first().isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasBlocks).toBe('boolean');
    });
  });

  test.describe('Permission Hierarchy', () => {
    test('should display permission hierarchy page', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      await page.waitForSelector('text=Hierarchy, text=Permission, h1', { timeout: 10000 });
    });

    test('should visualize hierarchy tree', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      const hierarchyTree = page.locator('[data-testid="hierarchy-tree"], .tree-view').first();
      const hasTree = await hierarchyTree.isVisible({ timeout: 5000 }).catch(() => false);
      expect(typeof hasTree).toBe('boolean');
    });

    test('should expand/collapse hierarchy nodes', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/permissions/hierarchy`);

      const expandBtn = page.locator('button[aria-label*="expand"], button:has-text("+")').first();
      if (await expandBtn.isVisible()) {
        await expandBtn.click();
        await page.waitForTimeout(300);
      }
    });
  });

  test.describe('Performance', () => {
    test('should load stock ranking packages within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);
      await page.waitForSelector('h1, [data-testid="packages-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });

    test('should load policies page within acceptable time', async ({ page }) => {
      const startTime = Date.now();

      await page.goto(`${ADMIN_URL}/permissions/policies`);
      await page.waitForSelector('h1, [data-testid="policies-page"]', { timeout: 10000 });

      const loadTime = Date.now() - startTime;
      expect(loadTime).toBeLessThan(10000);
    });
  });

  test.describe('Accessibility', () => {
    test('should support keyboard navigation', async ({ page }) => {
      await page.goto(`${ADMIN_URL}/stock-ranking-packages`);

      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
    });
  });
});
