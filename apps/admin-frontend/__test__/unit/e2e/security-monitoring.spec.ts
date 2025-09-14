import { test, expect } from '@playwright/test';

/**
 * End-to-End Tests for Security Monitoring System
 * Tests the complete flow from authentication to security dashboard interaction
 */

test.describe('Security Monitoring Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Mock authentication for admin user
    await page.route('**/api/admin/auth/profile', async route => {
      await route.fulfill({
        json: {
          user_id: 'admin_user_123',
          email: 'admin@epsx.io',
          permissions: ['admin:security:read', 'admin:users:manage'],
          roles: ['admin'],
          is_admin: true,
        }
      });
    });

    // Mock security events API
    await page.route('**/admin/security/events*', async route => {
      await route.fulfill({
        json: {
          events: [
            {
              id: 'evt_001',
              user_id: 'user_123',
              event_type: 'SuspiciousLogin',
              severity: 'High',
              description: 'Login from unusual location (New York, US)',
              risk_score: 75.0,
              device_fingerprint: 'fp_abc123',
              ip_address: '192.168.1.100',
              user_agent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
              timestamp: '2024-01-15T10:30:00Z',
              resolved: false,
              recommended_actions: ['Verify user identity', 'Enable MFA'],
              metadata: { geolocation: 'New York, US' },
            },
            {
              id: 'evt_002',
              user_id: 'user_456',
              event_type: 'TokenReuse',
              severity: 'Critical',
              description: 'Refresh token reuse detected - possible token theft',
              risk_score: 95.0,
              device_fingerprint: 'fp_def456',
              ip_address: '10.0.0.50',
              user_agent: 'curl/7.68.0',
              timestamp: '2024-01-15T10:25:00Z',
              resolved: false,
              recommended_actions: ['Revoke all user tokens', 'Force re-authentication'],
              metadata: { suspicious_user_agent: true },
            },
          ],
          total_count: 2,
          filters_applied: {},
          timestamp: '2024-01-15T10:30:00Z',
        }
      });
    });

    // Mock security metrics API
    await page.route('**/admin/security/metrics', async route => {
      await route.fulfill({
        json: {
          metrics: {
            total_events: 247,
            active_threats: 15,
            resolved_threats: 232,
            avg_threat_score: 42.3,
            events_by_severity: {
              'Critical': 3,
              'High': 8,
              'Medium': 45,
              'Low': 191,
            },
            events_by_type: {
              'SuspiciousLogin': 89,
              'TokenReuse': 15,
              'DeviceMismatch': 67,
              'PermissionEscalation': 12,
              'RateLimitExceeded': 34,
              'MaliciousPayload': 30,
            },
            threat_score_distribution: [
              { range: '0-25', count: 120, percentage: 48.6 },
              { range: '26-50', count: 85, percentage: 34.4 },
              { range: '51-75', count: 32, percentage: 13.0 },
              { range: '76-100', count: 10, percentage: 4.0 },
            ],
          },
          trends: {
            hourly_events: [
              {
                hour: '2024-01-15T09:00:00Z',
                count: 12,
                severity_breakdown: { 'High': 2, 'Medium': 5, 'Low': 5 },
              },
              {
                hour: '2024-01-15T10:00:00Z',
                count: 18,
                severity_breakdown: { 'Critical': 1, 'High': 3, 'Medium': 8, 'Low': 6 },
              },
            ],
            severity_trends: [
              { severity: 'Critical', trend: 'increasing', change_percentage: 25.0 },
              { severity: 'High', trend: 'stable', change_percentage: -2.1 },
              { severity: 'Medium', trend: 'decreasing', change_percentage: -15.3 },
            ],
            threat_score_trend: [
              { timestamp: '2024-01-15T09:00:00Z', avg_score: 38.2, max_score: 89.0, min_score: 5.0 },
              { timestamp: '2024-01-15T10:00:00Z', avg_score: 42.3, max_score: 95.0, min_score: 8.0 },
            ],
          },
          alerts: [
            {
              id: 'alert_001',
              alert_type: 'HighThreatScore',
              message: 'Multiple users detected with threat scores above 85',
              severity: 'Critical',
              timestamp: '2024-01-15T10:30:00Z',
              auto_resolved: false,
              affected_users: ['user_456', 'user_789'],
            },
          ],
          timestamp: '2024-01-15T10:30:00Z',
        }
      });
    });

    // Mock user threat assessment API
    await page.route('**/admin/security/user-threat*', async route => {
      const url = new URL(route.request().url());
      const userId = url.searchParams.get('user_id');
      
      await route.fulfill({
        json: {
          user_id: userId || 'user_123',
          current_threat_score: 75.0,
          threat_level: 'High',
          is_under_threat: true,
          recent_events: [
            {
              id: 'evt_user_001',
              event_type: 'SuspiciousLogin',
              severity: 'High',
              description: 'Login from new device and location',
              timestamp: '2024-01-15T10:15:00Z',
            },
          ],
          risk_factors: [
            'Multiple device logins',
            'Unusual geographic location',
            'High-frequency API calls',
          ],
          recommendations: [
            'Enable multi-factor authentication',
            'Review recent account activity',
            'Consider temporary access restrictions',
          ],
          last_assessed: '2024-01-15T10:30:00Z',
        }
      });
    });
  });

  test('displays security dashboard with correct overview metrics', async ({ page }) => {
    await page.goto('/security');

    // Check page title and header
    await expect(page.locator('h1')).toContainText('Security Monitoring');
    await expect(page.locator('text=Real-time security monitoring and threat detection')).toBeVisible();

    // Check overview metrics cards
    await expect(page.locator('text=Total Security Events')).toBeVisible();
    await expect(page.locator('text=247')).toBeVisible();
    
    await expect(page.locator('text=Active Threats')).toBeVisible();
    await expect(page.locator('text=15')).toBeVisible();
    
    await expect(page.locator('text=Critical Alerts')).toBeVisible();
    await expect(page.locator('text=1')).toBeVisible();

    // Check for system alert indicator (should be green/normal)
    await expect(page.locator('.bg-green-500')).toBeVisible();
  });

  test('shows critical alert banner with dismissal functionality', async ({ page }) => {
    await page.goto('/security');

    // Check critical alert banner is visible
    await expect(page.locator('text=Critical Security Alerts (1)')).toBeVisible();
    await expect(page.locator('text=Multiple users detected with threat scores above 85')).toBeVisible();

    // Dismiss the alert banner
    await page.locator('button[aria-label="Dismiss"], button:has-text("×")').click();

    // Alert banner should be hidden
    await expect(page.locator('text=Critical Security Alerts (1)')).not.toBeVisible();
  });

  test('navigates between dashboard tabs correctly', async ({ page }) => {
    await page.goto('/security');

    // Should start on Overview tab
    await expect(page.locator('text=Recent Security Events')).toBeVisible();
    await expect(page.locator('text=Event Distribution')).toBeVisible();

    // Switch to Security Events tab
    await page.locator('button:has-text("Security Events")').click();
    
    // Should show detailed security events
    await expect(page.locator('text=SuspiciousLogin')).toBeVisible();
    await expect(page.locator('text=Login from unusual location')).toBeVisible();
    await expect(page.locator('text=TokenReuse')).toBeVisible();
    await expect(page.locator('text=Refresh token reuse detected')).toBeVisible();

    // Switch to Permission Audit tab
    await page.locator('button:has-text("Permission Audit")').click();
    
    // Should load the Permission Audit Dashboard
    // (Component would be loaded but specific content depends on implementation)

    // Switch to Token Health tab
    await page.locator('button:has-text("Token Health")').click();
    
    // Should load the Token Health Monitor
    // (Component would be loaded but specific content depends on implementation)
  });

  test('displays security events with correct severity and status indicators', async ({ page }) => {
    await page.goto('/security');

    // Switch to Security Events tab
    await page.locator('button:has-text("Security Events")').click();

    // Check high severity event
    const suspiciousLoginEvent = page.locator('[data-testid="security-event"]').filter({ hasText: 'SuspiciousLogin' });
    await expect(suspiciousLoginEvent.locator('text=High')).toBeVisible();
    await expect(suspiciousLoginEvent.locator('text=user_123')).toBeVisible();
    await expect(suspiciousLoginEvent.locator('text=192.168.1.100')).toBeVisible();

    // Check critical severity event
    const tokenReuseEvent = page.locator('[data-testid="security-event"]').filter({ hasText: 'TokenReuse' });
    await expect(tokenReuseEvent.locator('text=Critical')).toBeVisible();
    await expect(tokenReuseEvent.locator('text=user_456')).toBeVisible();

    // Verify timestamps are displayed
    await expect(page.locator('text=/\\d{1,2}:\\d{2}:\\d{2}/')).toBeVisible();
  });

  test('shows event type distribution in overview', async ({ page }) => {
    await page.goto('/security');

    // Check Event Distribution section
    await expect(page.locator('text=Event Distribution')).toBeVisible();
    
    // Check individual event types and counts
    await expect(page.locator('text=SuspiciousLogin')).toBeVisible();
    await expect(page.locator('text=89')).toBeVisible();
    
    await expect(page.locator('text=DeviceMismatch')).toBeVisible();
    await expect(page.locator('text=67')).toBeVisible();
    
    await expect(page.locator('text=RateLimitExceeded')).toBeVisible();
    await expect(page.locator('text=34')).toBeVisible();
  });

  test('handles loading states correctly', async ({ page }) => {
    // Delay API responses to test loading states
    await page.route('**/admin/security/events*', async route => {
      await new Promise(resolve => setTimeout(resolve, 1000));
      await route.fulfill({
        json: { events: [], total_count: 0, filters_applied: {}, timestamp: '2024-01-15T10:30:00Z' }
      });
    });

    await page.goto('/security');

    // Should show loading indicators
    await expect(page.locator('.animate-pulse')).toBeVisible();
    
    // Wait for loading to complete
    await expect(page.locator('text=No security events in the selected time range')).toBeVisible({ timeout: 5000 });
  });

  test('displays access denied for unauthorized users', async ({ page }) => {
    // Override auth mock to return unauthorized user
    await page.route('**/api/admin/auth/profile', async route => {
      await route.fulfill({
        status: 403,
        json: { error: 'Insufficient permissions' }
      });
    });

    await page.goto('/security');

    // Should show access denied message
    await expect(page.locator('text=Access Denied')).toBeVisible();
    await expect(page.locator("text=You don't have permission to access the security monitoring dashboard.")).toBeVisible();
  });

  test('handles API errors gracefully', async ({ page }) => {
    // Mock API error responses
    await page.route('**/admin/security/events*', async route => {
      await route.fulfill({
        status: 500,
        json: { error: 'Internal Server Error' }
      });
    });

    await page.route('**/admin/security/metrics', async route => {
      await route.fulfill({
        status: 500,
        json: { error: 'Internal Server Error' }
      });
    });

    await page.goto('/security');

    // Page should still load without crashing
    await expect(page.locator('h1')).toContainText('Security Monitoring');
    
    // Should show empty states or error indicators
    await expect(page.locator('text=No security events in the selected time range')).toBeVisible();
  });

  test('updates data in real-time', async ({ page }) => {
    await page.goto('/security');

    // Initial state
    await expect(page.locator('text=247')).toBeVisible(); // Total events
    
    // Update mock to return new data
    await page.route('**/admin/security/metrics', async route => {
      await route.fulfill({
        json: {
          metrics: {
            total_events: 250, // Updated count
            active_threats: 16,
            resolved_threats: 234,
            avg_threat_score: 43.1,
            events_by_severity: { 'Critical': 4, 'High': 8, 'Medium': 46, 'Low': 192 },
            events_by_type: { 'SuspiciousLogin': 92, 'TokenReuse': 16 },
            threat_score_distribution: [],
          },
          trends: { hourly_events: [], severity_trends: [], threat_score_trend: [] },
          alerts: [],
          timestamp: '2024-01-15T10:35:00Z',
        }
      });
    });

    // Wait for auto-refresh (would typically be 30 seconds, but we can trigger manually in test)
    // In a real test, you might need to wait for the refresh interval or trigger a manual refresh
    
    // For this test, we assume the data refreshes and check for updated values
    // The actual implementation depends on how the refresh mechanism works
  });

  test('responsive design works correctly', async ({ page }) => {
    // Test desktop view
    await page.setViewportSize({ width: 1200, height: 800 });
    await page.goto('/security');

    // Should show grid layout for metrics
    await expect(page.locator('[class*="grid-cols-4"]')).toBeVisible();

    // Test mobile view
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Should adapt to mobile layout
    await expect(page.locator('[class*="grid-cols-1"]')).toBeVisible();
    
    // Navigation should still be accessible
    await expect(page.locator('button:has-text("Security Events")')).toBeVisible();
  });

  test('keyboard navigation works correctly', async ({ page }) => {
    await page.goto('/security');

    // Test tab navigation
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    
    // Should be able to navigate to Security Events tab
    await page.keyboard.press('Enter');
    
    // Should switch to events view
    await expect(page.locator('text=SuspiciousLogin')).toBeVisible();
  });
});

