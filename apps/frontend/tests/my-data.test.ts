import { test, expect } from '@playwright/test';

test('My Data page loads and displays portfolio positions', async ({ page }) => {
  // Navigate to the My Data page
  await page.goto('/my-data');
  
  // Check that the page title is correct
  await expect(page).toHaveTitle(/My Data - EPSX Analytics/);
  
  // Check that the header is visible
  await expect(page.locator('text=My Portfolio Analytics')).toBeVisible();
  
  // Check that portfolio positions are displayed
  await expect(page.locator('text=MSFT #1')).toBeVisible();
  await expect(page.locator('text=AMZN #2')).toBeVisible();
  await expect(page.locator('text=META #3')).toBeVisible();
  await expect(page.locator('text=TSLA #4')).toBeVisible();
  
  // Check that action buttons are present
  await expect(page.locator('button:has-text("KEEP")')).toHaveCount(4);
  
  // Check that the refresh button works
  const initialProcessingTime = await page.locator('text=Processed in').textContent();
  await page.click('button:has-text("Refresh")');
  await page.waitForTimeout(1500); // Wait for the mock API call to complete
  const newProcessingTime = await page.locator('text=Processed in').textContent();
  expect(newProcessingTime).not.toEqual(initialProcessingTime);
  
  // Check that action toggle works
  const firstKeepButton = page.locator('button:has-text("KEEP")').first();
  await firstKeepButton.click();
  await expect(firstKeepButton).toHaveText(/STOP/);
  
  // Toggle back
  await firstKeepButton.click();
  await expect(firstKeepButton).toHaveText(/KEEP/);
});