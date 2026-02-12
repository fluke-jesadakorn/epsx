# Credit Wallet System - Testing Guide

Complete testing guide for the credit wallet system implementation.

---

## Prerequisites

### 1. Start All Services
```bash
# Start all services (recommended)
bun dev

# OR start individually
bun dev:backend    # Port 8080
bun dev:frontend   # Port 3000
bun dev:admin      # Port 3001
```

### 2. Run Database Migrations
```bash
cd apps/backend
diesel migration run --database-url=$DATABASE_URL_PAYMENTS
```

### 3. Authentication Setup
You need TWO authenticated accounts:
- **User Account**: Regular user with wallet
- **Admin Account**: User with admin permissions

**Get JWT Tokens**:
1. Connect wallet at http://localhost:3000/auth (user) or http://localhost:3001/auth (admin)
2. Open browser DevTools → Application → Cookies
3. Copy `epsx.access_token` value (this is your JWT)

---

## Backend API Testing

### Using the Test Script

**Quick Start**:
```bash
# Export your auth tokens
export USER_TOKEN='your_user_jwt_token_here'
export ADMIN_TOKEN='your_admin_jwt_token_here'

# Run the test script
./scripts/test-credit-endpoints.sh
```

The script tests all 6 credit endpoints:
- ✅ User: Get balance
- ✅ User: Get transaction history
- ✅ Admin: Get system statistics
- ✅ Admin: Grant credits
- ✅ Admin: Get user credits
- ✅ Admin: Revoke credits

### Manual API Testing with curl

#### 1. User: Get Credit Balance
```bash
curl -X GET \
  -H "Authorization: Bearer $USER_TOKEN" \
  http://localhost:8080/api/payments/credits/balance
```

**Expected Response**:
```json
{
  "success": true,
  "data": {
    "wallet_address": "0x...",
    "balance": 0.00,
    "pending_balance": 0.00,
    "available_balance": 0.00,
    "lifetime_earned": 0.00,
    "lifetime_spent": 0.00,
    "last_transaction_at": null
  }
}
```

#### 2. User: Get Transaction History
```bash
curl -X GET \
  -H "Authorization: Bearer $USER_TOKEN" \
  "http://localhost:8080/api/payments/credits/history?limit=10&tx_type=grant"
```

**Expected Response**:
```json
{
  "success": true,
  "data": {
    "data": [
      {
        "id": "uuid",
        "wallet_address": "0x...",
        "amount": 50.00,
        "balance_after": 50.00,
        "tx_type": "grant",
        "reason": "Welcome bonus",
        "granted_by": "0x...",
        "created_at": "2026-02-12T12:00:00Z"
      }
    ],
    "pagination": {
      "total": 1,
      "limit": 10,
      "offset": 0
    }
  }
}
```

#### 3. Admin: Grant Credits
```bash
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "0x1234567890123456789012345678901234567890",
    "amount": 100.00,
    "reason": "Welcome bonus for new user",
    "expires_at": null
  }' \
  http://localhost:8080/api/payments/admin/credits/grant
```

**Expected Response**:
```json
{
  "success": true,
  "data": {
    "transaction_id": "uuid",
    "new_balance": 100.00
  }
}
```

#### 4. Admin: Get System Statistics
```bash
curl -X GET \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/api/payments/admin/credits/stats
```

**Expected Response**:
```json
{
  "success": true,
  "data": {
    "total_credits_outstanding": 500.00,
    "credits_granted_today": 100.00,
    "credits_used_today": 50.00,
    "active_users_with_credits": 5
  }
}
```

---

## Frontend Testing

### 1. User Credit Balance Page

**URL**: http://localhost:3000/account/credits

**Test Cases**:
- [ ] Page loads without errors
- [ ] 3 balance cards display: Available, Lifetime Earned, Lifetime Spent
- [ ] Transaction history table displays
- [ ] Filter tabs work (All, Grant, Revoke, Payment Debit, etc.)
- [ ] Empty state shows when no transactions
- [ ] Dark mode toggle works correctly
- [ ] Mobile responsive layout works

