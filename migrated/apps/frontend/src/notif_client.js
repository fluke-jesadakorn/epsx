(function() {
  console.log('[notif-client] starting');
  const list = document.getElementById('notif-list');
  const empty = document.getElementById('notif-empty');
  const totalEl = document.getElementById('notif-total');
  const unreadEl = document.getElementById('notif-unread');
  const filterStatus = document.getElementById('notif-filter-status');
  const filterType = document.getElementById('notif-filter-type');
  const filterPriority = document.getElementById('notif-filter-priority');
  const btnMarkAll = document.getElementById('notif-mark-all');
  const btnClearAll = document.getElementById('notif-clear-all');

  let allItems = [];

  function colorForPriority(p) {
    if (p === 'urgent' || p === 'critical') return 'var(--epsx-red)';
    if (p === 'high') return 'var(--epsx-amber)';
    if (p === 'low') return 'var(--text-subtle)';
    return 'var(--epsx-blue-start)';
  }
  function iconForType(t) {
    if (t === 'security') return 'shield-halved';
    if (t === 'payment') return 'wallet';
    if (t === 'analytics') return 'chart-line';
    if (t === 'permission') return 'key';
    return 'bell';
  }
  function timeAgo(iso) {
    if (!iso) return '';
    const d = new Date(iso);
    const diff = (Date.now() - d.getTime()) / 1000;
    if (diff < 60) return 'just now';
    if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
    if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
    return Math.floor(diff / 86400) + 'd ago';
  }
  function esc(s) {
    return String(s == null ? '' : s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function render() {
    const status = filterStatus ? filterStatus.value : 'all';
    const ntype = filterType ? filterType.value : 'all';
    const prio = filterPriority ? filterPriority.value : 'all';
    const items = allItems.filter(function(i) {
      if (status === 'unread' && i.read_at) return false;
      if (status === 'read' && !i.read_at) return false;
      if (ntype !== 'all' && i.notification_type !== ntype) return false;
      if (prio !== 'all' && i.priority !== prio) return false;
      return true;
    });
    list.innerHTML = '';
    if (items.length === 0) {
      empty.style.display = '';
      list.style.display = 'none';
    } else {
      empty.style.display = 'none';
      list.style.display = '';
      items.forEach(function(i) {
        const color = colorForPriority(i.priority || 'normal');
        const icon = iconForType(i.notification_type || 'system');
        const unread = !i.read_at;
        const border = unread
          ? 'border-left:3px solid var(--epsx-orange);background:rgba(249,115,22,0.05);'
          : '';
        const el = document.createElement('div');
        el.className = 'card-insight';
        el.style.cssText = 'padding:1.25rem;' + border;
        const title = esc(i.title || 'Notification');
        const body = esc(i.body || '');
        const ntypeS = esc(i.notification_type || 'system');
        const prioS = esc(i.priority || 'normal');
        const ago = timeAgo(i.created_at);
        const markBtn = !i.read_at
          ? '<button class="mark-read-btn nav-link" style="padding:0.25rem;color:var(--epsx-blue-start);" title="Mark as read"><i class="fa-solid fa-check"></i></button>'
          : '';
        el.innerHTML =
          '<div style="display:flex;gap:1rem;align-items:flex-start;">' +
            '<div style="width:2.5rem;height:2.5rem;border-radius:9999px;background:' + color + '20;color:' + color + ';display:flex;align-items:center;justify-content:center;flex-shrink:0;">' +
              '<i class="fa-solid fa-' + icon + '"></i>' +
            '</div>' +
            '<div style="flex:1;min-width:0;">' +
              '<div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:0.25rem;">' +
                '<h3 style="font-size:0.9375rem;font-weight:' + (unread ? '700' : '600') + ';margin:0;">' + title + '</h3>' +
                (unread ? '<span style="width:0.5rem;height:0.5rem;border-radius:9999px;background:var(--epsx-orange);display:inline-block;flex-shrink:0;"></span>' : '') +
              '</div>' +
              '<p style="color:var(--text-muted);font-size:0.875rem;margin:0 0 0.5rem;line-height:1.5;">' + body + '</p>' +
              '<div style="font-size:0.75rem;color:var(--text-subtle);">' + ntypeS + ' &middot; ' + prioS + ' &middot; ' + ago + '</div>' +
            '</div>' +
            '<div style="display:flex;gap:0.25rem;flex-shrink:0;">' +
              markBtn +
              '<button class="delete-btn nav-link" style="padding:0.25rem;color:var(--epsx-red);" title="Delete"><i class="fa-solid fa-trash"></i></button>' +
            '</div>' +
          '</div>';
        const markBtnEl = el.querySelector('.mark-read-btn');
        if (markBtnEl) markBtnEl.addEventListener('click', async function() {
          await fetch('/api/v1/notifications/' + i.id + '/read', { method: 'POST', credentials: 'include' });
          i.read_at = new Date().toISOString();
          render();
        });
        const delBtn = el.querySelector('.delete-btn');
        delBtn.addEventListener('click', async function() {
          if (!confirm('Delete this notification?')) return;
          await fetch('/api/v1/notifications/' + i.id + '/delete', { method: 'POST', credentials: 'include' });
          allItems = allItems.filter(function(x) { return x.id !== i.id; });
          render();
        });
        list.appendChild(el);
      });
    }
    const unread = allItems.filter(function(i) { return !i.read_at; }).length;
    if (totalEl) totalEl.textContent = allItems.length + ' total, ' + unread + ' unread';
    if (unreadEl) unreadEl.textContent = unread;
  }

  async function load() {
    try {
      const res = await fetch('/api/v1/notifications?user_id=__USER_ID__&limit=50', { credentials: 'include' });
      if (!res.ok) { console.warn('notif fetch failed', res.status); return; }
      const data = await res.json();
      allItems = (data.items || []).filter(function(i) { return i.id; });
      render();
    } catch (e) { console.error(e); }
  }

  if (btnMarkAll) btnMarkAll.addEventListener('click', async function() {
    const res = await fetch('/api/v1/notifications/mark-all-read', { method: 'POST', credentials: 'include' });
    if (res.ok) {
      epsx.toast('Marked all read', 'success');
      allItems.forEach(function(i) { if (!i.read_at) i.read_at = new Date().toISOString(); });
      render();
    }
  });
  if (btnClearAll) btnClearAll.addEventListener('click', async function() {
    if (!confirm('Clear all notifications?')) return;
    const res = await fetch('/api/v1/notifications/clear-all', { method: 'POST', credentials: 'include' });
    if (res.ok) {
      epsx.toast('All notifications cleared', 'success');
      allItems = [];
      render();
    }
  });
  [filterStatus, filterType, filterPriority].forEach(function(el) {
    if (el) el.addEventListener('change', render);
  });
  load();
  setInterval(load, 60000);
})();
