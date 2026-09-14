# EPSX sales flow (dev)

The catalog stores regular USDT/USDC prices in `plan_metadata.pay_prices`.
`pay_use_catalog_promotion: true` applies the catalog's `promotion` to those
prices. It is opt-in so existing independently priced tokens retain their terms.

The Rust backend calculates the regular price, final price, savings and promotion
status once using decimal arithmetic. Public plans, quotes and order creation use
the same calculation. Percentage discounts, fixed discounts, optional sale-price
overrides and UTC schedules are supported. Invalid discounts or dates are rejected.
Disabled, future and expired promotions charge the regular price.

Use [dev admin plans](https://dev-admin.epsx.io/plans) to edit regular prices and
promotion settings. The editor preserves features, access rules and unrelated
metadata. Only backend-authorized administrators may save changes.

[Plan cards](https://dev.epsx.io/plans) show the original price, sale price and
savings. Purchase review shows the price breakdown before continuing to Pay.
Each new order saves `pricing_snapshot` alongside its exact token amount. Pay
receives that snapshot and displays it with the QR and wallet checkout. Editing
or ending a promotion cannot change an existing order or idempotent retry.
Old orders retain their amounts and have an empty breakdown; no historical sale
claims are invented. Network gas remains separate from the plan price.

Production's public catalog supplied the initial sale amounts; production was
read only. The regular/sale prices are 5/1, 99/9.90, 3999/999, 6999/2999 and
9999/4999 for Day, Month, API Personal, API Company and Lifetime respectively.
They are configuration, not plan-specific code branches.

The additive core migration adds only `pay_purchase_orders.pricing_snapshot`.
A private core backup is taken before migration. Anvil and the tunnel are not
restarted during release activation.

Status and restart:

```sh
python3 infrastructure/native/dev-control.py status
python3 infrastructure/native/dev-control.py restart bff-frontend
```

Do not switch straight to a pre-sales backend while the catalog contains base
token prices: that backend does not understand the promotion policy. Before a
rollback to `20260909-merchant-v5`, run the compatibility preparation while the
new backend is available, followed by binary rollback:

```sh
python3 infrastructure/native/prepare-sales-rollback.py --apply
python3 infrastructure/native/dev-control.py rollback
```

The helper reads exact current quotes from the Rust backend, saves a private
catalog backup, converts active token promotions to explicit checkout prices,
and preserves their regular prices as `pay_base_prices` in metadata. Run it
without `--apply` to preview. This is a forward
catalog compatibility change, not a database restore; never revert purchase,
webhook or entitlement history. Promotion-aware releases require only the usual
binary/configuration rollback.

Validation completed on 2026-09-09 for dev release `20260909-sales-flow`:

- 29 targeted Rust tests passed, plus native and Wasm Clippy, formatting, assets,
  and the no-Node audit. See [checks](evidence/dev-20260909/sales-checks.json).
- The additive migration was applied twice to a restored dev database without
  changing existing orders. See [migration proof](evidence/dev-20260909/sales-migration.json).
- All five plans match the production regular and sale amounts; all ten USDT/USDC
  quotes agree. Admin promotion edits, disabled/upcoming/expired sales, new order
  prices and immutable retries passed on an isolated catalog row that is now
  disabled and private. See [pricing proof](evidence/dev-20260909/sales-flow.json).
- Anvil transfer `0x482727836c8cc21aa1fe96c4abfe59991bd0de7bac343652a9fcfaff94d62618`
  paid 1 USDT. The webhook granted access and the buyer/admin views agreed.
  See [payment proof](evidence/dev-20260909/sales-payment.json).
- Browser checks covered sale cards and Pay on desktop/mobile with light/dark
  themes, wallet/QR switching, and the confirmed receipt. All four dev domains
  returned successful health responses. See [deployment proof](evidence/dev-20260909/sales-deployment.json).

The native executables and Wasm were successfully compiled. Packaging used the
same copy layout and SHA-256 manifest as `xtask native package` after stopping
its redundant second native rebuild. All 366 packaged files were verified.
Production routes, contracts and data were unchanged.
