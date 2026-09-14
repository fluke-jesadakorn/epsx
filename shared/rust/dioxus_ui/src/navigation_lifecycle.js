// Browser-only scrolling/focus adapter. Dioxus owns URL changes and rendering.
if (!window.__epsxNavigationLifecycle) {
  window.__epsxNavigationLifecycle = true;
  let pending = null;
  let frame = 0;
  const finish = () => {
    frame = 0;
    if (!pending || location.href !== pending.href) return;
    const main = document.getElementById('epsx-main-content');
    if (!main) return;
    const task = pending;
    // Keep the desired offset until the destination replaces its skeleton.
    if (task.position) window.scrollTo(...task.position);
    if (main.querySelector('[data-page-skeleton]')) return;
    if (task.hash) {
      let id;
      try { id = decodeURIComponent(task.hash.slice(1)); } catch (_) { id = task.hash.slice(1); }
      const anchor = document.getElementById(id);
      if (anchor) anchor.scrollIntoView();
    } else if (task.focus) {
      main.setAttribute('tabindex', '-1');
      main.focus({preventScroll: true});
    }
    pending = null;
  };
  const schedule = () => { if (!frame) frame = requestAnimationFrame(finish); };
  for (const method of ['pushState', 'replaceState']) {
    const original = history[method].bind(history);
    history[method] = function(state, unused, url) {
      const before = new URL(location.href);
      const position = [scrollX, scrollY];
      const result = original(state, unused, url);
      const after = new URL(location.href);
      if (before.href !== after.href) {
        const samePage = before.pathname === after.pathname;
        pending = {href: after.href, hash: after.hash, focus: !samePage,
          position: samePage ? position : [0, 0]};
        schedule();
      }
      return result;
    };
  }
  addEventListener('popstate', event => {
    pending = {href: location.href, hash: '', focus: false,
      position: Array.isArray(event.state) && event.state.length === 2 ? event.state : null};
    schedule();
  });
  new MutationObserver(() => { if (pending) schedule(); })
    .observe(document.body, {childList: true, subtree: true});
}

if (typeof dioxus !== "undefined") dioxus.send(location.origin);
