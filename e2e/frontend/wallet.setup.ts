import { defineWalletSetup } from '@synthetixio/synpress';
import { MetaMask, getExtensionId } from '@synthetixio/synpress-metamask/playwright';

import 'dotenv/config'; // Ensure dotenv is used in synpress CLI too

const PASSWORD = 'Password123!';
// Standard valid 12-word seed phrase often used in anvil/testnets
const SEED_PHRASE = 'test test test test test test test test test test test junk';

// The defineWalletSetup function from synpress-cache takes a password and a setup callback
export default defineWalletSetup(PASSWORD, async (context, walletPage) => {
    // Instantiate MetaMask driver
    const extensionId = await getExtensionId(context, 'MetaMask');
    const metamask = new MetaMask(context, walletPage, PASSWORD, extensionId);

    // Setup network and import wallet
    await metamask.importWallet(SEED_PHRASE);

    // Explicitly click "Open wallet" since sometimes Synpress stops at the final "Your wallet is ready!" page
    try {
        await walletPage.getByRole('button', { name: /Open wallet/i }).click({ timeout: 5000 });
        await walletPage.waitForLoadState();
    } catch (e) {
        // Ignore if the button doesn't exist (e.g., if synpress already handled it)
    }

    // Wait until the main wallet page is loaded by checking for standard elements
    try {
        await walletPage.getByRole('button', { name: /Got it/i }).click({ timeout: 5000 });
    } catch (e) { }
    // Wagmi will trigger the "Add Network" prompt automatically during the test.
});
