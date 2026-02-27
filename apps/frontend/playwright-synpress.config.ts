import { defineConfig, devices } from '@playwright/test';
import dotenv from 'dotenv';
import path from 'node:path';

// Load environment variables
dotenv.config({ path: path.resolve(__dirname, '.env') });
dotenv.config({ path: path.resolve(__dirname, '.env.local') });

export default defineConfig({
    testDir: './__test__/e2e-synpress',
    // Synpress tests with real extensions cannot be run in highly parallel environments
    workers: 1,
    retries: process.env.CI ? 1 : 0,
    timeout: 120000, // 2 minutes due to extension interactions
    expect: {
        timeout: 15000,
    },
    reporter: [
        ['html', { outputFolder: 'playwright-synpress-report', open: 'never' }],
        ['list']
    ],
    use: {
        // URL to the frontend app
        baseURL: process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000',
        trace: 'on-first-retry',
        screenshot: 'only-on-failure',
        video: 'retain-on-failure',
    },
    projects: [
        {
            name: 'synpress-metamask',
            use: {
                ...devices['Desktop Chrome'],
            },
        },
    ],
});
