# Merchant guide

Accept crypto payments with a hosted checkout, payment links, or your own integration.

## Set up your shop

1. Open **Dashboard** from the navigation.
2. Choose **Connect MetaMask** and review the sign-in message in your wallet. Signing in does not send a payment.
3. Enter your shop name and choose **Create shop**. Your connected wallet is the shop's receiving wallet.
4. Select **Test environment** while trying your integration. Switch to **Live environment** when you are ready to accept real payments. Keep your API keys and webhooks separate for each environment.

## Create a package

Open **Workspace → Packages**. Enter a name, description and price in USDT, USDC, or both. You can optionally specify a service duration in whole days. Packages are one-time purchases, with no automatic renewal.

Choose **Save package**, then **View storefront** to see the public catalog. **Edit package** lets you update prices and availability. Existing checkouts retain the terms shown when they were created.

## Share a payment link

Open **Workspace → Payment links**. Enter the item name, token and amount. Optionally limit the number of payments and choose an expiry. Select **Create link**, then **Open link** to review the checkout before sharing its URL.

Disabling a link stops new checkouts. Customers who already have a checkout can still complete it before its deadline.

## Follow payments

**Overview** summarizes confirmed payments and funds ready to collect. **Payments** shows payment and collection status separately. Open **View payment** to see its details and the actions currently available.

Collection, refunds and escrow actions require confirmation in your wallet and may require network gas. A submitted transaction is still pending until the required chain confirmations arrive. Keep the checkout URL so you can return to check its status.

Direct payment fees are 0.5%. Escrow fees are 1% when funds are released. Network gas is separate. Refunds require the merchant to fund the full original amount; token approval may be needed.

## Connect your server

In **Settings**, create a named API key and save it when displayed. Keep API keys on your server; never place them in browser code.

In **Webhooks**, enter a public HTTPS endpoint and save the signing secret when displayed. Verify signatures, handle duplicate events safely, and fulfill orders only after a verified payment event. See the [API reference](/docs) for request formats, signature verification and event types.

To replace an endpoint, enter the new URL and choose **Replace with URL above** on the old endpoint. Its delivery history is retained. **Delivery history** shows attempts and lets you replay retained events.

## Manage your shop

Use **Settings** to change the shop name or revoke API keys. Shop name changes apply to new checkouts; existing receipts keep their original details. Your receiving wallet cannot be changed through the shop name form.

## Having trouble?

- **Wallet not available:** open Pay in a browser with MetaMask installed and unlock your wallet, then connect again.
- **Wrong network:** review the network requested by checkout and switch in your wallet.
- **Payment still confirming:** use **Check again** or return to the same checkout later. Do not send another payment just because confirmation is taking time.
- **Expired or incomplete link:** ask the merchant for a new checkout URL, including its full payment token.
- **Service unavailable:** try **Refresh** after a short wait. Keep your checkout URL and transaction ID.
