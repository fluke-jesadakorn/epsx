# Merchant dev validation — 9 September 2026

Deployed native release: `20260909-merchant-v5`. Previous release/configuration: `20260909-merchant-v4`. Dev only; existing Anvil 31337 contracts and loopback RPC retained.

- Dashboard: https://dev-pay.epsx.io/dashboard
- Demo store: https://dev-pay.epsx.io/m/mer_3bcbcbb085794a76b3286c9ad56c3038
- Demo package: https://dev-pay.epsx.io/packages/pkg_4fb12bf6d1024ffabc1dc51ab300c1f9
- [Merchant/API/webhook/rollback guide](merchant-dashboard-dev.md)

## Verified

| Check | Result |
|---|---|
| Wallet-authenticated catalog ownership | Two independent SIWE test wallets; cross-shop read/update rejected |
| Snapshot and disabled package | Original name/price/duration/payee preserved; new purchases rejected; same-key retry preserved |
| Dev USDT + USDC | 5 tokens each paid, scanner confirmed, shop collected to its wallet |
| Dev direct refund | Full 5 USDT refund confirmed, original processing fee retained |
| EPSX Plan regression | Order `85c540dc-5e33-4550-bdaf-989e46ccc4ce`: webhook HTTP 200 on first attempt, access granted, admin record matched |
| Restart | Payment/refund/collection and EPSX grant survived service + Anvil restart |
| Webhook integration | Isolated HTTP receiver tested retries, signatures, duplicates/rotation, endpoint ownership and replacement history |
| Browser | Separate headless Chrome profile, 1440px/390px, light/dark, no horizontal overflow |
| Browser actions | Create/edit/disable package; create link → QR checkout; QR/Wallet switch; reload retains receiver; generic receipt |
| Rust | Targeted native tests passed; isolated PostgreSQL/Anvil integration passed |
| Quality gates | Workspace Clippy, Wasm Clippy, formatting, asset verification, strict no-node audit passed |
| Contracts | Foundry: 33 passed |

[Payment/collection/refund evidence](evidence/dev-20260909/merchant-proof.json), [EPSX Plan evidence](evidence/dev-20260909/merchant-epsx-plan-regression.json), [browser checks](evidence/dev-20260909/merchant-ui/validation.json), [dashboard screenshot](evidence/dev-20260909/merchant-ui/overview-authenticated.png).

## Pending interactive acceptance

The actual dev transactions and SIWE sessions above were signed with dedicated test-wallet CLI keys. Interactive merchant MetaMask login → customer payment → merchant collection has not been verified: computer use reported the Mac locked and asked the user to unlock it. The existing checkout MetaMask functionality remains in place, but this result must not be presented as new interactive MetaMask proof.

WalletConnect still needs a valid project ID. Mobile-wallet payment cannot reach the private Anvil RPC and is outside this acceptance. A merchant's external service URL was not supplied; no arbitrary third-party webhook receiver was configured. Generic webhook delivery was tested against the isolated receiver, and EPSX's real dev webhook was independently verified over HTTPS.
