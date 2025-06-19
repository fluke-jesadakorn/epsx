'use client';

import React from 'react';

import { TokenGatedFeature } from '@/components/features/TokenGatedFeature';
import { useFeatureAccess } from '@/hooks/useFeatureAccess';
import { TokenFeature } from '@/types/auth/features';

const AutomationDashboard = (): React.JSX.Element => {
  const { tokenBalance, role } = useFeatureAccess();

  return (
    <div className="container mx-auto p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold mb-2">Trading Dashboard</h1>
        <div className="flex items-center space-x-4">
          <div className="bg-blue-100 dark:bg-blue-900 p-2 rounded">
            <span className="font-medium">Balance: </span>
            <span>{tokenBalance} EPSx Tokens</span>
          </div>
          <div className="bg-green-100 dark:bg-green-900 p-2 rounded">
            <span className="font-medium">Role: </span>
            <span>{role}</span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Real-time Analysis Feature */}
        <TokenGatedFeature feature={TokenFeature.REAL_TIME_ANALYSIS}>
          <div className="p-4 border rounded-lg">
            <h2 className="text-xl font-semibold mb-4">Real-time Analysis</h2>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Market Trend:</span>
                <span className="text-green-600">Bullish</span>
              </div>
              <div className="flex justify-between">
                <span>Volume:</span>
                <span>1.2M</span>
              </div>
            </div>
          </div>
        </TokenGatedFeature>

        {/* Trading Bot Feature */}
        <TokenGatedFeature
          feature={TokenFeature.TRADING_BOT}
          fallback={
            <div className="p-4 border rounded-lg bg-gray-50">
              <h2 className="text-xl font-semibold mb-2">Trading Bot</h2>
              <p className="text-gray-600">
                Upgrade to access automated trading features
              </p>
            </div>
          }
        >
          <div className="p-4 border rounded-lg">
            <h2 className="text-xl font-semibold mb-4">Trading Bot</h2>
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span>Bot Status:</span>
                <span className="text-green-600">Active</span>
              </div>
              <button className="w-full bg-blue-600 text-white py-2 rounded">
                Configure Bot
              </button>
            </div>
          </div>
        </TokenGatedFeature>

        {/* AI Analysis Feature */}
        <TokenGatedFeature feature={TokenFeature.AI_ANALYSIS}>
          <div className="p-4 border rounded-lg">
            <h2 className="text-xl font-semibold mb-4">AI Analysis</h2>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>AI Prediction:</span>
                <span className="text-green-600">Strong Buy</span>
              </div>
              <div className="flex justify-between">
                <span>Confidence:</span>
                <span>87%</span>
              </div>
            </div>
          </div>
        </TokenGatedFeature>

        {/* Portfolio Management Feature */}
        <TokenGatedFeature feature={TokenFeature.PORTFOLIO_MANAGEMENT}>
          <div className="p-4 border rounded-lg">
            <h2 className="text-xl font-semibold mb-4">Portfolio Management</h2>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Total Value:</span>
                <span>$125,430</span>
              </div>
              <div className="flex justify-between">
                <span>24h Change:</span>
                <span className="text-green-600">+2.5%</span>
              </div>
            </div>
          </div>
        </TokenGatedFeature>

        {/* Advanced Tools Feature */}
        <TokenGatedFeature
          feature={TokenFeature.ADVANCED_TOOLS}
          fallback={
            <div className="col-span-2 p-4 border rounded-lg bg-gray-50">
              <h2 className="text-xl font-semibold mb-2">Advanced Tools</h2>
              <p className="text-gray-600">
                Hold more EPSx tokens to access advanced trading tools
              </p>
            </div>
          }
        >
          <div className="col-span-2 p-4 border rounded-lg">
            <h2 className="text-xl font-semibold mb-4">Advanced Tools</h2>
            <div className="grid grid-cols-2 gap-4">
              <div className="p-3 border rounded">
                <h3 className="font-medium mb-2">Risk Analysis</h3>
                <div className="text-sm text-gray-600">
                  Portfolio Risk Level: Low
                </div>
              </div>
              <div className="p-3 border rounded">
                <h3 className="font-medium mb-2">Trend Detection</h3>
                <div className="text-sm text-gray-600">
                  3 new trends detected
                </div>
              </div>
            </div>
          </div>
        </TokenGatedFeature>
      </div>
    </div>
  );
};

export default AutomationDashboard;