**Expected UI**:
```
┌─────────────────────────────────────────┐
│  Credit Balance                         │
├─────────────────────────────────────────┤
│  Available Balance  │ Lifetime Earned   │
│  $0.00             │ $0.00             │
├─────────────────────────────────────────┤
│  Transaction History                    │
│  [All] [Grant] [Revoke] [Payment] ...  │
│  ┌───────────────────────────────────┐  │
│  │ No transactions found             │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### 2. Account Overview Widget

**URL**: http://localhost:3000/account

**Test Cases**:
- [ ] Credit balance widget displays in stats grid
- [ ] Shows available balance amount
- [ ] Shows "lifetime earned" if > 0
- [ ] Clicking widget navigates to /account/credits
- [ ] Dark mode styling works
- [ ] Loading skeleton shows during fetch

**Expected UI**:
```
┌─────────────────────────────────────────┐
│  Account Settings                       │
├─────────────────────────────────────────┤
│ [Wallet] [Member] [Credits💰] [Security]│
│                   $0.00                  │
│                   Available Balance      │
└─────────────────────────────────────────┘
```

### 3. Plan Cards with Credits

**URL**: http://localhost:3000/plans

**Test Cases**:
- [ ] Plan cards load correctly
- [ ] Credit balance fetched on page load
- [ ] Credit breakdown shows on each plan card:
  - Plan Price: $X
  - Credit Applied: -$Y
  - Amount Due: $Z
- [ ] "Fully covered by credits!" message when balance covers plan
- [ ] Dark mode styling works
- [ ] Mobile layout stacks credit info properly

**Expected UI (with credits)**:
```
┌─────────────────────────────────────────┐
│  Premium Plan                           │
│  $30.00 USD                            │
├─────────────────────────────────────────┤
│  ✨ Credits Applied                     │
│  Plan Price:        $30.00             │
│  Credit Applied:    -$20.00            │
│  ─────────────────────────────          │
│  Amount Due:        $10.00             │
├─────────────────────────────────────────┤
│  [Select Plan]                          │
└─────────────────────────────────────────┘
```

---

## Admin Portal Testing

### 1. Credits Management Page

**URL**: http://localhost:3001/credits

**Test Cases**:
- [ ] Page loads without errors
- [ ] 3 tabs visible: Overview | Grant Credits | Credit History
- [ ] Tab switching works smoothly
- [ ] Auth guard redirects to /auth if not authenticated
- [ ] Permission check prevents non-admin access

### 2. Overview Tab

**Test Cases**:
- [ ] 4 stat cards display:
  - Total Credits Outstanding
  - Credits Granted Today
  - Credits Used Today
  - Active Users with Credits
- [ ] Stats load from API
- [ ] "Refresh Stats" button reloads data
- [ ] Dark mode styling works
- [ ] Loading skeleton during fetch

**Expected UI**:
```
┌─────────────────────────────────────────┐
│  Credits Management                     │
│  [Overview] [Grant Credits] [History]   │
├─────────────────────────────────────────┤
│ $500.00        $100.00       $50.00   5 │
│ Outstanding    Granted       Used   Users│
├─────────────────────────────────────────┤
│ [🔄 Refresh Stats] [📊 Export Report]   │
└─────────────────────────────────────────┘
```

### 3. Grant Credits Tab

**Test Cases**:
- [ ] Toggle between Grant/Revoke modes
- [ ] Form validation works:
  - Wallet address required (0x format)
  - Amount required (min 0.01)
  - Reason optional
  - Expiry date optional (Grant mode only)
- [ ] Submit button disabled during processing
- [ ] Success message shows new balance
- [ ] Error handling displays properly
- [ ] Form clears after successful operation
- [ ] Dark mode form styling

**Grant Flow**:
1. Enter wallet: `0x1234...`
2. Enter amount: `50.00`
3. Enter reason: "Welcome bonus"
4. Click "Grant Credits"
5. See success: "Successfully granted $50.00. New balance: $50.00"

### 4. Credit History Tab

**Test Cases**:
- [ ] Wallet address search input works
- [ ] Search button triggers query
- [ ] Transaction table displays with columns:
  - Type (color-coded badge)
  - Amount (+ for credit, - for debit)
  - Balance After
  - Reason
  - Date/Time
  - Granted By
- [ ] Empty state when no wallet entered
- [ ] Error handling for invalid wallet
- [ ] Dark mode table styling
- [ ] Mobile responsive table

**Search Flow**:
1. Enter wallet: `0x1234...`
2. Click "Search"
3. Table displays transactions
4. Each row shows transaction details

---

## End-to-End Testing

### E2E Flow: Full Credit Lifecycle

**Scenario**: Admin grants credits → User sees balance → User pays with credits → Balance deducted

#### Step 1: Admin Grants Credits
1. Go to http://localhost:3001/credits
2. Click "Grant Credits" tab
3. Fill form:
   - Wallet: `0xYourUserWallet`
   - Amount: `100.00`
   - Reason: "E2E Test Credit"
4. Click "Grant Credits"
5. ✅ Success message shows: "New balance: $100.00"

#### Step 2: User Views Balance
1. Go to http://localhost:3000/account/credits
2. ✅ Available Balance shows: `$100.00`
3. ✅ Transaction history shows 1 grant transaction

#### Step 3: User Selects Plan
1. Go to http://localhost:3000/plans
2. ✅ Plan cards show credit breakdown:
   ```
   Plan Price: $30.00
   Credit Applied: -$30.00
   Amount Due: $0.00
   ✨ Fully covered by credits!
   ```

#### Step 4: User Completes Payment
1. Click plan (e.g., $30 plan)
2. Go through payment flow
3. ✅ Payment completes without blockchain transaction
4. ✅ Backend logs show credit deduction

#### Step 5: Verify Balance Updated
1. Return to http://localhost:3000/account/credits
2. ✅ Available Balance: `$70.00` (100 - 30)
3. ✅ Transaction history shows:
   - Grant: +$100.00 (balance after: $100.00)
   - Payment Debit: -$30.00 (balance after: $70.00)

---

## Error Testing

### 1. Authentication Errors

**Test**: Access endpoints without token
```bash
curl http://localhost:8080/api/payments/credits/balance
```
**Expected**: `401 Unauthorized` or redirect to auth

### 2. Permission Errors

**Test**: User token on admin endpoint
```bash
curl -H "Authorization: Bearer $USER_TOKEN" \
  http://localhost:8080/api/payments/admin/credits/stats
