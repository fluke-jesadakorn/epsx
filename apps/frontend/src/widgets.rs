//! Global widgets injected into every page (chat bubble, auth modal, toaster).

/// Floating chat widget shown on every authenticated page. Hidden on /chat.
pub fn chat_widget(is_authed: bool, user_id: &str) -> String {
    if !is_authed {
        return String::new();
    }
    // Keep the identity inside a JavaScript string literal instead of
    // interpolating it directly into the script.
    let user_id_js = serde_json::to_string(user_id).unwrap_or_else(|_| "\"\"".to_string());
    let js = format!(
        r##"<script>
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
  const userId = {user_id_js};
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
</script>"##,
        user_id_js = user_id_js
    );
    format!(
        r##"<div id="chat-widget" style="position:fixed;bottom:1.5rem;right:1.5rem;z-index:50;">
  <button class="chat-bubble-btn" aria-label="Open support chat" style="width:3.5rem;height:3.5rem;border-radius:9999px;background:linear-gradient(135deg,#3b82f6 0%,#2563eb 55%,#4f46e5 100%);color:white;border:none;cursor:pointer;box-shadow:0 10px 15px -3px rgba(0,0,0,.2),0 4px 6px -4px rgba(0,0,0,.2);display:flex;align-items:center;justify-content:center;position:relative;transition:all 0.3s;">
    <i data-lucide="message-circle" style="width:1.5rem;height:1.5rem;"></i>
    <span class="chat-bubble-badge" style="display:none;position:absolute;top:-0.375rem;right:-0.375rem;background:#ef4444;color:white;font-size:0.6875rem;font-weight:700;min-width:1.375rem;height:1.375rem;border-radius:9999px;padding:0 0.375rem;align-items:center;justify-content:center;border:2px solid hsl(var(--background));">0</span>
  </button>
  <div class="chat-panel" style="position:absolute;bottom:4.5rem;right:0;width:min(25rem,calc(100vw - 1rem));height:min(36.25rem,calc(100vh - 5rem));background:hsl(var(--card));border:1px solid hsl(var(--border));border-radius:1.5rem;box-shadow:0 25px 50px -12px rgba(0,0,0,.25);display:none;flex-direction:column;overflow:hidden;">
    <div class="chat-panel-close" style="padding:.875rem 1rem;border-bottom:1px solid hsl(var(--border));background:hsl(var(--muted));color:hsl(var(--foreground));display:flex;align-items:center;justify-content:space-between;cursor:pointer;">
      <div style="display:flex;align-items:center;gap:.75rem;">
        <span style="width:2.25rem;height:2.25rem;border-radius:.75rem;background:linear-gradient(135deg,#3b82f6 0%,#4f46e5 100%);color:white;display:flex;align-items:center;justify-content:center;box-shadow:0 4px 10px rgba(59,130,246,.2);"><i data-lucide="message-circle" style="width:1rem;height:1rem;"></i></span>
        <span style="display:flex;flex-direction:column;gap:.125rem;">
          <span style="font-weight:700;">Support</span>
          <span style="font-size:.625rem;color:hsl(var(--muted-foreground));"><span style="display:inline-block;width:.375rem;height:.375rem;border-radius:9999px;background:#34d399;margin-right:.25rem;"></span>Online - replies within minutes</span>
        </span>
      </div>
      <i data-lucide="x" style="width:1rem;height:1rem;color:hsl(var(--muted-foreground));"></i>
    </div>
    <div style="flex:1;padding:1.25rem;overflow-y:auto;display:flex;flex-direction:column;gap:0.75rem;background-image:radial-gradient(circle,rgba(118,69,217,.045) 1px,transparent 1px);background-size:22px 22px;">
      <p style="color:hsl(var(--muted-foreground));font-size:0.875rem;text-align:center;margin:auto 0;">Hi! How can we help?</p>
    </div>
    <a href="/chat" style="margin:0.5rem 1rem 1rem;padding:0.625rem;border-radius:0.5rem;background:linear-gradient(135deg,#3b82f6 0%,#4f46e5 100%);color:white;text-align:center;text-decoration:none;font-weight:600;font-size:0.875rem;">Open Full Chat</a>
  </div>
</div>
<style>
.chat-panel.open {{ display:flex !important; opacity:1; transform:scale(1) !important; }}
.chat-bubble-btn:hover {{ transform:scale(1.05); box-shadow:0 20px 25px -5px rgba(59,130,246,.25),0 8px 10px -6px rgba(59,130,246,.25); }}
.chat-panel {{ position:fixed !important; bottom:.5rem !important; right:.5rem !important; width:calc(100vw - 1rem) !important; height:calc(100vh - 4rem) !important; max-width:25rem !important; max-height:36.25rem !important; }}
@media (min-width:768px) {{ .chat-panel {{ bottom:6rem !important; right:1.5rem !important; width:25rem !important; height:36.25rem !important; }} }}
</style>
{js}"##,
        js = js
    )
}
