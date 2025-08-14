'use client';

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { BookOpen, AlertTriangle, Info, ChevronDown, ChevronUp } from 'lucide-react';
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '@epsx/ui';

export function EducationalContext() {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <Card className="bg-gradient-to-r from-blue-50 to-indigo-50 border-blue-200">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <BookOpen className="h-5 w-5 text-blue-600" />
          Educational Context: EPS Analysis
          <Badge variant="outline" className="bg-blue-100 text-blue-700">
            Learning Tool
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Basic Information */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-2">
            <h4 className="font-medium text-blue-900">What is EPS?</h4>
            <p className="text-sm text-blue-700">
              Earnings Per Share (EPS) measures a company&apos;s profitability by dividing net income by outstanding shares. 
              It&apos;s a key metric for evaluating financial performance and comparing companies.
            </p>
          </div>
          <div className="space-y-2">
            <h4 className="font-medium text-blue-900">Why Analyze EPS Growth?</h4>
            <p className="text-sm text-blue-700">
              EPS growth patterns can indicate business trends, management effectiveness, and potential investment opportunities. 
              Consistent growth often signals a healthy, expanding business.
            </p>
          </div>
        </div>

        {/* Expandable Detailed Information */}
        <Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
          <CollapsibleTrigger asChild>
            <Button variant="outline" className="w-full">
              <span>Learn More About EPS Analysis</span>
              {isExpanded ? <ChevronUp className="h-4 w-4 ml-2" /> : <ChevronDown className="h-4 w-4 ml-2" />}
            </Button>
          </CollapsibleTrigger>
          <CollapsibleContent className="space-y-4 mt-4">
            {/* Key Concepts */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <Card className="bg-white/50">
                <CardContent className="p-4">
                  <h5 className="font-medium text-blue-900 mb-2">Types of EPS</h5>
                  <ul className="text-sm text-blue-700 space-y-1">
                    <li>• Basic EPS: Net income ÷ Outstanding shares</li>
                    <li>• Diluted EPS: Includes potential share dilution</li>
                    <li>• Adjusted EPS: Excludes one-time items</li>
                  </ul>
                </CardContent>
              </Card>

              <Card className="bg-white/50">
                <CardContent className="p-4">
                  <h5 className="font-medium text-blue-900 mb-2">Growth Analysis</h5>
                  <ul className="text-sm text-blue-700 space-y-1">
                    <li>• Quarter-over-Quarter (QoQ)</li>
                    <li>• Year-over-Year (YoY)</li>
                    <li>• Long-term trend analysis</li>
                  </ul>
                </CardContent>
              </Card>

              <Card className="bg-white/50">
                <CardContent className="p-4">
                  <h5 className="font-medium text-blue-900 mb-2">Pattern Recognition</h5>
                  <ul className="text-sm text-blue-700 space-y-1">
                    <li>• Seasonal patterns</li>
                    <li>• Growth acceleration/deceleration</li>
                    <li>• Volatility compression</li>
                  </ul>
                </CardContent>
              </Card>
            </div>

            {/* Important Considerations */}
            <div className="space-y-3">
              <div className="flex items-start gap-3 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                <AlertTriangle className="h-4 w-4 text-yellow-600 mt-0.5" />
                <div>
                  <div className="font-medium text-yellow-900">Important Considerations</div>
                  <ul className="text-sm text-yellow-700 mt-1 space-y-1">
                    <li>• EPS can be influenced by share buybacks without operational improvement</li>
                    <li>• One-time charges or gains can distort quarterly comparisons</li>
                    <li>• Industry context is crucial for meaningful interpretation</li>
                    <li>• Consider cash flow and revenue growth alongside EPS</li>
                  </ul>
                </div>
              </div>

              <div className="flex items-start gap-3 p-3 bg-blue-50 border border-blue-200 rounded-lg">
                <Info className="h-4 w-4 text-blue-600 mt-0.5" />
                <div>
                  <div className="font-medium text-blue-900">Analysis Methodology</div>
                  <p className="text-sm text-blue-700 mt-1">
                    Our AI-powered analysis examines historical EPS data, identifies patterns, and provides confidence scores 
                    based on statistical models and industry benchmarks. All insights are generated for educational purposes 
                    and should not be considered as investment advice.
                  </p>
                </div>
              </div>
            </div>

            {/* Educational Resources */}
            <div className="space-y-2">
              <h5 className="font-medium text-blue-900">Learn More</h5>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                <Button variant="outline" size="sm" className="text-xs">
                  EPS Basics
                </Button>
                <Button variant="outline" size="sm" className="text-xs">
                  Financial Ratios
                </Button>
                <Button variant="outline" size="sm" className="text-xs">
                  Pattern Analysis
                </Button>
                <Button variant="outline" size="sm" className="text-xs">
                  Investment Fundamentals
                </Button>
              </div>
            </div>
          </CollapsibleContent>
        </Collapsible>

        {/* Legal Disclaimer */}
        <div className="text-xs text-blue-600 bg-blue-100 p-3 rounded-lg border border-blue-300">
          <strong>Educational Disclaimer:</strong> This analysis tool is designed for educational purposes only. 
          It does not constitute investment advice, financial advice, trading advice, or any other type of advice. 
          You should not make any investment decisions based solely on this information. 
          Always conduct your own research and consult with qualified financial professionals before making investment decisions.
        </div>
      </CardContent>
    </Card>
  );
}