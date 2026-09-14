try {
    const ethereum = window.ethereum;
    const provider = (Array.isArray(ethereum?.providers)
        ? ethereum.providers.find(candidate => candidate.isMetaMask)
        : null) || ethereum;
    if (provider?.request) {
        await provider.request({
            method: 'wallet_revokePermissions',
            params: [{ eth_accounts: {} }],
        });
    }
    dioxus.send(true);
} catch (error) {
    // An unavailable/older extension must not prevent server-side logout.
    console.warn('Wallet permission revocation failed', error);
    dioxus.send(false);
}
