import { expect } from '@playwright/test';
import { metaMaskFixtures } from '@synthetixio/synpress-metamask/playwright';
import walletSetup from './wallet.setup';

// metaMaskFixtures returns a customized test object with the metamask fixtures installed
const test = metaMaskFixtures(walletSetup);

test.describe('Web3 Authentication Flow', () => {

    test('Should connect wallet and sign SIWE message successfully', async ({ context, metamask }) => {
        const dappPage = await context.newPage();
        await dappPage.goto('/');
        await dappPage.bringToFront();

        // 1. Click Connect Wallet (The trigger on your DApp)
        await dappPage.getByRole('button', { name: /Connect Wallet/i }).click();

        // 2. Click MetaMask option in the RainbowKit/Wagmi modal
        await dappPage.getByRole('button', { name: /MetaMask/i }).click();

        // 3. Instruct Synpress to approve the connection inside the real extension
        await metamask.connectToDapp();

        // 4. (Optional) Depending on the wagmi config, wait for Network Switch prompt
        // Wait slightly to observe if a network switch is requested (e.g., to BSC Testnet)
        try {
            await expect(dappPage.getByText('Switch Network')).toBeVisible({ timeout: 5000 });
            await metamask.approveSwitchNetwork();
        } catch (e) {
            // Network switch either wasn't required or already handled
        }

        // 5. Trigger the Sign-In (SIWE) flow from your UI
        // Ensure this matches the button text that appears after connecting
        await dappPage.getByRole('button', { name: /Sign Message/i }).click();

        // 6. Instruct Synpress to sign the message in the extension
        await metamask.confirmSignature();

        // 7. Assert successful login
        // Look for a UI element that denotes the user is authenticated
        await expect(dappPage.getByText('Dashboard')).toBeVisible();
        await expect(dappPage.getByText(/0x/)).toBeVisible(); // E.g., shortened address
    });

});