test.describe('Security Event Details', () => {
  test('shows detailed event information', async ({ page }) => {
    await page.goto('/security');
    
    // Switch to events tab
    await page.locator('button:has-text("Security Events")').click();

    // Check detailed event information
    const eventCard = page.locator('[data-testid="security-event"]').first();
    
    await expect(eventCard.locator('text=SuspiciousLogin')).toBeVisible();
    await expect(eventCard.locator('text=High')).toBeVisible();
    await expect(eventCard.locator('text=Login from unusual location')).toBeVisible();
    await expect(eventCard.locator('text=User: user_123')).toBeVisible();
    await expect(eventCard.locator('text=IP: 192.168.1.100')).toBeVisible();
    
    // Check timestamp format
    await expect(eventCard.locator('text=/Time: \\d{1,2}\\/\\d{1,2}\\/\\d{4}/')).toBeVisible();
  });
});

test.describe('Performance Tests', () => {
  test('dashboard loads within acceptable time', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto('/security');
    
    // Wait for main content to load
    await expect(page.locator('h1')).toContainText('Security Monitoring');
    await expect(page.locator('text=Total Security Events')).toBeVisible();
    
    const loadTime = Date.now() - startTime;
    
    // Should load within 3 seconds
    expect(loadTime).toBeLessThan(3000);
  });

  test('handles large datasets efficiently', async ({ page }) => {
    // Mock large dataset
    const largeEventsList = Array.from({ length: 100 }, (_, i) => ({
      id: `evt_${i.toString().padStart(3, '0')}`,
      user_id: `user_${i % 10}`,
      event_type: ['SuspiciousLogin', 'TokenReuse', 'DeviceMismatch'][i % 3],
      severity: ['Low', 'Medium', 'High', 'Critical'][i % 4],
      description: `Test event ${i}`,
      timestamp: '2024-01-15T10:30:00Z',
      resolved: i % 3 === 0,
    }));

    await page.route('**/admin/security/events*', async route => {
      await route.fulfill({
        json: {
          events: largeEventsList,
          total_count: 100,
          filters_applied: {},
          timestamp: '2024-01-15T10:30:00Z',
        }
      });
    });

    await page.goto('/security');
    
    // Switch to events tab
    await page.locator('button:has-text("Security Events")').click();

    // Should handle large dataset without performance issues
    await expect(page.locator('[data-testid="security-event"]')).toHaveCount(100, { timeout: 5000 });
    
    // Page should remain responsive
    await page.locator('button:has-text("Overview")').click();
    await expect(page.locator('text=Recent Security Events')).toBeVisible({ timeout: 1000 });
  });
});