# Admin menu organization

The sidebar, desktop dropdowns and mobile navigation share the inventory in
`shared/rust/dioxus_ui/src/layout/sidebar.rs`. The active fullstack shell uses
four dropdown/mobile sections: Workspace, Manage, Content & support, and System.
The sidebar retains expandable feature groups from the same inventory.

| Feature group | Destinations |
| --- | --- |
| Workspace | Dashboard, Analytics |
| Wallets & access | Wallets & permissions, Subscriptions, Plan catalog, Credits |
| Payments | Plan purchases, Payment intents, Payment links, Escrows, Merchant escrows |
| Content | News, Create news, Media library |
| Support | Chat support, Manage notifications, Send notification |
| Developer | Overview, API keys, Create API key, Usage, Documentation |
| Settings | General, Notification preferences, Security, Appearance, Audit log |

Wallets and the former Access entry render the same canonical wallet interface;
only Wallets & permissions is listed. Subscription access is a distinct backend
projection and remains reachable under Wallets & access using its existing
`/payments?tab=user-access` URL. The plan catalog has one entry at `/plans`;
legacy wallet-plan URLs still work. Authentication appears once per menu and
is hidden after sign-in.

No page routes or backend permissions were changed. Detail/edit/disable routes
remain contextual actions under their list entry. `/users` renders a not-found
page and is intentionally absent. Alias URLs do not receive duplicate entries.

Selection uses the most specific leaf, query tabs and page defaults. Existing
`/admin` prefixes, wallet and plan aliases, and dashboard aliases select their
canonical entry. Parent expansion follows the selected child rather than a URL
prefix, so Subscriptions opens Wallets & access, Escrows opens Payments, and
Audit log opens Settings. Desktop and mobile receive the same selected leaf.

## Verification

- 43 layout tests and one fullstack admin menu test passed, including route
  resolution, unique destinations, alias/detail/tab selection, parent expansion,
  and authenticated/guest menu presentation.
- `cargo fmt --all --check`, strict no-node audit and asset verification passed.
- The general navigation inventory check flags two existing native block-explorer
  anchors in `fullstack/pay/checkout.rs` and `fullstack/pay/ui.rs`; neither is
  changed by this work.

- All 27 authenticated desktop destinations were clicked with the same document
  and shell retained, exactly one selected menu entry, and menu closure.
- Desktop screenshots at 1440 and 1101 pixels and signed mobile checks at 390
  pixels show no horizontal overflow. Mobile contains the same 27 destinations,
  closes after navigation, and preserves the document through Back/Forward.
- Six direct alias/default/detail URLs select their canonical destination.
- Two combined crawl attempts timed out on the browser command immediately
  after resizing to mobile. Separate signed mobile runs starting on both Plans
  and Audit log passed, including layout, navigation and direct-link checks.
  The evidence therefore combines the desktop crawl with separate mobile runs.
- Fixture upstreams intentionally lack some services: News/Media return 502 and
  escrow reads return 404 while their navigation and shells render. This audit
  verifies navigation, not production data availability or backend operations.
- The updated development server returns HTTP 200 with the new menu at `/`,
  `/auth` and `/settings?tab=general`; its realtime LaunchAgent remains running
  with automatic rebuilds disabled. No fixture writes were made.

Browser screenshots and reports are under `target/navigation-audit/admin-menu/`;
local HTTP evidence is `target/navigation-audit/admin-dev-http.json`. This change
does not deploy production.