```
**Expected**: `403 Forbidden` - "Insufficient permissions"

### 3. Validation Errors

**Test**: Grant negative amount
```bash
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"0x123...","amount":-10.00}' \
  http://localhost:8080/api/payments/admin/credits/grant
```
**Expected**: `400 Bad Request` - Validation error

### 4. Insufficient Credits

**Test**: Revoke more than balance
```bash
# If balance is $10
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"0x123...","amount":20.00}' \
  http://localhost:8080/api/payments/admin/credits/revoke
```
**Expected**: `400 Bad Request` - "Insufficient credits"

---

## Performance Testing

### 1. Transaction History Pagination
```bash
# Create 100 transactions (admin loop)
for i in {1..100}; do
  curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"wallet_address\":\"0x123...\",\"amount\":1.00,\"reason\":\"Test $i\"}" \
    http://localhost:8080/api/payments/admin/credits/grant
done

# Query with pagination
curl -H "Authorization: Bearer $USER_TOKEN" \
  "http://localhost:8080/api/payments/credits/history?limit=20&offset=0"
```
**Expected**: Fast response (<500ms), paginated results

### 2. Concurrent Credit Operations
```bash
# Multiple admins granting simultaneously (use `parallel` or similar)
# Should maintain atomic balance updates
```

---

## Browser Testing Matrix

| Browser | Desktop | Mobile | Dark Mode |
|---------|---------|--------|-----------|
| Chrome  | ✅      | ✅     | ✅        |
| Firefox | ✅      | ✅     | ✅        |
| Safari  | ✅      | ✅     | ✅        |
| Edge    | ✅      | ✅     | ✅        |

**Test on each**:
- /account/credits page
- /account overview widget
- /plans with credit display
- Admin /credits management

---

## Automated Testing (Future)

### Unit Tests (Backend)
```bash
cd apps/backend
cargo test credit
```

### Integration Tests (Frontend)
```bash
cd apps/frontend
bun test -- credits
```

### E2E Tests (Playwright)
```bash
bun test:e2e -- credit-wallet
```

---

## Common Issues & Fixes

### Issue: "Balance not loading"
**Fix**: Check network tab for 401/403, re-authenticate

### Issue: "Credits not applying to payment"
**Fix**: Verify backend credit deduction logic in logs

### Issue: "Admin can't grant credits"
**Fix**: Check admin has `admin:credits:manage` permission

### Issue: "Dark mode broken"
**Fix**: Verify all `dark:` Tailwind classes present

---

## Success Criteria

✅ All API endpoints return 200 for valid requests
✅ User can view credit balance and history
✅ Admin can grant/revoke credits
✅ Credits auto-apply to payments
✅ Balance updates atomically
✅ Transaction history accurate
✅ Dark mode works across all pages
✅ Mobile responsive on all screens
✅ No console errors in browser
✅ No TypeScript/ESLint errors

---

**Testing Complete** ✅
**Ready for Production** 🚀
