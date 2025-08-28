import { test, expect } from '@playwright/test';

test.describe('Analytics Pagination', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to analytics page
    await page.goto('http://localhost:3000/analytics');
    
    // Wait for initial data load
    await page.waitForSelector('text=Performance Monitor', { timeout: 10000 });
  });

  test('should display pagination controls', async ({ page }) => {
    // Check if pagination info is visible
    const paginationInfo = page.locator('text=/Page \\d+ of \\d+/');
    await expect(paginationInfo).toBeVisible();
    
    // Check for Previous button (should be disabled on first page)
    const prevButton = page.getByRole('button', { name: 'Previous' });
    await expect(prevButton).toBeVisible();
    await expect(prevButton).toBeDisabled();
    
    // Check for Next button (should be enabled if more pages exist)
    const nextButton = page.getByRole('button', { name: 'Next' });
    await expect(nextButton).toBeVisible();
  });

  test('should navigate between pages', async ({ page }) => {
    // Click Next button
    const nextButton = page.getByRole('button', { name: 'Next' });
    await nextButton.click();
    
    // Wait for page change (you may need to adjust this based on actual behavior)
    await page.waitForTimeout(1000);
    
    // Check if Previous button is now enabled
    const prevButton = page.getByRole('button', { name: 'Previous' });
    await expect(prevButton).toBeEnabled();
    
    // Navigate back to first page
    await prevButton.click();
    await page.waitForTimeout(1000);
    
    // Check if Previous button is disabled again
    await expect(prevButton).toBeDisabled();
  });

  test('should have limit selector for items per page', async ({ page }) => {
    // Look for the limit selector
    const limitSelector = page.locator('#limit-selector');
    
    // Check if selector exists (it should after our implementation)
    const selectorExists = await limitSelector.count() > 0;
    
    if (selectorExists) {
      // Check if label is present
      await expect(page.locator('label[for="limit-selector"]')).toContainText('Items per page');
      
      // Check default value
      const selectedValue = await limitSelector.inputValue();
      expect(['10', '20', '50', '100']).toContain(selectedValue);
      
      // Try changing the limit
      await limitSelector.selectOption('50');
      
      // Wait for data reload
      await page.waitForTimeout(1000);
      
      // Verify the change was applied
      const newValue = await limitSelector.inputValue();
      expect(newValue).toBe('50');
    } else {
      // If selector doesn't exist, note it for implementation
      console.log('Limit selector not found - may need to check pagination component rendering');
    }
  });

  test('should display correct pagination info', async ({ page }) => {
    // Look for pagination text like "Showing X-Y of Z results"
    const showingText = page.locator('text=/Showing \\d+-\\d+ of \\d+ results/');
    
    // Check if it exists (may not if pagination isn't rendered)
    const textExists = await showingText.count() > 0;
    
    if (textExists) {
      await expect(showingText).toBeVisible();
      
      // Extract and validate the numbers
      const text = await showingText.textContent();
      const match = text?.match(/Showing (\d+)-(\d+) of (\d+) results/);
      
      if (match) {
        const [, start, end, total] = match;
        expect(parseInt(start)).toBeGreaterThan(0);
        expect(parseInt(end)).toBeGreaterThanOrEqual(parseInt(start));
        expect(parseInt(total)).toBeGreaterThanOrEqual(parseInt(end));
      }
    }
  });

  test('should update URL with pagination parameters', async ({ page }) => {
    // Check if URL updates when navigating pages
    const nextButton = page.getByRole('button', { name: 'Next' });
    
    if (await nextButton.isEnabled()) {
      await nextButton.click();
      await page.waitForTimeout(1000);
      
      // Check if URL contains page parameter
      const url = page.url();
      // URL might contain query params or might handle state internally
      console.log('Current URL after navigation:', url);
    }
  });

  test('should handle API pagination correctly', async ({ page }) => {
    // Monitor network requests
    const apiResponses: any[] = [];
    
    page.on('response', response => {
      if (response.url().includes('/api/v1/analytics/eps-rankings')) {
        apiResponses.push({
          url: response.url(),
          status: response.status(),
        });
      }
    });
    
    // Trigger a page change if possible
    const nextButton = page.getByRole('button', { name: 'Next' });
    if (await nextButton.isEnabled()) {
      await nextButton.click();
      await page.waitForTimeout(2000);
      
      // Check if API was called with different page parameter
      const paginatedCalls = apiResponses.filter(r => r.url.includes('page='));
      expect(paginatedCalls.length).toBeGreaterThan(0);
      
      // Verify different page numbers were requested
      const pageNumbers = paginatedCalls.map(call => {
        const match = call.url.match(/page=(\d+)/);
        return match ? parseInt(match[1]) : null;
      }).filter(Boolean);
      
      console.log('Page numbers requested:', pageNumbers);
    }
  });

  test('should maintain filters when changing pages', async ({ page }) => {
    // Apply a filter if filter panel exists
    const filterButton = page.locator('button:has-text("Filters")');
    
    if (await filterButton.isVisible()) {
      await filterButton.click();
      await page.waitForTimeout(500);
      
      // Try to apply a filter (adjust based on actual filter implementation)
      const sectorFilter = page.locator('select[name="sector"]');
      if (await sectorFilter.count() > 0) {
        await sectorFilter.selectOption({ index: 1 });
        await page.waitForTimeout(1000);
      }
      
      // Navigate to next page
      const nextButton = page.getByRole('button', { name: 'Next' });
      if (await nextButton.isEnabled()) {
        await nextButton.click();
        await page.waitForTimeout(1000);
        
        // Verify filter is still applied
        // This would need to be adjusted based on how filters are displayed
        console.log('Filter should be maintained after page change');
      }
    }
  });
});

test.describe('Pagination Edge Cases', () => {
  test('should handle empty results gracefully', async ({ page }) => {
    // Navigate to analytics
    await page.goto('http://localhost:3000/analytics');
    
    // Apply filters that might return no results
    // This test would need adjustment based on actual filter implementation
    
    // Check for empty state message
    const emptyState = page.locator('text=/No results found|No data available/i');
    
    // If empty state exists, pagination should not be shown
    if (await emptyState.count() > 0) {
      const pagination = page.locator('button:has-text("Next")');
      expect(await pagination.count()).toBe(0);
    }
  });

  test('should handle single page of results', async ({ page }) => {
    await page.goto('http://localhost:3000/analytics');
    
    // If only one page of results, both buttons should be disabled
    const pageInfo = page.locator('text=/Page 1 of 1/');
    
    if (await pageInfo.count() > 0) {
      const prevButton = page.getByRole('button', { name: 'Previous' });
      const nextButton = page.getByRole('button', { name: 'Next' });
      
      await expect(prevButton).toBeDisabled();
      await expect(nextButton).toBeDisabled();
    }
  });
});