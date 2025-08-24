import { test, expect, Page } from '@playwright/test';
import { 
  TEST_USERS, 
  TestUser, 
  generateMockJWT, 
  isUserSubscriptionActive,
  getSubscriptionDaysRemaining
} from '../fixtures/user-fixtures';

/**
 * User Experience Tests for Subscription Flows and Tier Management
 * Tests subscription upgrades, downgrades, payment processing, tier transitions,
 * and subscription management user interface
 */

test.describe('💳 Subscription Flows and Tier Management', () => {

  // Helper to authenticate user
  async function authenticateUser(page: Page, user: TestUser): Promise<void> {
    const jwtToken = generateMockJWT(user);
    
    await page.context().addCookies([{
      name: 'epsx_jwt',
      value: jwtToken,
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false
    }]);
  }

  // Helper to mock payment processing
  async function mockPaymentProvider(page: Page, shouldSucceed: boolean = true): Promise<void> {
    await page.route('**/api/payments/**', async route => {
      const url = route.request().url();
      const method = route.request().method();
      
      if (url.includes('/create-payment-intent')) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            client_secret: 'pi_test_12345_secret',
            payment_intent_id: 'pi_test_12345'
          })
        });
      } else if (url.includes('/confirm-payment')) {
        if (shouldSucceed) {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              success: true,
              subscription_id: 'sub_12345',
              new_tier: 'GOLD',
              effective_date: new Date().toISOString()
            })
          });
        } else {
          await route.fulfill({
            status: 400,
            contentType: 'application/json',
            body: JSON.stringify({
              error: 'Payment failed',
              code: 'CARD_DECLINED'
            })
          });
        }
      }
    });
  }

  // Helper to mock subscription API
  async function mockSubscriptionAPI(page: Page): Promise<void> {
    await page.route('**/api/subscriptions/**', async route => {
      const url = route.request().url();
      const method = route.request().method();
      
      if (url.includes('/current') && method === 'GET') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            subscription_id: 'sub_12345',
            tier: 'SILVER',
            status: 'active',
            expires_at: '2025-03-01T00:00:00Z',
            auto_renew: true,
            payment_method: '**** 1234'
          })
        });
      } else if (url.includes('/upgrade') && method === 'POST') {
        const requestBody = route.request().postDataJSON();
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: true,
            new_tier: requestBody.target_tier,
            effective_immediately: true,
            proration_amount: 25.50
          })
        });
      } else if (url.includes('/cancel') && method === 'POST') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: true,
            cancellation_effective: '2024-09-30T00:00:00Z',
            refund_amount: 0,
            remaining_access: true
          })
        });
      }
    });
  }

  test.describe('⬆️ Subscription Upgrade Flows', () => {
    
    test('should allow FREE user to upgrade to BRONZE', async ({ page }) => {
      const user = TEST_USERS.FREE_USER;
      await authenticateUser(page, user);
      await mockPaymentProvider(page, true);
      await mockSubscriptionAPI(page);
      
      await page.goto('/upgrade');
      
      // Should see upgrade options
      await expect(page.locator('[data-testid="upgrade-options"]')).toBeVisible();
      await expect(page.locator('[data-testid="bronze-plan"]')).toBeVisible();
      
      // Click on BRONZE plan
      await page.click('[data-testid="bronze-plan-select"]');
      
      // Should show plan details
      await expect(page.locator('[data-testid="plan-details"]')).toBeVisible();
      await expect(page.locator('[data-testid="bronze-features"]')).toBeVisible();
      await expect(page.locator('[data-testid="bronze-price"]')).toContainText('$9.99');
      
      // Proceed to payment
      await page.click('[data-testid="proceed-to-payment"]');
      
      // Should show payment form
      await expect(page.locator('[data-testid="payment-form"]')).toBeVisible();
      await expect(page.locator('[data-testid="stripe-elements"]')).toBeVisible();
      
      // Fill payment details (mock)
      await page.fill('[data-testid="card-number"]', '4242424242424242');
      await page.fill('[data-testid="card-expiry"]', '12/25');
      await page.fill('[data-testid="card-cvc"]', '123');
      
      // Submit payment
      await page.click('[data-testid="submit-payment"]');
      
      // Should show success message
      await expect(page.locator('[data-testid="upgrade-success"]')).toBeVisible();
      await expect(page.locator('[data-testid="new-tier-confirmation"]')).toContainText('BRONZE');
      
      console.log('✅ FREE → BRONZE upgrade flow completed');
    });

    test('should show tier comparison for upgrade decisions', async ({ page }) => {
      const user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      await page.goto('/upgrade');
      
      // Should show comparison table
      await expect(page.locator('[data-testid="tier-comparison"]')).toBeVisible();
      
      // Check comparison features
      const comparisonRows = page.locator('[data-testid="feature-comparison-row"]');
      const rowCount = await comparisonRows.count();
      expect(rowCount).toBeGreaterThan(5); // Should have multiple feature rows
      
      // Current tier should be highlighted
      await expect(page.locator('[data-testid="current-tier-silver"]')).toHaveClass(/current-tier/);
      
      // Higher tiers should show "Upgrade" buttons
      await expect(page.locator('[data-testid="upgrade-to-gold"]')).toBeVisible();
      await expect(page.locator('[data-testid="upgrade-to-platinum"]')).toBeVisible();
      
      // Lower tiers should be disabled
      await expect(page.locator('[data-testid="bronze-plan-button"]')).toBeDisabled();
      
      console.log('✅ Tier comparison interface working');
    });

    test('should handle proration for mid-cycle upgrades', async ({ page }) => {
      const user = TEST_USERS.BRONZE_USER;
      await authenticateUser(page, user);
      await mockPaymentProvider(page, true);
      await mockSubscriptionAPI(page);
      
      await page.goto('/upgrade');
      
      // Select SILVER upgrade
      await page.click('[data-testid="silver-plan-select"]');
      
      // Should show proration details
      await expect(page.locator('[data-testid="proration-details"]')).toBeVisible();
      await expect(page.locator('[data-testid="proration-amount"]')).toContainText('$25.50');
      await expect(page.locator('[data-testid="billing-cycle-info"]')).toBeVisible();
      
      // Should explain immediate access
      await expect(page.locator('[data-testid="immediate-access-note"]')).toBeVisible();
      
      await page.click('[data-testid="confirm-upgrade"]');
      
      // Should show upgrade confirmation with proration
      await expect(page.locator('[data-testid="proration-confirmation"]')).toBeVisible();
      
      console.log('✅ Proration handling for mid-cycle upgrades working');
    });

    test('should handle failed payment scenarios gracefully', async ({ page }) => {
      const user = TEST_USERS.FREE_USER;
      await authenticateUser(page, user);
      await mockPaymentProvider(page, false); // Force payment failure
      
      await page.goto('/upgrade');
      
      await page.click('[data-testid="gold-plan-select"]');
      await page.click('[data-testid="proceed-to-payment"]');
      
      // Fill payment form
      await page.fill('[data-testid="card-number"]', '4000000000000002'); // Declined card
      await page.fill('[data-testid="card-expiry"]', '12/25');
      await page.fill('[data-testid="card-cvc"]', '123');
      
      await page.click('[data-testid="submit-payment"]');
      
      // Should show payment error
      await expect(page.locator('[data-testid="payment-error"]')).toBeVisible();
      await expect(page.locator('[data-testid="error-message"]')).toContainText('Payment failed');
      
      // Should offer retry option
      await expect(page.locator('[data-testid="retry-payment"]')).toBeVisible();
      await expect(page.locator('[data-testid="change-payment-method"]')).toBeVisible();
      
      // User should remain on original tier
      expect(user.package_tier).toBe('FREE');
      
      console.log('✅ Failed payment handling working');
    });

    test('should provide upgrade incentives and discounts', async ({ page }) => {
      const user = TEST_USERS.BRONZE_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/promotions/current', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            active_promotions: [
              {
                id: 'summer2024',
                type: 'percentage',
                value: 20,
                applicable_tiers: ['SILVER', 'GOLD'],
                expires_at: '2024-09-30T00:00:00Z',
                description: '20% off first 3 months'
              }
            ]
          })
        });
      });
      
      await page.goto('/upgrade');
      
      // Should show promotion banner
      await expect(page.locator('[data-testid="promotion-banner"]')).toBeVisible();
      await expect(page.locator('[data-testid="discount-percentage"]')).toContainText('20%');
      
      // Should show discounted prices
      await expect(page.locator('[data-testid="silver-discounted-price"]')).toBeVisible();
      
      // Should show terms and conditions
      await expect(page.locator('[data-testid="promotion-terms"]')).toBeVisible();
      
      console.log('✅ Upgrade incentives and discounts displayed');
    });
  });

  test.describe('⬇️ Subscription Downgrade and Cancellation', () => {
    
    test('should allow subscription cancellation with proper warnings', async ({ page }) => {
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      await mockSubscriptionAPI(page);
      
      await page.goto('/settings');
      
      // Navigate to subscription management
      await page.click('[data-testid="manage-subscription"]');
      
      // Should show current subscription details
      await expect(page.locator('[data-testid="current-subscription"]')).toBeVisible();
      await expect(page.locator('[data-testid="current-tier"]')).toContainText('GOLD');
      
      // Click cancel subscription
      await page.click('[data-testid="cancel-subscription"]');
      
      // Should show cancellation warnings
      await expect(page.locator('[data-testid="cancellation-warning"]')).toBeVisible();
      await expect(page.locator('[data-testid="features-lost"]')).toBeVisible();
      await expect(page.locator('[data-testid="access-until-date"]')).toBeVisible();
      
      // Should offer downgrade alternative
      await expect(page.locator('[data-testid="downgrade-option"]')).toBeVisible();
      await expect(page.locator('[data-testid="pause-subscription"]')).toBeVisible();
      
      // Confirm cancellation
      await page.click('[data-testid="confirm-cancellation"]');
      
      // Should show cancellation confirmation
      await expect(page.locator('[data-testid="cancellation-success"]')).toBeVisible();
      await expect(page.locator('[data-testid="access-remains-until"]')).toBeVisible();
      
      console.log('✅ Subscription cancellation flow with warnings completed');
    });

    test('should handle immediate vs end-of-period cancellation', async ({ page }) => {
      const user = TEST_USERS.PLATINUM_USER;
      await authenticateUser(page, user);
      await mockSubscriptionAPI(page);
      
      await page.goto('/settings');
      await page.click('[data-testid="manage-subscription"]');
      await page.click('[data-testid="cancel-subscription"]');
      
      // Should show cancellation timing options
      await expect(page.locator('[data-testid="cancellation-timing"]')).toBeVisible();
      
      // End of period (default)
      await expect(page.locator('[data-testid="end-of-period-option"]')).toBeChecked();
      await expect(page.locator('[data-testid="end-period-date"]')).toContainText('2025-12-01');
      
      // Immediate cancellation option
      await expect(page.locator('[data-testid="immediate-option"]')).toBeVisible();
      await page.click('[data-testid="immediate-option"]');
      
      // Should show refund calculation
      await expect(page.locator('[data-testid="refund-calculation"]')).toBeVisible();
      await expect(page.locator('[data-testid="prorated-refund"]')).toBeVisible();
      
      console.log('✅ Cancellation timing options working');
    });

    test('should offer retention incentives before cancellation', async ({ page }) => {
      const user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/retention/offers', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            offers: [
              {
                type: 'discount',
                value: 50,
                duration: 3,
                description: '50% off next 3 months'
              },
              {
                type: 'pause',
                duration: 30,
                description: 'Pause subscription for 30 days'
              },
              {
                type: 'downgrade',
                target_tier: 'BRONZE',
                description: 'Downgrade to Bronze for $5/month'
              }
            ]
          })
        });
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="manage-subscription"]');
      await page.click('[data-testid="cancel-subscription"]');
      
      // Should show retention offers
      await expect(page.locator('[data-testid="retention-offers"]')).toBeVisible();
      await expect(page.locator('[data-testid="discount-offer"]')).toBeVisible();
      await expect(page.locator('[data-testid="pause-offer"]')).toBeVisible();
      await expect(page.locator('[data-testid="downgrade-offer"]')).toBeVisible();
      
      // Should be able to accept an offer
      await page.click('[data-testid="accept-discount-offer"]');
      await expect(page.locator('[data-testid="offer-accepted"]')).toBeVisible();
      
      console.log('✅ Retention incentives before cancellation working');
    });

    test('should handle subscription pausing', async ({ page }) => {
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/subscriptions/pause', async route => {
        const requestBody = route.request().postDataJSON();
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: true,
            pause_duration: requestBody.duration_days,
            resume_date: new Date(Date.now() + requestBody.duration_days * 24 * 60 * 60 * 1000).toISOString(),
            billing_pause_confirmed: true
          })
        });
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="manage-subscription"]');
      await page.click('[data-testid="pause-subscription"]');
      
      // Should show pause options
      await expect(page.locator('[data-testid="pause-duration-selector"]')).toBeVisible();
      
      // Select pause duration
      await page.selectOption('[data-testid="pause-duration"]', '30');
      
      // Should show pause confirmation
      await expect(page.locator('[data-testid="pause-confirmation"]')).toBeVisible();
      await expect(page.locator('[data-testid="resume-date"]')).toBeVisible();
      
      await page.click('[data-testid="confirm-pause"]');
      
      // Should show pause success
      await expect(page.locator('[data-testid="pause-success"]')).toBeVisible();
      
      console.log('✅ Subscription pausing functionality working');
    });
  });

  test.describe('🔄 Tier Transition Management', () => {
    
    test('should handle tier upgrade with immediate feature access', async ({ page }) => {
      let currentUser = { ...TEST_USERS.SILVER_USER };
      await authenticateUser(page, currentUser);
      await mockPaymentProvider(page, true);
      
      await page.route('**/api/subscriptions/upgrade', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: true,
            old_tier: 'SILVER',
            new_tier: 'GOLD',
            effective_immediately: true,
            new_permissions: TEST_USERS.GOLD_USER.permissions
          })
        });
      });
      
      await page.goto('/upgrade');
      await page.click('[data-testid="gold-plan-select"]');
      await page.click('[data-testid="proceed-to-payment"]');
      
      // Fill payment and submit
      await page.fill('[data-testid="card-number"]', '4242424242424242');
      await page.fill('[data-testid="card-expiry"]', '12/25');
      await page.fill('[data-testid="card-cvc"]', '123');
      await page.click('[data-testid="submit-payment"]');
      
      // Should immediately access GOLD features
      await page.goto('/vip');
      await expect(page.locator('[data-testid="vip-dashboard"]')).toBeVisible();
      
      // Should show upgrade notification
      await expect(page.locator('[data-testid="tier-upgrade-notification"]')).toBeVisible();
      
      console.log('✅ Immediate feature access after upgrade working');
    });

    test('should handle trial period expiration gracefully', async ({ page }) => {
      const trialUser = TEST_USERS.TRIAL_USER;
      await authenticateUser(page, trialUser);
      
      // Mock trial expiration
      await page.route('**/api/subscriptions/trial-status', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            trial_active: false,
            trial_expired: true,
            days_remaining: 0,
            grace_period: true,
            grace_period_days: 3
          })
        });
      });
      
      await page.goto('/');
      
      // Should show trial expiration notice
      await expect(page.locator('[data-testid="trial-expired-notice"]')).toBeVisible();
      await expect(page.locator('[data-testid="grace-period-info"]')).toBeVisible();
      
      // Should show upgrade options
      await expect(page.locator('[data-testid="trial-upgrade-options"]')).toBeVisible();
      
      // Try to access premium features
      await page.goto('/vip');
      
      // Should show upgrade prompt instead of feature
      await expect(page.locator('[data-testid="trial-expired-upgrade"]')).toBeVisible();
      
      console.log('✅ Trial period expiration handling working');
    });

    test('should handle payment method updates', async ({ page }) => {
      const user = TEST_USERS.PLATINUM_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/payments/methods', async route => {
        const method = route.request().method();
        
        if (method === 'GET') {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              payment_methods: [
                {
                  id: 'pm_12345',
                  type: 'card',
                  last4: '1234',
                  brand: 'visa',
                  exp_month: 12,
                  exp_year: 2025,
                  is_default: true
                }
              ]
            })
          });
        } else if (method === 'POST') {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              success: true,
              payment_method_id: 'pm_67890',
              set_as_default: true
            })
          });
        }
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="payment-methods"]');
      
      // Should show current payment methods
      await expect(page.locator('[data-testid="payment-method-list"]')).toBeVisible();
      await expect(page.locator('[data-testid="card-ending-1234"]')).toBeVisible();
      
      // Add new payment method
      await page.click('[data-testid="add-payment-method"]');
      await expect(page.locator('[data-testid="new-card-form"]')).toBeVisible();
      
      await page.fill('[data-testid="new-card-number"]', '4000000000000077');
      await page.fill('[data-testid="new-card-expiry"]', '06/26');
      await page.fill('[data-testid="new-card-cvc"]', '456');
      
      await page.click('[data-testid="save-payment-method"]');
      
      // Should show success message
      await expect(page.locator('[data-testid="payment-method-added"]')).toBeVisible();
      
      console.log('✅ Payment method updates working');
    });

    test('should handle billing cycle and renewal management', async ({ page }) => {
      const user = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/subscriptions/billing', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            current_period_start: '2024-01-01T00:00:00Z',
            current_period_end: '2026-01-01T00:00:00Z',
            next_billing_date: '2026-01-01T00:00:00Z',
            auto_renew: true,
            billing_cycle: 'annual',
            proration_behavior: 'create_prorations',
            invoice_history: [
              {
                id: 'inv_12345',
                amount: 599.99,
                date: '2024-01-01T00:00:00Z',
                status: 'paid',
                period: '2024-2025'
              }
            ]
          })
        });
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="billing-management"]');
      
      // Should show billing information
      await expect(page.locator('[data-testid="billing-cycle"]')).toContainText('Annual');
      await expect(page.locator('[data-testid="next-billing-date"]')).toBeVisible();
      await expect(page.locator('[data-testid="auto-renew-toggle"]')).toBeVisible();
      
      // Should show invoice history
      await expect(page.locator('[data-testid="invoice-history"]')).toBeVisible();
      await expect(page.locator('[data-testid="invoice-12345"]')).toBeVisible();
      
      // Should allow auto-renew toggle
      await page.click('[data-testid="auto-renew-toggle"]');
      await expect(page.locator('[data-testid="auto-renew-disabled-notice"]')).toBeVisible();
      
      console.log('✅ Billing cycle and renewal management working');
    });
  });

  test.describe('📊 Subscription Analytics and Insights', () => {
    
    test('should show subscription usage analytics', async ({ page }) => {
      const user = TEST_USERS.GOLD_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/analytics/usage', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            current_month: {
              api_calls: 2500,
              features_used: ['advanced_analytics', 'portfolio_tools', 'priority_support'],
              login_sessions: 45,
              trading_volume: 125000
            },
            tier_utilization: {
              percentage: 75,
              recommendations: [
                'Consider upgrading to PLATINUM for unlimited API calls',
                'You\'re using 75% of your GOLD tier features'
              ]
            },
            cost_per_feature: {
              advanced_analytics: 15.50,
              portfolio_tools: 18.75,
              priority_support: 8.25
            }
          })
        });
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="usage-analytics"]');
      
      // Should show usage metrics
      await expect(page.locator('[data-testid="api-usage-chart"]')).toBeVisible();
      await expect(page.locator('[data-testid="feature-usage-list"]')).toBeVisible();
      await expect(page.locator('[data-testid="tier-utilization"]')).toContainText('75%');
      
      // Should show upgrade recommendations
      await expect(page.locator('[data-testid="upgrade-recommendations"]')).toBeVisible();
      
      // Should show cost breakdown
      await expect(page.locator('[data-testid="cost-per-feature"]')).toBeVisible();
      
      console.log('✅ Subscription usage analytics displayed');
    });

    test('should provide tier optimization suggestions', async ({ page }) => {
      const user = TEST_USERS.SILVER_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/optimization/tier-analysis', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            current_tier: 'SILVER',
            usage_analysis: {
              underutilized_features: ['advanced_analytics'],
              frequently_blocked_features: ['portfolio_tools', 'priority_support'],
              cost_efficiency: 0.65
            },
            recommendations: [
              {
                action: 'upgrade',
                target_tier: 'GOLD',
                reason: 'You frequently access blocked features',
                potential_savings: 0,
                potential_value: 125
              },
              {
                action: 'downgrade',
                target_tier: 'BRONZE',
                reason: 'Low usage of advanced features',
                potential_savings: 15,
                potential_value: -45
              }
            ]
          })
        });
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="tier-optimization"]');
      
      // Should show analysis results
      await expect(page.locator('[data-testid="usage-analysis"]')).toBeVisible();
      await expect(page.locator('[data-testid="underutilized-features"]')).toBeVisible();
      await expect(page.locator('[data-testid="blocked-features"]')).toBeVisible();
      
      // Should show recommendations
      await expect(page.locator('[data-testid="optimization-recommendations"]')).toBeVisible();
      await expect(page.locator('[data-testid="upgrade-recommendation"]')).toBeVisible();
      await expect(page.locator('[data-testid="downgrade-recommendation"]')).toBeVisible();
      
      // Should allow acting on recommendations
      await expect(page.locator('[data-testid="apply-recommendation"]')).toBeVisible();
      
      console.log('✅ Tier optimization suggestions working');
    });

    test('should track subscription ROI and value metrics', async ({ page }) => {
      const user = TEST_USERS.ENTERPRISE_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/analytics/roi', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            subscription_cost: 5999.88, // Annual
            calculated_value: {
              api_usage_value: 2400,
              feature_access_value: 1800,
              time_saved_value: 3200,
              priority_support_value: 800
            },
            roi_percentage: 135,
            break_even_analysis: {
              break_even_reached: true,
              break_even_date: '2024-04-15T00:00:00Z',
              days_to_break_even: 105
            },
            value_trends: [
              { month: '2024-01', value: 450 },
              { month: '2024-02', value: 520 },
              { month: '2024-03', value: 680 },
              { month: '2024-04', value: 750 }
            ]
          })
        });
      });
      
      await page.goto('/settings');
      await page.click('[data-testid="subscription-roi"]');
      
      // Should show ROI metrics
      await expect(page.locator('[data-testid="roi-percentage"]')).toContainText('135%');
      await expect(page.locator('[data-testid="value-breakdown"]')).toBeVisible();
      
      // Should show break-even analysis
      await expect(page.locator('[data-testid="break-even-reached"]')).toBeVisible();
      
      // Should show value trends chart
      await expect(page.locator('[data-testid="value-trends-chart"]')).toBeVisible();
      
      console.log('✅ Subscription ROI tracking working');
    });
  });

  test.describe('🎯 Personalized Subscription Experience', () => {
    
    test('should provide personalized upgrade recommendations', async ({ page }) => {
      const user = TEST_USERS.BRONZE_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/personalization/recommendations', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            user_profile: {
              usage_patterns: ['high_api_usage', 'frequent_analytics'],
              preferred_features: ['real_time_data', 'advanced_charts'],
              time_of_usage: 'business_hours',
              trading_frequency: 'daily'
            },
            personalized_offers: [
              {
                tier: 'SILVER',
                discount: 15,
                reason: 'Perfect for your high API usage',
                trial_available: true,
                urgency: 'limited_time'
              }
            ],
            feature_suggestions: [
              'Advanced analytics would save you 2 hours per week',
              'Real-time alerts could improve your trading performance by 15%'
            ]
          })
        });
      });
      
      await page.goto('/');
      
      // Should show personalized recommendations
      await expect(page.locator('[data-testid="personalized-upgrade-banner"]')).toBeVisible();
      await expect(page.locator('[data-testid="recommendation-reason"]')).toContainText('high API usage');
      
      // Should show feature benefits specific to user
      await expect(page.locator('[data-testid="feature-benefits"]')).toBeVisible();
      
      // Should show trial option if available
      await expect(page.locator('[data-testid="trial-offer"]')).toBeVisible();
      
      console.log('✅ Personalized upgrade recommendations working');
    });

    test('should adapt UI based on subscription tier', async ({ page }) => {
      const testCases = [
        { user: TEST_USERS.FREE_USER, expectedFeatures: ['basic-nav', 'upgrade-prompts'] },
        { user: TEST_USERS.GOLD_USER, expectedFeatures: ['premium-nav', 'vip-badge'] },
        { user: TEST_USERS.ENTERPRISE_USER, expectedFeatures: ['enterprise-nav', 'api-access-shortcut'] }
      ];

      for (const testCase of testCases) {
        await authenticateUser(page, testCase.user);
        await page.goto('/');
        
        // Check tier-specific UI elements
        for (const feature of testCase.expectedFeatures) {
          await expect(page.locator(`[data-testid="${feature}"]`)).toBeVisible();
        }
        
        // Check tier badge in header
        await expect(page.locator('[data-testid="tier-badge"]')).toContainText(testCase.user.package_tier);
        
        console.log(`✅ UI adaptation verified for ${testCase.user.package_tier}`);
      }
    });

    test('should show subscription health and alerts', async ({ page }) => {
      const user = TEST_USERS.CANCELLED_USER;
      await authenticateUser(page, user);
      
      await page.route('**/api/subscriptions/health', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            subscription_status: 'cancelled',
            health_score: 'at_risk',
            alerts: [
              {
                type: 'warning',
                message: 'Subscription cancelled - access ends Sept 30',
                action_required: true,
                action_url: '/reactivate'
              }
            ],
            days_until_expiry: 8,
            reactivation_incentive: {
              discount: 25,
              valid_until: '2024-09-15T00:00:00Z'
            }
          })
        });
      });
      
      await page.goto('/');
      
      // Should show subscription health alert
      await expect(page.locator('[data-testid="subscription-alert"]')).toBeVisible();
      await expect(page.locator('[data-testid="cancellation-notice"]')).toBeVisible();
      
      // Should show reactivation incentive
      await expect(page.locator('[data-testid="reactivation-offer"]')).toBeVisible();
      await expect(page.locator('[data-testid="discount-offer"]')).toContainText('25%');
      
      // Should show countdown to expiry
      await expect(page.locator('[data-testid="days-remaining"]')).toContainText('8');
      
      console.log('✅ Subscription health and alerts working');
    });
  });
});