//! Interactive showcase & component playground.
//!
//! Live demos of toasts, modals, tabs, form validation, dropdowns, and
//! accordion behavior, all wired up to the global `epsx.*` JS controllers.

use epsx_templates::components::{Badge, BadgeKind, Btn, Input, Skeleton, StatCard, Tabs};

pub fn interactive_body() -> String {
    let toast_section = format!(
        r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="info" style="color:var(--epsx-blue-start);"></i> Toast Notifications
    </h2>
    <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1rem;">Click a button to fire a toast. Auto-dismisses after 3.5s.</p>
    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
      <button class="btn btn-primary btn-sm" onclick="epsx.toast('Saved successfully!', 'success')">
        <i data-lucide="check"></i> Success
      </button>
      <button class="btn btn-danger btn-sm" onclick="epsx.toast('Something went wrong!', 'error')">
        <i data-lucide="x"></i> Error
      </button>
      <button class="btn btn-outline btn-sm" onclick="epsx.toast('Heads up — read this.', 'warning')">
        <i data-lucide="alert-triangle"></i> Warning
      </button>
      <button class="btn btn-ghost btn-sm" onclick="epsx.toast('FYI: New release available.', 'info')">
        <i data-lucide="info"></i> Info
      </button>
    </div>
  </div>"##
    );

    let modal_btn = Btn::new("Open Demo Modal").gradient().icon_left("window-maximize").onclick("epsx.openModal(modalDemoHtml)").render();
    let modal_section = format!(
        r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="maximize" style="color:var(--epsx-purple);"></i> Modal Dialogs
    </h2>
    <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1rem;">Click outside or press Escape to close.</p>
    {modal_btn}
  </div>"##,
        modal_btn = modal_btn
    );

    let tabs_basic = Tabs::new("demo-basic")
        .tab("a", "Overview")
        .tab("b", "Details")
        .tab("c", "Settings")
        .active("a")
        .render();
    let tabs_section = format!(
        r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="folder-tree" style="color:var(--epsx-orange);"></i> Tabs
    </h2>
    {tabs}
    <div data-tab-group="demo-basic" data-tab-name="a" class="card-insight" style="padding:1rem;margin-top:1rem;background:var(--bg-secondary);">
      <p style="margin:0;color:var(--text-muted);">Overview content — only this tab is visible right now.</p>
    </div>
    <div data-tab-group="demo-basic" data-tab-name="b" class="card-insight" style="padding:1rem;margin-top:1rem;background:var(--bg-secondary);display:none;">
      <p style="margin:0;color:var(--text-muted);">Details content — switch to this tab to see it.</p>
    </div>
    <div data-tab-group="demo-basic" data-tab-name="c" class="card-insight" style="padding:1rem;margin-top:1rem;background:var(--bg-secondary);display:none;">
      <p style="margin:0;color:var(--text-muted);">Settings content — switch to this tab to see it.</p>
    </div>
  </div>"##,
        tabs = tabs_basic
    );

    let name = Input::new("demo-name").label("Your Name").placeholder("Jane Doe").required().render();
    let email = Input::new("demo-email").label("Email").email().required().placeholder("you@example.com").icon("envelope").render();
    let msg = Input::new("demo-msg").label("Message").textarea().rows(3).required().placeholder("Tell us something...").render();

    let form_section = format!(
        r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="list" style="color:var(--epsx-green);"></i> Form Validation
    </h2>
    <form id="demo-form" style="display:grid;gap:1rem;" onsubmit="event.preventDefault(); epsx.toast('Form submitted!', 'success'); document.getElementById('demo-form').reset();">
      {name}
      {email}
      {msg}
      <button type="submit" class="btn btn-gradient">
        <i data-lucide="send"></i> Submit
      </button>
    </form>
  </div>"##,
        name = name, email = email, msg = msg
    );

    let skeleton_section = format!(
        r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="loader" style="color:var(--epsx-cyan);"></i> Skeleton Loaders
    </h2>
    <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1rem;">Animated placeholders during data fetch.</p>
    <div style="display:grid;gap:1rem;">
      {single}
      {multi}
    </div>
  </div>"##,
        single = Skeleton::new().w("100%").h("1rem").render(),
        multi = Skeleton::new().count(3).gap(8).render()
    );

    let dropdown_section = r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="chevron-down" style="color:var(--epsx-amber);"></i> Dropdowns
    </h2>
    <p style="color:var(--text-muted);font-size:0.875rem;margin-bottom:1rem;">Click trigger to open. Click outside or Escape to close.</p>
    <div style="position:relative;display:inline-block;">
      <button class="btn btn-outline" data-dropdown-trigger onclick="epsx.toggleDropdown('demo-dropdown')">
        Options <i data-lucide="chevron-down"></i>
      </button>
      <div id="demo-dropdown" class="dropdown-menu" style="display:none;position:absolute;top:calc(100% + 0.25rem);right:0;min-width:12rem;background:var(--bg-solid);border:1px solid var(--border);border-radius:0.5rem;box-shadow:var(--shadow-lg);padding:0.5rem;z-index:50;">
        <a href="#" class="nav-link" style="width:100%;display:flex;gap:0.5rem;align-items:center;"><i data-lucide="pencil"></i> Edit</a>
        <a href="#" class="nav-link" style="width:100%;display:flex;gap:0.5rem;align-items:center;"><i data-lucide="copy"></i> Duplicate</a>
        <a href="#" class="nav-link" style="width:100%;display:flex;gap:0.5rem;align-items:center;color:var(--epsx-red);"><i data-lucide="trash-2"></i> Delete</a>
      </div>
    </div>
  </div>"##;

    let badges = [
        Badge::new("Default").default().render(),
        Badge::new("Primary").primary().render(),
        Badge::new("Success").success().icon("check").render(),
        Badge::new("Warning").warning().icon("exclamation").render(),
        Badge::new("Danger").danger().icon("xmark").render(),
        Badge::new("Info").info().render(),
        Badge::new("Brand").brand().render(),
        Badge::new("Cool").cool().render(),
        Badge::new("Warm").warm().icon("fire").render(),
        Badge::new("Purple").purple().render(),
        Badge::new("Outline").outline().render(),
    ].join(" ");

    let badge_section = format!(
        r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="tag" style="color:var(--epsx-pink);"></i> Badges
    </h2>
    <div style="display:flex;flex-wrap:wrap;gap:0.5rem;">
      {badges}
    </div>
  </div>"##,
        badges = badges
    );

    let accordion_section = r##"<div class="card-insight" style="padding:1.5rem;">
    <h2 style="font-size:1.25rem;font-weight:700;margin:0 0 1rem;display:flex;align-items:center;gap:0.5rem;">
      <i data-lucide="menu" style="color:var(--epsx-cyan);"></i> Accordion
    </h2>
    <div style="display:grid;gap:0.5rem;">
      <div class="nav-accordion" id="acc-1">
        <button class="nav-accordion-trigger" onclick="epsx.toggleNavAccordion('acc-1')" style="width:100%;">
          <span><i data-lucide="help-circle" style="color:var(--epsx-orange);margin-right:0.5rem;"></i> What is EPSX?</span>
          <i data-lucide="chevron-right"></i>
        </button>
        <div class="nav-accordion-content" style="padding:0.75rem 1rem;color:var(--text-muted);font-size:0.875rem;">
          EPSX is a web3 analytics and payments platform on BSC, providing on-chain rankings, subscription vaults, and stablecoin payments.
        </div>
      </div>
      <div class="nav-accordion" id="acc-2">
        <button class="nav-accordion-trigger" onclick="epsx.toggleNavAccordion('acc-2')" style="width:100%;">
          <span><i data-lucide="shield" style="color:var(--epsx-green);margin-right:0.5rem;"></i> Is it secure?</span>
          <i data-lucide="chevron-right"></i>
        </button>
        <div class="nav-accordion-content" style="padding:0.75rem 1rem;color:var(--text-muted);font-size:0.875rem;">
          All payments are settled on-chain via audited smart contracts. No custodial risk.
        </div>
      </div>
      <div class="nav-accordion" id="acc-3">
        <button class="nav-accordion-trigger" onclick="epsx.toggleNavAccordion('acc-3')" style="width:100%;">
          <span><i data-lucide="fuel" style="color:var(--epsx-blue-start);margin-right:0.5rem;"></i> Do users need BNB?</span>
          <i data-lucide="chevron-right"></i>
        </button>
        <div class="nav-accordion-content" style="padding:0.75rem 1rem;color:var(--text-muted);font-size:0.875rem;">
          No! Our paymaster sponsors gas and charges users in stablecoins via ERC-4337.
        </div>
      </div>
    </div>
  </div>"##;

    let js = r##"<script>
const modalDemoHtml = `
  <div style="padding:2rem;max-width:32rem;text-align:center;">
    <div style="width:4rem;height:4rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
      <i data-lucide="rocket" style="color:white;font-size:1.5rem;"></i>
    </div>
    <h2 style="font-size:1.5rem;font-weight:800;margin-bottom:0.5rem;">Ready to launch?</h2>
    <p style="color:var(--text-muted);margin-bottom:1.5rem;">This is a vanilla JS modal powered by epsx.openModal(). No framework needed.</p>
    <div style="display:flex;gap:0.5rem;justify-content:center;">
      <button class="btn btn-outline" onclick="epsx.closeModal()">Cancel</button>
      <button class="btn btn-gradient" onclick="epsx.toast('Launched!', 'success'); epsx.closeModal();">
        <i data-lucide="rocket"></i> Launch
      </button>
    </div>
  </div>
`;

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') epsx.closeModal();
});
</script>"##;

    let hero = r##"<div style="text-align:center;margin-bottom:3rem;">
    <span class="badge-pill"><i data-lucide="sparkles" style="color:var(--epsx-orange);"></i> Interactive</span>
    <h1 class="gradient-text" style="font-size:3rem;font-weight:800;margin:1rem 0 1rem;">Component Playground</h1>
    <p style="font-size:1.125rem;color:var(--text-muted);max-width:42rem;margin:0 auto;">Live demos of every interactive component &mdash; toasts, modals, tabs, forms, dropdowns, accordions, badges, and skeletons. All running on vanilla JS via the global <code style="background:var(--bg-secondary);padding:0.125rem 0.5rem;border-radius:0.25rem;font-size:0.875rem;">window.epsx</code> namespace.</p>
  </div>"##;

    let s1 = StatCard::new("Components", "8").icon("cubes", "var(--epsx-orange)").render();
    let s2 = StatCard::new("JS Functions", "11").icon("code", "var(--epsx-blue-start)").render();
    let s3 = StatCard::new("No Framework", "Vanilla").icon("leaf", "var(--epsx-green)").render();
    let s4 = StatCard::new("Bundle", "0 KB").icon("feather", "var(--epsx-purple)").render();

    format!(
        r##"<section class="section">
<div class="container-x" style="max-width:64rem;">
  {hero}

  <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(180px, 1fr));gap:1rem;margin-bottom:2rem;">
    {s1}
    {s2}
    {s3}
    {s4}
  </div>

  <div style="display:grid;grid-template-columns:1fr 1fr;gap:1.5rem;margin-bottom:1.5rem;">
    {toast}
    {modal}
    {tabs}
    {dropdown}
    {form}
    {skeleton}
  </div>

  <div style="display:grid;grid-template-columns:1fr 1fr;gap:1.5rem;">
    {accordion}
    {badges}
  </div>
</div>
{js}
</section>"##,
        hero = hero,
        s1 = s1, s2 = s2, s3 = s3, s4 = s4,
        toast = toast_section,
        modal = modal_section,
        tabs = tabs_section,
        dropdown = dropdown_section,
        form = form_section,
        skeleton = skeleton_section,
        accordion = accordion_section,
        badges = badge_section,
        js = js
    )
}
