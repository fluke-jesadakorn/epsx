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

var epsxSessionLockName = 'epsx.auth.session-mutation.v1';
var epsxSessionChannelName = 'epsx.auth.session.v1';
var epsxRefreshPromise = null;
var epsxRecoverPromise = null;
var epsxSessionChannel = null;
if (typeof BroadcastChannel === 'function') {
  try {
    epsxSessionChannel = new BroadcastChannel(epsxSessionChannelName);
  } catch (_) {}
}

function epsxSessionReason(reason) {
  return reason === 'logout' || reason === 'refresh_rejected' || reason === 'refresh_unknown'
    ? reason
    : 'refresh_unknown';
}

function epsxDispatchSessionEvent(event) {
  try {
    document.dispatchEvent(new CustomEvent('epsx:auth:session', { detail: event }));
  } catch (_) {}
}

function epsxPublishSessionEvent(type, reason) {
  var event = { version: 1, type: type };
  if (type === 'session-ended') event.reason = epsxSessionReason(reason);
  if (epsxSessionChannel) {
    try { epsxSessionChannel.postMessage(event); } catch (_) {}
  }
  epsxDispatchSessionEvent(event);
}

function epsxEndLocalSession(reason, target) {
  epsxPublishSessionEvent('session-ended', reason);
  window.location.assign(target);
}

if (epsxSessionChannel) {
  epsxSessionChannel.onmessage = function(message) {
    var event = message && message.data;
    if (!event || event.version !== 1 || event.type !== 'session-ended') return;
    epsxDispatchSessionEvent({
      version: 1,
      type: 'session-ended',
      reason: epsxSessionReason(event.reason)
    });
    window.location.assign('/');
  };
}

function epsxWithSessionMutation(operation, requireCrossTabLock) {
  if (navigator.locks && typeof navigator.locks.request === 'function') {
    return navigator.locks.request(epsxSessionLockName, { mode: 'exclusive' }, operation);
  }
  if (requireCrossTabLock) {
    return Promise.reject(new Error('Session refresh requires cross-tab coordination'));
  }
  return operation();
}

function epsxSafeSessionTarget(raw) {
  if (!raw || raw.charAt(0) !== '/' || raw.indexOf('//') === 0 || raw.indexOf('\\') !== -1) {
    return '/';
  }
  try {
    var target = new URL(raw, window.location.origin);
    if (target.origin !== window.location.origin || target.username || target.password) return '/';
    return target.pathname + target.search + target.hash;
  } catch (_) {
    return '/';
  }
}

async function epsxBestEffortLocalEnd(reason) {
  try {
    var response = await fetch('/api/v1/auth/logout', {
      method: 'POST',
      credentials: 'same-origin'
    });
    if (response.headers.get('x-epsx-session-state') !== 'cleared') return false;
    epsxEndLocalSession(reason, '/');
    return true;
  } catch (_) {
    return false;
  }
}

async function epsxRejectRefreshResponse(response) {
  try {
    await epsxReadJson(response, 'Session refresh failed');
  } catch (error) {
    throw error;
  }
  throw new Error('Session refresh failed: rotation was not attested');
}

async function epsxRefreshOnce() {
  var response;
  try {
    response = await fetch('/api/v1/auth/refresh', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: '{}'
    });
  } catch (error) {
    await epsxBestEffortLocalEnd('refresh_unknown');
    throw error;
  }

  var state = response.headers.get('x-epsx-session-state');
  if (state === 'preserved') {
    return epsxRejectRefreshResponse(response);
  }
  if (state === 'cleared') {
    epsxEndLocalSession(response.status === 401 ? 'refresh_rejected' : 'refresh_unknown', '/');
    return epsxRejectRefreshResponse(response);
  }
  if (state !== 'rotated') {
    await epsxBestEffortLocalEnd('refresh_unknown');
    return epsxRejectRefreshResponse(response);
  }

  var session = await epsxReadJson(response, 'Session refresh failed');
  epsxPublishSessionEvent('session-refreshed');
  return session;
}

function epsxRefreshSession() {
  if (epsxRefreshPromise) return epsxRefreshPromise;
  epsxRefreshPromise = epsxWithSessionMutation(epsxRefreshOnce, true).finally(function() {
    epsxRefreshPromise = null;
  });
  return epsxRefreshPromise;
}

function epsxRecoverSession() {
  if (epsxRecoverPromise) return epsxRecoverPromise;
  epsxRecoverPromise = epsxRefreshSession().then(function(session) {
    window.location.reload();
    return session;
  });
  return epsxRecoverPromise;
}

function epsxSiweLogin(message, signature, address, nonce, chainId) {
  return epsxWithSessionMutation(function() {
    return epsxPostJson(
      '/api/v1/auth/siwe',
      { message: message, signature: signature, address: address, nonce: nonce, chain_id: String(chainId || '') },
      'Verification failed'
    );
  }, false);
}

function epsxLogoutSession(target) {
  var safeTarget = epsxSafeSessionTarget(target || '/');
  return epsxWithSessionMutation(async function() {
    var response = await fetch('/api/v1/auth/logout', {
      method: 'POST',
      credentials: 'same-origin'
    });
    if (response.headers.get('x-epsx-session-state') !== 'cleared') {
      throw new Error('Local session clearing was not confirmed');
    }
    epsxEndLocalSession('logout', safeTarget);
  }, false);
}

