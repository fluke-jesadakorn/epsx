"use client";

import React from "react";

import { useFeatureAccess } from "@/hooks/useFeatureAccess";
import { UserRole } from "@/types/auth/roles";

import { FeatureErrorBoundary } from "./FeatureErrorBoundary";

import type { TokenFeature } from "@/types/auth/features";

interface TokenGatedFeatureProps {
  feature: TokenFeature;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

interface UpgradeCardProps {
  feature: TokenFeature;
  currentTokens: number;
  requiredTokens: number;
  currentRole: UserRole;
  requiredRole: UserRole;
}

const UpgradeCard: React.FC<UpgradeCardProps> = ({
  currentTokens,
  requiredTokens,
  currentRole,
  requiredRole,
}) => {
  return (
    <div className="p-4 border rounded-lg bg-gray-50 dark:bg-gray-800">
      <h3 className="text-lg font-semibold mb-2">Feature Locked</h3>
      <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
        You need to meet the following requirements to access this feature:
      </p>
      <div className="space-y-2">
        {currentRole !== requiredRole && (
          <div className="flex justify-between items-center">
            <span>Required Role:</span>
            <span className="font-medium">{requiredRole}</span>
          </div>
        )}
        {currentTokens < requiredTokens && (
          <div>
            <div className="flex justify-between items-center">
              <span>Required Tokens:</span>
              <span className="font-medium">{requiredTokens}</span>
            </div>
            <div className="mt-2">
              <div className="w-full bg-gray-200 rounded-full h-2.5">
                <div
                  className="bg-blue-600 h-2.5 rounded-full"
                  style={{
                    width: `${Math.min(100, (currentTokens / requiredTokens) * 100)}%`,
                  }}
                ></div>
              </div>
              <div className="flex justify-between text-sm mt-1">
                <span>{currentTokens} tokens</span>
                <span>{requiredTokens} needed</span>
              </div>
            </div>
          </div>
        )}
        <div className="mt-4">
          <button
            className="w-full bg-blue-600 text-white rounded-lg px-4 py-2 hover:bg-blue-700 transition-colors"
            onClick={() => (window.location.href = "/upgrade")}
          >
            Upgrade Now
          </button>
        </div>
      </div>
    </div>
  );
};

export const TokenGatedFeature: React.FC<TokenGatedFeatureProps> = ({
  feature,
  children,
  fallback,
}) => {
  return (
    <FeatureErrorBoundary feature={feature} fallback={fallback}>
      <TokenGatedContent feature={feature} fallback={fallback}>
        {children}
      </TokenGatedContent>
    </FeatureErrorBoundary>
  );
};

const TokenGatedContent: React.FC<TokenGatedFeatureProps> = ({
  feature,
  children,
  fallback,
}) => {
  const { checkFeatureAccess } = useFeatureAccess();
  const access = checkFeatureAccess(feature);

  if (access.hasAccess) {
    return <>{children}</>;
  }

  if (fallback) {
    return <>{fallback}</>;
  }

  return (
    <UpgradeCard
      feature={feature}
      currentTokens={access.currentTokens}
      requiredTokens={access.requiredTokens || 0}
      currentRole={access.currentRole}
      requiredRole={access.requiredRole || UserRole.USER}
    />
  );
};

// Higher-order component with error boundary support
export function withTokenGate<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  feature: TokenFeature,
  fallback?: React.ReactNode
) {
  return function WithTokenGate(props: P) {
    return (
      <TokenGatedFeature feature={feature} fallback={fallback}>
        <WrappedComponent {...props} />
      </TokenGatedFeature>
    );
  };
}

// Example usage with error handling:
// const TradingBot = () => <div>Trading Bot Component</div>;
// const SafeTradingBot = withTokenGate(
//   TradingBot,
//   TokenFeature.TRADING_BOT,
//   <div>Feature unavailable</div>
// );
