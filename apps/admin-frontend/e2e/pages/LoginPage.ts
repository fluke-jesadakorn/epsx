import { Page, expect } from '@playwright/test';

export class LoginPage {
  constructor(private page: Page) {}

  async goto() {
    await this.page.goto('/login');
    await expect(this.page.getByRole('heading', { name: /admin sign in/i })).toBeVisible();
  }

  async login(email: string, password: string) {
    await this.page.getByLabel(/email/i).fill(email);
    await this.page.getByRole('textbox', { name: /password/i }).fill(password);
    await this.page.getByRole('button', { name: /sign in/i }).click();
  }

  async verifyLoginError(errorMessage: string) {
    const errorElement = this.page.locator('[role="alert"]').or(
      this.page.locator('.error-message')
    );
    await expect(errorElement).toBeVisible();
    await expect(errorElement).toContainText(errorMessage);
  }

  async verifySuccessfulLogin() {
    await this.page.waitForURL(/\/$|\/admin/);
  }
}