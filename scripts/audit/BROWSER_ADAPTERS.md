# Browser adapter audit

`cargo xtask audit no-node --strict` checks tracked files and untracked source before commit. Git-ignored files and untracked `node_modules` installs are excluded. Tracked dependencies are still audited.

Dioxus owns UI and state. Browser capability adapters are permitted only as reviewed exact Rust expressions or blocks in `xtask/src/browser_adapters.json`; each entry names its path and purpose. Dynamic adapters include script construction in the matched block. An additional or changed `document::eval` in the same file fails the gate. The standalone wallet adapter also requires the exact SHA-256 pinned in `xtask/src/node_free.rs`.

When changing a browser adapter, review its capability boundary before updating its exact manifest entry or hash. Do not automatically regenerate approvals from source. Wallet, clipboard, display preferences, push/service worker, and notification-stream adapters must return data to Dioxus rather than build UI. Run `python3 scripts/audit/dioxus_ownership.py` as the separate DOM-ownership gate, then the strict no-node audit. Unit tests verify changed scripts, additional evals, changed wallet bytes, and precommit inventory behavior.
