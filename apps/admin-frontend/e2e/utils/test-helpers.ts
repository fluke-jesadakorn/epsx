import { Page, expect, Locator } from '@playwright/test';

export class TestHelpers {
  constructor(private page: Page) {}

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async waitForElement(selector: string, timeout: number = 10000): Promise<Locator> {
    const element = this.page.locator(selector);
    await expect(element).toBeVisible({ timeout });
    return element;
  }

  async clickAndWait(selector: string, waitForSelector?: string) {
    await this.page.click(selector);
    if (waitForSelector) {
      await this.waitForElement(waitForSelector);
    } else {
      await this.waitForPageLoad();
    }
  }

  async fillFormField(label: string, value: string) {
    const field = this.page.getByLabel(new RegExp(label, 'i'));
    await expect(field).toBeVisible();
    await field.fill(value);
  }

  async selectOption(selectLabel: string, optionText: string) {
    const select = this.page.getByLabel(new RegExp(selectLabel, 'i'));
    await expect(select).toBeVisible();
    await select.click();
    await this.page.getByText(optionText).click();
  }

  async verifyNotification(message: string, type: 'success' | 'error' | 'info' = 'success') {
    const notification = this.page.locator(`[data-testid="${type}-notification"]`).or(
      this.page.locator('.toast').or(
        this.page.locator('[role="alert"]')
      )
    );
    
    await expect(notification).toBeVisible();
    await expect(notification).toContainText(message);
  }

  async verifyTableData(columnHeader: string, expectedData: string[]) {
    const table = this.page.locator('table').first();
    await expect(table).toBeVisible();
    
    // Find column index
    const headers = table.locator('thead th');
    const headerCount = await headers.count();
    let columnIndex = -1;
    
    for (let i = 0; i < headerCount; i++) {
      const headerText = await headers.nth(i).textContent();
      if (headerText?.toLowerCase().includes(columnHeader.toLowerCase())) {
        columnIndex = i;
        break;
      }
    }
    
    expect(columnIndex).toBeGreaterThan(-1);
    
    // Verify data in column
    const cells = table.locator(`tbody tr td:nth-child(${columnIndex + 1})`);
    const cellCount = await cells.count();
    
    for (let i = 0; i < Math.min(cellCount, expectedData.length); i++) {
      await expect(cells.nth(i)).toContainText(expectedData[i]);
    }
  }

  async searchInTable(searchTerm: string) {
    const searchInput = this.page.getByPlaceholder(/search/i).or(
      this.page.getByRole('textbox', { name: /search/i })
    );
    
    if (await searchInput.isVisible()) {
      await searchInput.fill(searchTerm);
      await this.waitForPageLoad();
    }
  }

  async navigateToSection(sectionName: string) {
    const nav = this.page.locator('nav').or(this.page.locator('[role="navigation"]'));
    const link = nav.getByRole('link', { name: new RegExp(sectionName, 'i') });
    
    await expect(link).toBeVisible();
    await link.click();
    await this.waitForPageLoad();
  }
}