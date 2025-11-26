'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { CheckCircle } from 'lucide-react';

interface AnalyticsLegendProps {
  onComplete: () => void;
  className?: string;
}

export function AnalyticsLegend({ onComplete, className = '' }: AnalyticsLegendProps) {
  return (
    <div className={`max-w-4xl mx-auto space-y-6 p-6 ${className}`}>
      {/* Header */}
      <Card className="text-center">
        <CardHeader>
          <CardTitle className="text-3xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
            📋 How to Read Performance Cards
          </CardTitle>
          <p className="text-gray-600 dark:text-gray-300 text-lg">
            Quick guide to understand all indicators and make better decisions
          </p>
        </CardHeader>
      </Card>

      {/* System Modes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl font-semibold flex items-center gap-2">
            🎯 System Modes
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-4 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">🟢</span>
                <span className="font-bold text-green-700 dark:text-green-300">ACTIVE</span>
              </div>
              <p className="text-sm text-green-600 dark:text-green-400">
                Keep monitoring - good data pattern detected
              </p>
            </div>
            
            <div className="p-4 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">🔴</span>
                <span className="font-bold text-red-700 dark:text-red-300">INACTIVE</span>
              </div>
              <p className="text-sm text-red-600 dark:text-red-400">
                Stop monitoring - declining performance pattern
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Status Indicators */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl font-semibold flex items-center gap-2">
            📊 Status Indicators
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="p-4 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">⬆️</span>
                <span className="font-bold text-blue-700 dark:text-blue-300">POSITIVE</span>
              </div>
              <p className="text-sm text-blue-600 dark:text-blue-400">
                Both Growth and Price numbers going UP
              </p>
            </div>
            
            <div className="p-4 rounded-lg bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">↕️</span>
                <span className="font-bold text-purple-700 dark:text-purple-300">MIXED</span>
              </div>
              <p className="text-sm text-purple-600 dark:text-purple-400">
                One number UP, one number DOWN
              </p>
            </div>
            
            <div className="p-4 rounded-lg bg-gray-50 dark:bg-gray-900/20 border border-gray-200 dark:border-gray-800">
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">⬇️</span>
                <span className="font-bold text-gray-700 dark:text-gray-300">NEGATIVE</span>
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Both Growth and Price numbers going DOWN
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Data Meaning */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl font-semibold flex items-center gap-2">
            📈 Data Meaning
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-3">
              <div className="flex items-start gap-3">
                <span className="font-semibold text-orange-600 dark:text-orange-400 min-w-16">Growth:</span>
                <span className="text-gray-700 dark:text-gray-300">Company performance change percentage</span>
              </div>
              <div className="flex items-start gap-3">
                <span className="font-semibold text-orange-600 dark:text-orange-400 min-w-16">Price:</span>
                <span className="text-gray-700 dark:text-gray-300">Stock value change percentage</span>
              </div>
            </div>
            
            <div className="space-y-3">
              <div className="flex items-start gap-3">
                <span className="font-semibold text-orange-600 dark:text-orange-400 min-w-20">Pattern:</span>
                <span className="text-gray-700 dark:text-gray-300">How many quarters in a row showing UP</span>
              </div>
              <div className="flex items-start gap-3">
                <span className="font-semibold text-orange-600 dark:text-orange-400 min-w-20">Next Check:</span>
                <span className="text-gray-700 dark:text-gray-300">When new performance data becomes available</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Example Card Preview */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl font-semibold flex items-center gap-2">
            👀 Example Card Preview
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50/50 dark:bg-gray-900/50">
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              Each card shows the most recent 2 quarters of data for easy comparison:
            </p>
            <div className="space-y-2 text-sm font-mono bg-white dark:bg-gray-800 p-3 rounded border">
              <div className="text-green-600">Jul 30 | ₩3,650 | ✅ UP | Growth: +5.49% | Price: +4.94%</div>
              <div className="text-blue-600">Status: POSITIVE | Both numbers UP! ⬆️ | Mode: ACTIVE</div>
              <div className="border-t pt-2 text-purple-600">Apr 30 | €346.00 | ✅ UP | Growth: +7.12% | Price: -0.34%</div>
              <div className="text-blue-600">Status: MIXED | Growth UP, Price DOWN ↕️ | Mode: ACTIVE</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Start Button */}
      <Card className="text-center">
        <CardContent className="pt-6">
          <Button 
            onClick={onComplete}
            size="lg"
            className="bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white font-semibold px-12 py-3 text-lg"
          >
            <CheckCircle className="w-5 h-5 mr-2" />
            ✅ Got It! Start Analyzing
          </Button>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-3">
            You can always access this guide again from the help menu
          </p>
        </CardContent>
      </Card>
    </div>
  );
}