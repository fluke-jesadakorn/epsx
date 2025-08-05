import { Page } from '@playwright/test';

export class APIMocks {
  constructor(private page: Page) {}

  async mockSuccessfulAuth(email: string = 'admin@epsx.com', role: string = 'admin') {
    // Mock the backend login API
    await this.page.route('**/api/v1/auth/login', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user_id: '1',
          email: email,
          role: role,
          permissions: ['admin:read', 'admin:write'],
          subscription_tier: 'premium',
          access_token: 'mock-token',
          expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString()
        })
      });
    });

    // Mock NextAuth session endpoint
    await this.page.route('**/api/auth/session', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user: {
            id: '1',
            email: email,
            role: role,
            permissions: ['admin:read', 'admin:write'],
            subscription_tier: 'premium'
          },
          expires: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
          accessToken: 'mock-token',
          expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString()
        })
      });
    });

    // Mock NextAuth callback endpoint
    await this.page.route('**/api/auth/callback/credentials', async route => {
      // Instead of using redirect status, mock successful response
      await route.fulfill({
        status: 200,
        contentType: 'text/html',
        headers: {
          'Set-Cookie': 'next-auth.session-token=mock-session-token; Path=/; HttpOnly; SameSite=lax'
        },
        body: `
          <!DOCTYPE html>
          <html>
          <head><title>Redirecting...</title></head>
          <body>
            <script>
              window.location.href = '${new URL(route.request().url()).origin}/admin';
            </script>
          </body>
          </html>
        `
      });
    });

    // Mock NextAuth signin endpoint
    await this.page.route('**/api/auth/signin/credentials', async route => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            url: `${new URL(route.request().url()).origin}/`
          })
        });
      } else {
        await route.continue();
      }
    });

    // Mock CSRF token endpoint
    await this.page.route('**/api/auth/csrf', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          csrfToken: 'mock-csrf-token'
        })
      });
    });
  }

  async mockFailedAuth(error: string = 'Invalid credentials') {
    // Mock failed backend login
    await this.page.route('**/api/v1/auth/login', async route => {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({
          error: error
        })
      });
    });

    // Mock NextAuth endpoints for failed auth
    await this.page.route('**/api/auth/callback/credentials', async route => {
      // Mock failed auth with error response instead of redirect
      await route.fulfill({
        status: 200,
        contentType: 'text/html',
        body: `
          <!DOCTYPE html>
          <html>
          <head><title>Authentication Error</title></head>
          <body>
            <script>
              window.location.href = '${new URL(route.request().url()).origin}/login?error=CredentialsSignin';
            </script>
          </body>
          </html>
        `
      });
    });

    await this.page.route('**/api/auth/session', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({})
      });
    });
  }

  async mockNetworkError() {
    await this.page.route('**/api/v1/auth/login', route => route.abort());
    await this.page.route('**/api/auth/**', route => route.abort());
  }

  async mockLoggedOutSession() {
    await this.page.route('**/api/auth/session', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({})
      });
    });
  }
}