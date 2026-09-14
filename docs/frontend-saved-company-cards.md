# Saved companies — draggable cards

Implemented locally on 8 September 2026. Scope: `/portfolio` and its frontend-only browser interactions. Backend contracts, ownership checks and saved-company membership rules are unchanged.

The page uses responsive company cards with prominent symbols, a drag handle, a group selector and an actions menu. Group sections can also be reordered. Pointer dragging lifts the card, shows a dashed destination and animates surrounding cards into position. Keyboard users can pick up with Space, choose a position with arrows, change groups with Tab, drop with Space/Enter, or cancel with Escape. Reduced-motion mode removes rotation and reflow animations.

Layout changes use the existing `PUT /api/users/watchlist/layout` route. The page remains open while saving. The returned owner layout must acknowledge the requested order and memberships before the UI reports success. Failures and unchanged or malformed responses restore the previous layout. Counts, membership controls, empty-group hints and focus update in place. Other existing add, remove, rename and group-creation flows retain their existing server behavior.

The signed-out page has a new explanatory layout and retains `/auth?return_url=%2Fportfolio`. It does not render sample saved companies. Port 3300 connects to the real local backend on 8080. Authenticated UI tests use an explicitly marked, isolated fixture preview on 3311, with its own BFF on 3310 and fixture authority/adapter on 48081/48082. No real wallet session or owner data was created for testing.

## Validation

- Portfolio Rust tests: **9 passed**.
- Browser-runtime host tests: **13 passed**, including canonical acknowledgement and duplicate-membership cases.
- Host Clippy for frontend and browser runtime, Rust formatting, native frontend build, WASM release build and asset verification: **passed**.
- Native browser interactions against the Rust SSR + WASM fixture preview: **12 passed**. Covered pointer reorder; keyboard reorder; group reorder; cross-group and empty-group moves; reload persistence; focus; Escape; pending/busy state; failed saves; HTTP 200 without acknowledgement; duplicate memberships; native select alternative.
- Responsive screenshots at **375, 768, 1024, 1280 and 1440px** in both themes: no horizontal overflow. Mobile card handles are at least 44px. Reduced-motion inspection confirmed no running animations or ghost rotation.
- Admin and Pay local HTTP smoke: both returned 200 and do not load the frontend stylesheet or saved-card board.
- Repository E2E doctor: passed. The full repository E2E scenario suite was not run for this change; the interaction results above are the targeted real-browser checks.
- WASM Clippy remains blocked by **three pre-existing diagnostics**: `possible_missing_else` in `browser/merchant_pay.rs`, and `useless_format` / `let_unit_value` in `browser-runtime/src/lib.rs`. No new saved-card module diagnostics were reported.

Evidence and source backups are in `target/saved-company-cards/`: interaction/membership/responsive JSON reports, screenshots, build and lint logs, and `before/`. The loopback fixture adapter now preserves ungrouped order and can return an unchanged canonical layout to exercise acknowledgement failures accurately.

Testing did not include signing a real wallet or touch input on physical mobile hardware. Pointer interaction was exercised with native browser mouse events; responsive touch target sizes were measured separately. No deployment was performed.
