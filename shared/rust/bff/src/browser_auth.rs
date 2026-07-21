//! SSR-safe browser bridge for the wallet challenge/sign/verify flow.
//!
//! The bridge only talks to same-origin BFF endpoints. Session tokens never
//! enter browser-readable JSON, storage, or JavaScript state.

/// Shared wallet authentication bridge used by the frontend today and the
/// admin BFF once it adopts the same canonical session endpoints.
pub fn browser_auth_script() -> &'static str {
    r#"
window.epsx = window.epsx || {};

window.epsxWallet = {
  isAvailable: () => typeof window.ethereum !== 'undefined',
  request: (method, params) => {
    if (!window.ethereum) return Promise.reject(new Error('No wallet'));
    return window.ethereum.request({ method: method, params: params || [] });
  },
  personalSign: (message, address) => {
    if (!window.ethereum) return Promise.reject(new Error('No wallet'));
    return window.ethereum.request({ method: 'personal_sign', params: [message, address] });
  },
  address: () => window.ethereum && window.ethereum.selectedAddress ? window.ethereum.selectedAddress : null,
  chainId: () => window.ethereum && window.ethereum.chainId ? window.ethereum.chainId : null,
  onAccountsChanged: (cb) => { if (window.ethereum) window.ethereum.on('accountsChanged', cb); },
  onChainChanged: (cb) => { if (window.ethereum) window.ethereum.on('chainChanged', cb); },
  addToken: (token) => {
    if (!window.ethereum) return Promise.reject(new Error('No wallet'));
    return window.ethereum.request({
      method: 'wallet_watchAsset',
      params: { type: 'ERC20', options: token }
    });
  }
};

function epsxSafeReturnUrl() {
  var raw = new URLSearchParams(window.location.search).get('return_url');
  if (!raw || raw.charAt(0) !== '/' || raw.indexOf('//') === 0 || raw.indexOf('\\') !== -1) return '/';
  try {
    var target = new URL(raw, window.location.origin);
    if (target.origin !== window.location.origin || target.username || target.password) return '/';
    if (target.pathname === '/auth') return '/';
    return target.pathname + target.search + target.hash;
  } catch (_) {
    return '/';
  }
}

async function epsxReadJson(response, label) {
  var text = await response.text();
  var data = null;
  if (text) {
    try { data = JSON.parse(text); } catch (_) { data = null; }
  }
  if (!response.ok || !data || data.success === false || data.authenticated === false) {
    var message = data && (data.message || data.error);
    throw new Error(label + ': ' + (message || ('HTTP ' + response.status)));
  }
  return data;
}

async function epsxPostJson(path, payload, label) {
  var response = await fetch(path, {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload || {})
  });
  return epsxReadJson(response, label);
}

window.epsxAuth = {
  challenge: (address) => epsxPostJson('/api/v1/auth/challenge', { address: address }, 'Challenge failed'),
  siweLogin: (message, signature, address, nonce, chainId) => epsxPostJson(
    '/api/v1/auth/siwe',
    { message: message, signature: signature, address: address, nonce: nonce, chain_id: String(chainId || '') },
    'Verification failed'
  ),
  refresh: () => epsxPostJson('/api/v1/auth/refresh', {}, 'Session refresh failed'),
  me: async () => {
    var response = await fetch('/api/v1/auth/me', { credentials: 'same-origin' });
    return epsxReadJson(response, 'Session lookup failed');
  },
  logout: async () => {
    try {
      await fetch('/api/v1/auth/logout', { method: 'POST', credentials: 'same-origin' });
    } finally {
      window.location.assign('/');
    }
  }
};

window.epsxWalletStatus = function(detail) {
  try {
    document.dispatchEvent(new CustomEvent('epsx:wallet:status', { detail: detail || {} }));
  } catch (_) {}
};

window.epsxSetWalletCookie = function(address, chainId, connectorId) {
  try {
    var payload = JSON.stringify({
      address: address,
      connector_id: connectorId || 'injected',
      chain_id: String(chainId)
    });
    document.cookie = 'epsx_wallet=' + encodeURIComponent(payload) + '; Path=/; Max-Age=86400; SameSite=Lax';
  } catch (_) {}
};

window.epsx.connectWallet = async function() {
  if (typeof window.ethereum === 'undefined') {
    window.epsxWalletStatus({ status: 'error', kind: 'no_wallet', message: 'No wallet detected. Install MetaMask or another BSC wallet.' });
    return;
  }

  try {
    var accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
    if (!accounts || !accounts[0]) throw new Error('No accounts returned by wallet');
    var address = accounts[0];
    var chainIdHex = await window.ethereum.request({ method: 'eth_chainId' });
    var chainId = String(parseInt(chainIdHex, 16));

    window.epsxWalletStatus({ status: 'challenge', address: address });
    var challenge = await window.epsxAuth.challenge(address);
    if (!challenge.message || !challenge.nonce) throw new Error('Challenge response is incomplete');

    window.epsxWalletStatus({ status: 'signing', address: address });
    var signature = await window.epsxWallet.personalSign(challenge.message, address);

    window.epsxWalletStatus({ status: 'verifying', address: address });
    var session = await window.epsxAuth.siweLogin(
      challenge.message,
      signature,
      address,
      challenge.nonce,
      chainId
    );
    if (!session.authenticated || !session.user) throw new Error('Verification did not establish a session');

    window.epsxSetWalletCookie(address, chainId, 'injected');
    window.epsxWalletStatus({ status: 'success', address: address });
    window.location.replace(epsxSafeReturnUrl());
  } catch (error) {
    var kind = 'error';
    var message = error && error.message ? error.message : String(error);
    var lower = message.toLowerCase();
    if (error && (error.code === 4001 || lower.indexOf('user rejected') !== -1 || lower.indexOf('user denied') !== -1)) {
      kind = 'rejected';
      message = 'Signature cancelled. Click Connect Wallet to try again.';
    } else if (lower.indexOf('no wallet') !== -1 || lower.indexOf('install') !== -1) {
      kind = 'no_wallet';
    } else if (lower.indexOf('chain') !== -1 || lower.indexOf('network') !== -1) {
      kind = 'wrong_network';
    }
    window.epsxWalletStatus({ status: 'error', kind: kind, message: message });
  }
};

document.addEventListener('DOMContentLoaded', function() {
  document.querySelectorAll('[data-connect-wallet]').forEach(function(element) {
    element.addEventListener('click', function(event) {
      event.preventDefault();
      window.epsx.connectWallet();
    });
  });
});
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_never_reads_or_stores_session_tokens() {
        let script = browser_auth_script();
        assert!(!script.contains("localStorage"));
        assert!(!script.contains("sessionStorage"));
        assert!(!script.contains("epsx_token"));
        assert!(!script.contains("access_token"));
        assert!(!script.contains("refresh_token"));
        assert!(script.contains("credentials: 'same-origin'"));
    }

    #[test]
    fn bridge_rejects_cross_origin_return_targets() {
        let script = browser_auth_script();
        assert!(script.contains("target.origin !== window.location.origin"));
        assert!(script.contains("target.pathname === '/auth'"));
        assert!(script.contains("raw.indexOf('//') === 0"));
        assert!(script.contains("raw.indexOf('\\\\') !== -1"));
    }

    #[test]
    fn logout_redirects_from_finally() {
        let script = browser_auth_script();
        assert!(script.contains("} finally {"));
        assert!(script.contains("window.location.assign('/')"));
    }
}
