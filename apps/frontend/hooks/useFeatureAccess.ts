import { useState, useEffect } from 'react';
import { useAuth } from '@/context/auth-context';
import { usePermissionContext } from '@epsx/server-providers';
import { TokenFeature, FeatureAccess, Permission } from '@/types/auth/features';
import { UserRole } from '@/types/auth/roles';

export function useFeatureAccess() {
  const { user } = useAuth();
  const { featureAccess: serverFeatureAccess, rankingAccess: serverRankingAccess, error } = usePermissionContext();
  const [loading, setLoading] = useState(true);
  const [featureAccess, setFeatureAccess] = useState({
    canAccessTrading: false,
    canAccessRankings: false,
    canAccessAnalytics: false,
    userTier: 'BRONZE'
  });
  
  useEffect(() => {
    if (!user) {
      setFeatureAccess({
        canAccessTrading: false,
        canAccessRankings: false,
        canAccessAnalytics: false,
        userTier: 'BRONZE'
      });
      setLoading(false);
      return;
    }

    if (error) {
      console.error('Feature access context error:', error);
      setFeatureAccess({
        canAccessTrading: !!user,
        canAccessRankings: !!user,
        canAccessAnalytics: !!user,
        userTier: 'BRONZE'
      });
      setLoading(false);
      return;
    }

    if (serverFeatureAccess || serverRankingAccess) {
      setFeatureAccess({
        canAccessTrading: serverFeatureAccess?.trading?.allowed || false,
        canAccessRankings: serverRankingAccess?.allowed || false,
        canAccessAnalytics: serverFeatureAccess?.analytics?.allowed || false,
        userTier: serverRankingAccess?.tier || 'BRONZE'
      });
      setLoading(false);
    }
  }, [user, serverFeatureAccess, serverRankingAccess, error]);

  // Function to check feature access based on TokenFeature enum
  const checkFeatureAccess = (feature: TokenFeature): FeatureAccess => {
    if (!user) {
      return {
        hasAccess: false,
        currentTokens: 0,
        requiredTokens: 100,
        currentRole: UserRole.USER,
        requiredRole: UserRole.USER,
        missingPermissions: []
      };
    }

    const userRole = user.role === 'ADMIN' ? UserRole.ADMIN : UserRole.USER;
    const tokenBalance = 0; // Backend doesn't provide token balance, using 0 for now

    // Map TokenFeature to server feature access
    let hasAccess = false;
    let requiredTokens = 0;
    let requiredRole = UserRole.USER;
    let missingPermissions: Permission[] = [];

    switch (feature) {
      case TokenFeature.TRADING:
        hasAccess = featureAccess.canAccessTrading;
        requiredTokens = 50;
        requiredRole = UserRole.USER;
        if (!hasAccess) {
          missingPermissions = [Permission.EXECUTE_TRADES];
        }
        break;
      
      case TokenFeature.REAL_TIME_ANALYSIS:
      case TokenFeature.AI_ANALYSIS:
        hasAccess = featureAccess.canAccessAnalytics;
        requiredTokens = 100;
        requiredRole = UserRole.USER;
        if (!hasAccess) {
          missingPermissions = [Permission.VIEW_ANALYTICS];
        }
        break;
      
      case TokenFeature.ADMIN_ACCESS:
        hasAccess = userRole === UserRole.ADMIN;
        requiredTokens = 0;
        requiredRole = UserRole.ADMIN;
        if (!hasAccess) {
          missingPermissions = [Permission.ADMIN];
        }
        break;
      
      case TokenFeature.TRADING_BOT:
      case TokenFeature.PORTFOLIO_MANAGEMENT:
      case TokenFeature.ADVANCED_TOOLS:
        hasAccess = featureAccess.canAccessRankings; // Use rankings as proxy for premium features
        requiredTokens = 200;
        requiredRole = UserRole.USER;
        if (!hasAccess) {
          missingPermissions = [Permission.ACCESS_API];
        }
        break;
      
      default:
        hasAccess = !!user;
        requiredTokens = 0;
        requiredRole = UserRole.USER;
        break;
    }

    return {
      hasAccess,
      currentTokens: tokenBalance,
      requiredTokens,
      currentRole: userRole,
      requiredRole,
      missingPermissions
    };
  };
  
  return {
    ...featureAccess,
    loading,
    checkFeatureAccess
  };
}
