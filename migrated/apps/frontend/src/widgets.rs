//! Global widgets injected into every page (chat bubble, auth modal, toaster).

/// Floating chat widget shown on every authenticated page. Hidden on /chat.
pub fn chat_widget(is_authed: bool, user_id: &str) -> String {
    if !is_authed {
        return String::new();
    }
    let js = format!(r##"<script>
(function() {{
  const widget = document.getElementById('chat-widget');
  if (!widget) return;
  const bubble = widget.querySelector('.chat-bubble-btn');
  const panel = widget.querySelector('.chat-panel');
  const closeBtn = widget.querySelector('.chat-panel-close');
  if (bubble) bubble.addEventListener('click', () => {{
    panel.classList.toggle('open');
    bubble.style.display = panel.classList.contains('open') ? 'none' : '';
  }});
  if (closeBtn) closeBtn.addEventListener('click', () => {{
    panel.classList.remove('open');
    bubble.style.display = '';
  }});
  // Refresh unread count every 30s
  const userId = '{user_id}';
  async function refreshUnread() {{
    try {{
      const res = await fetch('/api/v1/notifications?user_id=' + userId + '&limit=1');
      if (!res.ok) return;
      const data = await res.json();
      const badge = widget.querySelector('.chat-bubble-badge');
      const unread = (data.items || []).filter(i => !i.read_at).length;
      if (unread > 0) {{
        badge.textContent = unread;
        badge.style.display = '';
      }} else {{
        badge.style.display = 'none';
      }}
    }} catch (e) {{}}
  }}
  refreshUnread();
  setInterval(refreshUnread, 30000);
  window.epsxChatRefresh = refreshUnread;
}})();
</script>"##);
    format!(
        r##"<div id="chat-widget" style="position:fixed;bottom:1.5rem;right:1.5rem;z-index:100;">
  <button class="chat-bubble-btn" style="width:3.5rem;height:3.5rem;border-radius:9999px;background:var(--gradient-brand);color:white;border:none;cursor:pointer;box-shadow:var(--shadow-lg);display:flex;align-items:center;justify-content:center;position:relative;transition:transform 0.2s;" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">
    <i class="fa-solid fa-message" style="font-size:1.25rem;"></i>
    <span class="chat-bubble-badge" style="display:none;position:absolute;top:-0.25rem;right:-0.25rem;background:var(--epsx-red);color:white;font-size:0.6875rem;font-weight:700;min-width:1.25rem;height:1.25rem;border-radius:9999px;padding:0 0.375rem;display:flex;align-items:center;justify-content:center;">0</span>
  </button>
  <div class="chat-panel" style="position:absolute;bottom:0;right:0;width:22rem;height:32rem;background:var(--bg-solid);border:1px solid var(--border);border-radius:1rem;box-shadow:var(--shadow-2xl);display:none;flex-direction:column;overflow:hidden;">
    <div class="chat-panel-close" style="padding:1rem;background:var(--gradient-brand);color:white;display:flex;align-items:center;justify-content:space-between;cursor:pointer;">
      <div style="display:flex;align-items:center;gap:0.5rem;">
        <i class="fa-solid fa-headset"></i>
        <span style="font-weight:700;">Support</span>
      </div>
      <i class="fa-solid fa-xmark"></i>
    </div>
    <div style="flex:1;padding:1rem;overflow-y:auto;display:flex;flex-direction:column;gap:0.75rem;">
      <p style="color:var(--text-muted);font-size:0.875rem;text-align:center;margin:auto 0;">Hi! How can we help?</p>
    </div>
    <a href="/chat" style="margin:0.5rem 1rem 1rem;padding:0.625rem;border-radius:0.5rem;background:var(--gradient-warm);color:white;text-align:center;text-decoration:none;font-weight:600;font-size:0.875rem;">Open Full Chat</a>
  </div>
</div>
<style>
.chat-panel.open {{ display:flex !important; animation:scaleIn 0.2s ease; }}
</style>
{js}"##,
        js = js
    )
}

/// Returns the global auth modal (sign-in / sign-up). Triggered by
/// `epsx.openAuthModal()`.
pub fn auth_modal() -> String {
    r##"<div id="epsx-auth-modal" style="display:none;"></div>
<script>
window.epsxAuth = {
  open() {
    if (document.getElementById('auth-modal-content')) {
      document.getElementById('epsx-auth-modal').style.display = 'block';
      return;
    }
    const html = `
      <div id="auth-modal-backdrop" class="modal-backdrop" onclick="if(event.target===this) epsxAuth.close()" style="position:fixed;inset:0;background:rgba(0,0,0,0.6);backdrop-filter:blur(8px);z-index:200;display:flex;align-items:center;justify-content:center;padding:1rem;">
        <div id="auth-modal-content" class="modal" style="background:var(--bg-solid);border-radius:1rem;box-shadow:var(--shadow-2xl);max-width:28rem;width:100%;padding:2rem;position:relative;">
          <button onclick="epsxAuth.close()" class="nav-link" style="position:absolute;top:0.75rem;right:0.75rem;width:2rem;height:2rem;padding:0;justify-content:center;"><i class="fa-solid fa-xmark"></i></button>
          <div style="text-align:center;margin-bottom:1.5rem;">
            <div style="width:3.5rem;height:3.5rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;box-shadow:var(--shadow-orange);">
              <i class="fa-solid fa-arrow-right-to-bracket" style="color:white;font-size:1.125rem;"></i>
            </div>
            <h2 style="font-size:1.5rem;font-weight:800;margin-bottom:0.5rem;">Sign in to EPSX</h2>
            <p style="color:var(--text-muted);font-size:0.9375rem;">Connect your wallet to continue</p>
          </div>
          <button onclick="epsxAuth.connectWallet()" class="btn btn-gradient btn-block btn-lg" style="margin-bottom:0.75rem;">
            <i class="fa-solid fa-wallet"></i> Connect Wallet (SIWE on BSC)
          </button>
          <a href="/auth" class="btn btn-outline btn-block">More sign-in options</a>
          <p style="font-size:0.75rem;color:var(--text-subtle);text-align:center;margin-top:1.5rem;">
            By signing in, you agree to our <a href="/terms" class="footer-link">Terms</a> and <a href="/privacy" class="footer-link">Privacy</a>.
          </p>
        </div>
      </div>`;
    document.getElementById('epsx-auth-modal').innerHTML = html;
    document.getElementById('epsx-auth-modal').style.display = 'block';
  },
  close() {
    document.getElementById('epsx-auth-modal').style.display = 'none';
  },
  async connectWallet() {
    if (typeof window.ethereum === 'undefined') {
      epsx.toast('Install MetaMask or another BSC wallet', 'warning');
      return;
    }
    try {
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
      const address = accounts[0];
      const chainIdHex = await window.ethereum.request({ method: 'eth_chainId' });
      const chainId = String(parseInt(chainIdHex, 16));
      const domain = location.host;
      const uri = location.origin;
      const nonce = Math.random().toString(36).slice(2);
      const issuedAt = new Date().toISOString();
      const message = `${domain} wants you to sign in with your Ethereum account:\n${address}\n\nURI: ${uri}\nVersion: 1\nChain ID: ${chainId}\nNonce: ${nonce}\nIssued At: ${issuedAt}`;
      const signature = await window.ethereum.request({ method: 'personal_sign', params: [message, address] });
      const res = await fetch('/api/v1/auth/siwe', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ message, signature, chain_id: chainId }),
      });
      if (res.ok) {
        epsx.toast('Signed in! Reloading...', 'success');
        setTimeout(() => location.reload(), 500);
      } else {
        const err = await res.text();
        epsx.toast('Sign-in failed: ' + (err || res.status), 'error');
      }
    } catch (e) {
      epsx.toast('Error: ' + e.message, 'error');
    }
  },
  async demoLogin() {
    const res = await fetch('/api/v1/auth/demo', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({}),
    });
    if (res.ok) {
      epsx.toast('Demo session created!', 'success');
      setTimeout(() => location.reload(), 500);
    } else {
      epsx.toast('Demo login unavailable', 'warning');
    }
  }
};
</script>"##.to_string()
}

pub fn toaster_init() -> String {
    r##"<script>
// Toaster already initialized by global_js; this is a no-op hook.
</script>"##.to_string()
}
