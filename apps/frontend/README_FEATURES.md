# Token-Based Feature Access System

## Overview
This implementation provides a comprehensive token-gated feature access system that integrates with the EPSx token system to manage user access to features based on their role and token holdings.

## Key Components

### Backend Components
- `UserRole` enum: Defines user roles (GUEST, REGISTERED_USER, PREMIUM_USER, TOKEN_HOLDER, ADMINISTRATOR)
- `TokenFeature` enum: Defines token-gated features
- `TokenService`: Manages token generation and verification with enhanced claims
- Custom guards for role and feature access
- Decorators for easy route protection

### Frontend Components
- `useFeatureAccess` hook: Manages feature access checks
- `TokenGatedFeature` component: UI wrapper for token-gated features
- Enhanced AuthContext with token features
- Example Trading Dashboard implementation

## Testing the Implementation

### 1. Role-Based Access
```typescript
// Test different roles
const testRoles = async () => {
  // Guest access
  await testAccess(UserRole.GUEST);
  // Should only see public features

  // Registered user access
  await testAccess(UserRole.REGISTERED_USER);
  // Should see basic features

  // Premium user access
  await testAccess(UserRole.PREMIUM_USER);
  // Should see premium features
};
```

### 2. Token-Based Access
Test different token balances to verify feature access:
```typescript
const testTokenAccess = async () => {
  // No tokens
  await updateTokenBalance(0);
  // Should see no token-gated features

  // Basic token holder
  await updateTokenBalance(1000);
  // Should see basic token-gated features

  // Premium token holder
  await updateTokenBalance(10000);
  // Should see all token-gated features
};
```

### 3. Feature Component Testing
```typescript
// Test TokenGatedFeature component
<TokenGatedFeature
  feature={TokenFeature.TRADING_BOT}
  fallback={<div>Upgrade required</div>}
>
  <TradingBot />
</TokenGatedFeature>
```

### 4. Permission Checks
```typescript
// Test permission-based access
const { hasPermission } = useAuth();
if (hasPermission(PERMISSIONS.USE_TRADING_BOT)) {
  // Show trading bot interface
}
```

## Example Usage

### Protecting Routes
```typescript
// In your page component
export default function ProtectedPage() {
  return (
    <TokenGatedFeature feature={TokenFeature.PORTFOLIO_MANAGEMENT}>
      <PortfolioContent />
    </TokenGatedFeature>
  );
}
```

### Using the Hook
```typescript
function TradingComponent() {
  const { checkFeatureAccess } = useFeatureAccess();
  const access = checkFeatureAccess(TokenFeature.TRADING_BOT);

  if (!access.hasAccess) {
    return (
      <div>
        Required tokens: {access.requiredTokens}
        Current balance: {access.currentTokens}
      </div>
    );
  }

  return <TradingInterface />;
}
```

### Backend Route Protection
```typescript
@Controller('trading')
export class TradingController {
  @RequireFeatures([TokenFeature.TRADING_BOT])
  @Post('bot/execute')
  async executeTrade() {
    // Protected endpoint
  }
}
```

## Feature Requirements

| Feature | Min Tokens | Required Role |
|---------|------------|---------------|
| Real-time Analysis | 100 | REGISTERED_USER |
| Portfolio Management | 500 | REGISTERED_USER |
| Portfolio Assistance | 1,000 | PREMIUM_USER |
| Trading Bot | 2,500 | PREMIUM_USER |
| AI Analysis | 5,000 | PREMIUM_USER |
| Advanced Tools | 7,500 | PREMIUM_USER |
| Governance | 10,000 | TOKEN_HOLDER |

## Best Practices
1. Always use the `TokenGatedFeature` component for consistent UX
2. Implement both frontend and backend checks
3. Use the `useFeatureAccess` hook for programmatic checks
4. Keep token requirements in sync between frontend and backend
5. Provide clear upgrade paths for users
6. Handle token balance changes reactively

## Known Limitations
1. Token balance updates require page refresh or polling
2. Feature access is checked on each render
3. Backend token verification adds slight latency