window.epsxAuth = {
  challenge: (address) => epsxPostJson('/api/v1/auth/challenge', { address: address }, 'Challenge failed'),
  siweLogin: epsxSiweLogin,
  refresh: epsxRefreshSession,
  recover: epsxRecoverSession,
  me: async () => {
    var response = await fetch('/api/v1/auth/me', { credentials: 'same-origin' });
    return epsxReadJson(response, 'Session lookup failed');
  },
  logout: epsxLogoutSession
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

document.addEventListener('click', function(event) {
  var target = event.target && typeof event.target.closest === 'function'
    ? event.target.closest('[data-epsx-logout]')
    : null;
  if (!target) return;
  event.preventDefault();
  window.epsxAuth.logout(target.getAttribute('data-epsx-logout-target') || '/').catch(function(error) {
    window.epsxWalletStatus({
      status: 'error',
      kind: 'logout_unconfirmed',
      message: error && error.message ? error.message : 'Logout could not be confirmed.'
    });
  });
});
"#
}

/// Fixed SSR bootstrap for pages whose BFF observed a missing/rejected access
/// credential alongside its own HttpOnly refresh cookie. No credential or
/// request-derived value is interpolated into this script.
pub fn browser_session_recovery_script() -> &'static str {
    "window.epsxAuth.recover().catch(function(){try{document.dispatchEvent(new CustomEvent('epsx:auth:recovery',{detail:{version:1,state:'failed'}}));}catch(_){}});"
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
    fn refresh_siwe_and_logout_share_one_cross_tab_mutation_lock() {
        let script = browser_auth_script();
        assert!(script.contains("navigator.locks.request(epsxSessionLockName"));
        assert!(script.contains("epsxWithSessionMutation(epsxRefreshOnce, true)"));
        assert!(
            script.contains("function epsxSiweLogin(message, signature, address, nonce, chainId)")
        );
        assert!(script.contains("siweLogin: epsxSiweLogin"));
        assert!(script.contains("function epsxLogoutSession(target)"));
        assert!(script.contains("}, false);"));
    }

    #[test]
    fn refresh_has_one_in_page_flight_and_no_automatic_retry() {
        let script = browser_auth_script();
        assert!(script.contains("if (epsxRefreshPromise) return epsxRefreshPromise"));
        assert_eq!(script.matches("fetch('/api/v1/auth/refresh'").count(), 1);
        assert!(script.contains("Session refresh requires cross-tab coordination"));
    }

    #[test]
    fn recovery_is_one_shot_and_reloads_only_after_verified_rotation() {
        let script = browser_auth_script();
        assert!(script.contains("if (epsxRecoverPromise) return epsxRecoverPromise"));
        assert!(script.contains("epsxRecoverPromise = epsxRefreshSession().then(function(session)"));
        assert!(script.contains("window.location.reload();"));
        assert!(script.contains("recover: epsxRecoverSession"));
        assert!(script.contains("if (state === 'cleared')"));
        assert!(script.contains("await epsxBestEffortLocalEnd('refresh_unknown')"));
        assert!(script.contains("return epsxRejectRefreshResponse(response)"));
        assert!(!browser_session_recovery_script().contains("token"));
        assert_eq!(
            browser_session_recovery_script(),
            "window.epsxAuth.recover().catch(function(){try{document.dispatchEvent(new CustomEvent('epsx:auth:recovery',{detail:{version:1,state:'failed'}}));}catch(_){}});"
        );
    }

    #[test]
    fn recovery_bootstrap_emits_only_fixed_token_free_failure_state() {
        let script = browser_session_recovery_script();
        assert_eq!(script.matches("window.epsxAuth.recover()").count(), 1);
        assert!(script.contains("'epsx:auth:recovery'"));
        assert!(script.contains("detail:{version:1,state:'failed'}"));
        for forbidden in [
            "function(error)",
            "error.message",
            "String(",
            "JSON.stringify",
            "token",
            "wallet",
            "permission",
            "plan",
            "fetch(",
        ] {
            assert!(
                !script.contains(forbidden),
                "recovery bootstrap must not disclose or interpolate {forbidden:?}"
            );
        }
    }

    #[test]
    fn session_broadcasts_are_closed_and_token_free() {
        let script = browser_auth_script();
        assert!(script.contains("{ version: 1, type: type }"));
        assert!(script.contains("type === 'session-ended'"));
        assert!(script.contains("epsxPublishSessionEvent('session-refreshed')"));
        assert!(!script.contains("localStorage"));
        assert!(!script.contains("sessionStorage"));
    }

    #[test]
    fn logout_redirects_only_after_local_clear_confirmation() {
        let script = browser_auth_script();
        let confirmed = script
            .find("!== 'cleared'")
            .expect("logout must inspect the BFF state marker");
        let redirect = script
            .find("epsxEndLocalSession('logout', safeTarget)")
            .expect("confirmed logout must redirect");
        assert!(redirect > confirmed);
        assert!(script.contains("epsxSafeSessionTarget(target || '/')"));
    }

    #[test]
    fn channel_failures_degrade_to_same_tab_events_and_logout_is_delegated() {
        let script = browser_auth_script();
        assert!(script.contains("try { epsxSessionChannel.postMessage(event); } catch (_) {}"));
        assert!(script.contains("epsxSessionChannel = new BroadcastChannel"));
        assert!(script.contains("event.target.closest('[data-epsx-logout]')"));
        assert!(script.contains("epsxEndLocalSession(reason, '/')"));
    }
}
