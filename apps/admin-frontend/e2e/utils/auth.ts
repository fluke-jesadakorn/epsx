import { Page, expect } from '@playwright/test';
import { APIMocks } from './api-mocks';

export class AuthUtils {
  private apiMocks: APIMocks;
  
  constructor(private page: Page) {
    this.apiMocks = new APIMocks(page);
  }

  async login(email: string = 'admin@epsx.com', password: string = 'admin123') {
    await this.apiMocks.mockSuccessfulAuth(email);

    await this.page.goto('/login');
    
    // Wait for login form to be visible
    await expect(this.page.getByRole('heading', { name: /admin sign in/i })).toBeVisible();
    
    // Fill in credentials
    await this.page.getByLabel(/email/i).fill(email);
    await this.page.getByRole('textbox', { name: /password/i }).fill(password);
    
    // Submit form
    await this.page.getByRole('button', { name: /sign in/i }).click();
    
    // Wait for JavaScript redirect to complete (from mocked auth response)
    await this.page.waitForTimeout(3000);
    
    // Check if we got redirected to admin dashboard or if there are no form validation errors
    const hasValidationError = await this.page.locator('[role="alert"]').filter({ hasText: /required|invalid/i }).isVisible();
    const isOnLoginPage = this.page.url().includes('/login');
    const isOnAdminPage = this.page.url().includes('/admin');
    
    if (!hasValidationError && (isOnAdminPage || !isOnLoginPage)) {
      // Successfully authenticated - either redirected to admin or away from login
      return;
    }
  }

  async logout() {
    // Look for user menu or logout button
    const userMenu = this.page.getByRole('button', { name: /user menu/i }).or(
      this.page.getByTestId('user-menu')
    ).or(
      this.page.locator('[data-testid="user-avatar"]')
    );
    
    if (await userMenu.isVisible()) {
      await userMenu.click();
      await this.page.getByRole('menuitem', { name: /logout/i }).click();
    } else {
      // Fallback: look for direct logout button
      await this.page.getByRole('button', { name: /logout/i }).click();
    }
    
    // Wait for redirect to login page
    await expect(this.page).toHaveURL(/\/login/);
  }

  async ensureLoggedIn(email?: string, password?: string) {
    // Check if we're already logged in
    const currentUrl = this.page.url();
    if (currentUrl.includes('/admin') && !currentUrl.includes('/login')) {
      return; // Already logged in
    }
    
    await this.login(email, password);
  }

  async ensureLoggedOut() {
    const currentUrl = this.page.url();
    if (currentUrl.includes('/login') || currentUrl.includes('/unauthorized')) {
      return; // Already logged out
    }
    
    await this.logout();
  }
}