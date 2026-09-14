// WalletConnect transport adapter. Checkout state and transactions belong to Rust.
import { EthereumProvider } from '@walletconnect/ethereum-provider';
let provider;
let epoch = 0;
let state = { address: '', chain: '', uri: '' };
const changed = () => window.dispatchEvent(new Event('epsx-wallet-changed'));
const reset = () => { state = { address: '', chain: '', uri: '' }; changed(); };
window.EPSXWalletConnect = {
  state: () => ({ ...state }),
  async connect(config) {
    const current = ++epoch;
    if (!provider) {
      provider = await EthereumProvider.init({
        projectId: config.projectId,
        metadata: { name: 'EPSX Pay', description: 'EPSX crypto checkout', url: location.origin, icons: [location.origin + '/brand-icon.svg'] },
        optionalChains: [config.chainId],
        optionalMethods: ['eth_sendTransaction', 'wallet_switchEthereumChain', 'wallet_addEthereumChain'],
        rpcMap: { [config.chainId]: config.rpcUrl },
        showQrModal: false,
        disableProviderPing: true,
        telemetryEnabled: false
      });
      const source = provider;
      provider.on('display_uri', uri => { if (source === provider) { state.uri = uri; changed(); } });
      provider.on('accountsChanged', accounts => { if (source === provider) { state.address = accounts[0] || ''; changed(); } });
      provider.on('chainChanged', chain => { if (source === provider) { state.chain = String(chain); changed(); } });
      provider.on('disconnect', () => { if (source === provider) reset(); });
    }
    const active = provider;
    if (current !== epoch) throw new Error('Connection cancelled');
    await active.connect({ optionalChains: [config.chainId] });
    if (current !== epoch) { if (active.session) await active.disconnect(); throw new Error('Connection cancelled'); }
    state = { address: active.accounts[0] || '', chain: String(active.chainId), uri: '' };
    changed();
    return provider;
  },
  async disconnect() {
    ++epoch;
    const old = provider;
    provider = undefined;
    reset();
    if (old?.session) await old.disconnect();
  }
};
