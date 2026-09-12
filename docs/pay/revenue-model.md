# EPSX Pay launch economics

Launch rates are 50 basis points for direct settlement and 100 basis points for released merchant escrow. Signup, links, dashboard, API and outgoing webhooks have no monthly charge. No subscription billing dependency is added to this release.

Gross fees in each token are the sum of its verified direct settlement fees (including those later refunded) and released escrow fees. Unreleased/refunded escrow earns zero. Fees on EPSX's own package purchases are internal transfers and must be excluded from consolidated platform revenue. Gas belongs to the submitting wallet and is not platform revenue. There is no automatic currency conversion; a dollar report requires an explicitly sourced valuation separately from token accounting.

| Completed monthly external volume (illustrative USD equivalent) | 80% direct / 20% released escrow gross fees |
|---:|---:|
| $10,000 | $60 |
| $100,000 | $600 |
| $1,000,000 | $6,000 |

The blended rate is 0.6%. These are scenarios, not forecasts or profit. Deduct RPC, machine/network costs, backups, monitoring, support, dispute handling and other operating costs. At $600 monthly operating costs, this mix needs about $100,000 completed external volume merely to cover those costs, before taxes or one-time development costs. Model refunds, token valuation and support cost per dispute separately.

Checked 8 September 2026: [NOWPayments publishes a 1% service fee](https://nowpayments.io/pricing); [BitPay's pricing page](https://www.bitpay.com/pricing) describes its processing tiers. Their settlement, conversion, networks and service scope differ; compare the complete service rather than treating transaction rates as equivalent products. EPSX's escrow rate equals the quoted NOWPayments percentage; its direct rate is lower.

Track external completed volume by token/mode, earned fees, full refunds, unreleased escrow, webhook success and retries, disputes per 100 payments, support time and RPC cost. Revisit optional reporting, branding and support plans only after measuring demand and cost. Keep transaction revenue as the only launch billing model.
