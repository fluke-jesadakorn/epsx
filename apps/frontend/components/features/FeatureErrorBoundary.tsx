"use client";

import React, { Component } from 'react';
import type { ErrorInfo, ReactNode } from 'react';
import { TokenFeature } from '@/types/auth/features';

interface Props {
  children: ReactNode;
  feature: TokenFeature;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class FeatureErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false
  };

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error
    };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Feature Error:', error);
    console.error('Error Info:', errorInfo);
  }

  private getErrorMessage(): string {
    if (this.state.error?.message.includes('token balance')) {
      return 'Insufficient token balance to access this feature.';
    }
    if (this.state.error?.message.includes('permission')) {
      return 'You do not have permission to access this feature.';
    }
    if (this.state.error?.message.includes('role')) {
      return 'Your current role cannot access this feature.';
    }
    return 'An error occurred while loading this feature.';
  }

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="p-4 border rounded-lg bg-red-50 dark:bg-red-900/20">
          <h3 className="text-lg font-semibold mb-2 text-red-700 dark:text-red-400">
            Feature Error
          </h3>
          <p className="text-red-600 dark:text-red-300 mb-4">
            {this.getErrorMessage()}
          </p>
          <div className="flex gap-2">
            <button
              onClick={() => this.setState({ hasError: false })}
              className="px-4 py-2 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300 rounded hover:bg-red-200 dark:hover:bg-red-800 transition-colors"
            >
              Try Again
            </button>
            <a
              href="/upgrade"
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              Upgrade Access
            </a>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

// HOC to wrap components with FeatureErrorBoundary
export function withFeatureErrorBoundary<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  feature: TokenFeature,
  fallback?: ReactNode
) {
  return function WithFeatureErrorBoundary(props: P) {
    return (
      <FeatureErrorBoundary feature={feature} fallback={fallback}>
        <WrappedComponent {...props} />
      </FeatureErrorBoundary>
    );
  };
}

// Example usage:
// const SafeTradingBot = withFeatureErrorBoundary(TradingBot, TokenFeature.TRADING_BOT);
