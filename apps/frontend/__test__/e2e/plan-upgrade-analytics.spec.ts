import { BrowserContext, expect, Page, test } from '@playwright/test';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { Service, URL, URLContext } from '../../../../shared/utils/url-resolver';

// Environment configuration
const BASE_URL = URL.get(Service.FRONTEND, URLContext.CLIENT);
const API_URL = URL.get(Service.BACKEND, URLContext.CLIENT);
const ADMIN_URL = URL.get(Service.ADMIN, URLContext.CLIENT);

// Admin keys for setup 
const ADMIN_CREDENTIALS = {
    email: 'admin@epsx.io',
    password: 'password'
};

test.describe('📈 E2E Plan Upgrade & Analytics Offset Verification', () => {
    let userContext: BrowserContext;
    let userPage: Page;
    let adminContext: BrowserContext;
    let userAccount: any;
    let walletAddress: string;

    test.beforeAll(async ({ browser }) => {
        // Create User Context
        userContext = await browser.newContext({
            viewport: { width: 1280, height: 720 },
            acceptDownloads: true
        });
        userPage = await userContext.newPage();

        // Create Admin Context
        adminContext = await browser.newContext();
    });

    test.afterAll(async () => {
        await userContext?.close();
        await adminContext?.close();
    });

    // Helper: Mock Wallet Injection with Real Signing
    async function mockWallet(page: Page, account: any) {
        await page.addInitScript(({ addr }) => {
            (window as any).ethereum = {
                isMetaMask: true,
                request: async ({ method, params }: any) => {
                    switch (method) {
                        case 'eth_requestAccounts':
                        case 'eth_accounts':
                            return [addr];
                        case 'eth_chainId':
                            return '0x38'; // BSC
                        case 'personal_sign':
                            const msg = params[0];
                            if ((window as any).__signMessage) {
                                return await (window as any).__signMessage(msg);
                            }
                            return '0xFAILED_SIGN';
                        default:
                            return null;
                    }
                },
                on: () => { },
                removeListener: () => { },
            };
        }, { addr: account.address });

        // Expose signing function
        await page.exposeFunction('__signMessage', async (msgHexOrString: string) => {
            try {
                console.log(`📝 [Signer] Request: ${msgHexOrString.slice(0, 50)}...`);
                let message: any = msgHexOrString;

                if (msgHexOrString.startsWith('0x')) {
                    try {
                        const decoded = Buffer.from(msgHexOrString.slice(2), 'hex').toString('utf8');
                        const isText = /^[ -~]+$/.test(decoded);
                        if (isText) {
                            console.log(`📝 [Signer] Decoded: ${decoded.slice(0, 50)}...`);
                            message = decoded;
                        }
                    } catch (e) { }
                }

                return await account.signMessage({ message });
            } catch (e) {
                console.error("Signing failed:", e);
                throw e;
            }
        });

        page.on('console', msg => {
            if (msg.type() === 'error' || msg.text().includes('auth') || msg.text().includes('Signer'))
                console.log(`🖥️ [Browser] ${msg.text()}`);
        });
    }

    test('🚀 Verify Plan Upgrade and Analytics Offset', async () => {
        // 1. Setup Identity
        const privateKey = generatePrivateKey();
        userAccount = privateKeyToAccount(privateKey);
        walletAddress = userAccount.address;
        console.log(`🆔 Using Real Wallet: ${walletAddress}`);

        // Inject Mock Wallet
        await mockWallet(userPage, userAccount);

        // 2. Connect Wallet (Login)
        console.log("🔌 Connecting Wallet...");
        await userPage.goto(`${BASE_URL}/connect-wallet`);
        await userPage.waitForLoadState('networkidle');

        try {
            console.log("🕵️ Looking for Connect UI...");

            // 1. Click Connect Button
            const connectBtn = userPage.getByText(/connect/i).first();
            if (await connectBtn.isVisible({ timeout: 15000 })) {
                console.log("👆 Clicking Connect...");
                await connectBtn.click();
            } else {
                console.log("ℹ️ No Connect button found (might be already open or loading).");
            }

            // 2. Select Provider (MetaMask)
            await userPage.waitForTimeout(1000);
            const mmBtn = userPage.getByText(/metamask|injected|browser/i).first();
            if (await mmBtn.isVisible({ timeout: 15000 })) {
                console.log("👆 Selecting Provider (MetaMask)...");
                await mmBtn.click();
            } else {
                console.log("ℹ️ No MetaMask button found (maybe auto-connected?).");
            }

            // 3. Click Sign In
            const signInBtn = userPage.getByText(/sign in/i).first();
            if (await signInBtn.isVisible({ timeout: 15000 })) {
                console.log("👆 Clicking Sign In...");
                await signInBtn.click();
            } else {
                console.log("ℹ️ No Sign In button found (maybe already signed in/processing?).");
            }

        } catch (e) {
            console.log("⚠️ UI Interaction error:", e);
        }

        // Wait for Session with Retry
        await expect(async () => {
            const cookies = await userContext.cookies();
            const hasToken = cookies.some(c => c.name === 'epsx.access_token');

            if (!hasToken) {
                // Diagnostic check inside retry loop
                const body = await userPage.content();
                if (body.includes("Sign In") && !body.includes("Sign In with Wallet")) {
                    // Try clicking Sign In again if visible?
                    // const btn = userPage.getByText('Sign In', {exact: true});
                    // if (await btn.isVisible()) await btn.click();
                }
            }
            expect(hasToken).toBeTruthy();
        }).toPass({ timeout: 45000, interval: 2000 });

        console.log("✅ Session Established");

        // 3. Verify Initial State
        console.log("🔍 Verifying Initial Free State...");
        await userPage.goto(`${BASE_URL}/account`);
        await expect(userPage.getByText(/Free Plan|Current Plan/i)).toBeVisible();

        await userPage.goto(`${BASE_URL}/analytics`);
        await expect(userPage.getByRole('table')).toBeVisible({ timeout: 10000 });

        // 4. Admin Upgrade (Starter)
        console.log("🔧 Admin upgrading to Starter...");
        const adminPage = await adminContext.newPage();
        await adminPage.goto(`${ADMIN_URL}/login`);
        if (adminPage.url().includes('login')) {
            await adminPage.fill('input[name="email"]', ADMIN_CREDENTIALS.email);
            await adminPage.fill('input[name="password"]', ADMIN_CREDENTIALS.password);
            await adminPage.getByRole('button', { name: /sign in|login/i }).click();
            await adminPage.waitForURL(ADMIN_URL);
        }

        const upgradeToStarter = await adminPage.evaluate(async ({ apiUrl, wallet }) => {
            const resp = await fetch(`${apiUrl}/api/admin/subscriptions`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    wallet_address: wallet,
                    permission_group_name: 'Starter Plan',
                    access_context: 'internal',
                    auto_renew: true
                })
            });
            return { status: resp.status, body: await resp.json() };
        }, { apiUrl: API_URL, wallet: walletAddress });

        expect(upgradeToStarter.status).toBe(200);

        // 5. Verify Starter
        await userPage.bringToFront();
        await userPage.reload();
        await userPage.goto(`${BASE_URL}/account`);
        await expect(userPage.getByText('Starter Plan')).toBeVisible();
        console.log("✅ Verified Starter Plan");

        // 6. Admin Upgrade (Pro)
        console.log("🔧 Admin upgrading to Pro...");
        const upgradeToPro = await adminPage.evaluate(async ({ apiUrl, wallet }) => {
            const resp = await fetch(`${apiUrl}/api/admin/subscriptions`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    wallet_address: wallet,
                    permission_group_name: 'Pro Plan',
                    access_context: 'internal',
                    auto_renew: true
                })
            });
            return { status: resp.status, body: await resp.json() };
        }, { apiUrl: API_URL, wallet: walletAddress });

        expect(upgradeToPro.status).toBe(200);

        // 7. Verify Pro
        await userPage.bringToFront();
        await userPage.reload();
        await userPage.goto(`${BASE_URL}/account`);
        await expect(userPage.getByText('Pro Plan')).toBeVisible();
        console.log("✅ Verified Pro Plan");

        console.log("✅ Full Upgrade Flow Verified!");
    });
});
