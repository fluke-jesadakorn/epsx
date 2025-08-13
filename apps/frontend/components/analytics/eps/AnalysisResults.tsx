'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import { 
  TrendingUp, 
  TrendingDown, 
  DollarSign, 
  Calendar, 
  Target,
  AlertCircle,
  CheckCircle,
  Info
} from 'lucide-react';

interface AnalysisResultsProps {
  data: {
    symbol: string;
    currentEPS: number;
    previousEPS: number;
    growth: number;
    confidence: number;
    analysis: string;
  };
  symbol: string;
}

interface MetricCard {
  label: string;
  value: string;
  change?: number;
  icon: React.ReactNode;
  color: string;
}

export function AnalysisResults({ data, symbol }: AnalysisResultsProps) {
  // Calculate derived metrics
  const quarterOverQuarter = ((data.currentEPS - data.previousEPS) / data.previousEPS) * 100;
  const yearOverYear = data.growth;
  
  // Mock additional data for comprehensive analysis
  const analysisMetrics = {
    epsGuidance: 2.65,
    analystConsensus: 2.58,
    beatRate: 73,
    surprisePercentage: 8.2,
    revision: 'upward',
    qualityScore: 87
  };

  const metrics: MetricCard[] = [
    {
      label: 'Current EPS',
      value: `$${data.currentEPS.toFixed(2)}`,
      change: quarterOverQuarter,
      icon: <DollarSign className="h-4 w-4" />,
      color: 'text-blue-600'
    },
    {
      label: 'QoQ Growth',
      value: `${quarterOverQuarter.toFixed(1)}%`,
      change: quarterOverQuarter,
      icon: quarterOverQuarter >= 0 ? <TrendingUp className="h-4 w-4" /> : <TrendingDown className="h-4 w-4" />,
      color: quarterOverQuarter >= 0 ? 'text-green-600' : 'text-red-600'
    },
    {
      label: 'YoY Growth',
      value: `${yearOverYear.toFixed(1)}%`,
      change: yearOverYear,
      icon: yearOverYear >= 0 ? <TrendingUp className="h-4 w-4" /> : <TrendingDown className="h-4 w-4" />,
      color: yearOverYear >= 0 ? 'text-green-600' : 'text-red-600'
    },
    {
      label: 'Analysis Confidence',
      value: `${data.confidence}%`,
      icon: <Target className="h-4 w-4" />,
      color: data.confidence >= 80 ? 'text-green-600' : data.confidence >= 60 ? 'text-yellow-600' : 'text-red-600'
    }
  ];

  const getConfidenceLevel = (confidence: number) => {
    if (confidence >= 85) return { level: 'High', color: 'text-green-600', bg: 'bg-green-50' };
    if (confidence >= 70) return { level: 'Medium', color: 'text-yellow-600', bg: 'bg-yellow-50' };
    return { level: 'Low', color: 'text-red-600', bg: 'bg-red-50' };
  };

  const confidenceInfo = getConfidenceLevel(data.confidence);

  return (
    <div className="space-y-6">
      {/* Key Metrics Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {metrics.map((metric, index) => (
          <Card key={index}>
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <div className={`p-1 rounded ${metric.color}`}>
                  {metric.icon}
                </div>
                {metric.change !== undefined && (
                  <Badge variant={metric.change >= 0 ? 'default' : 'destructive'} className="text-xs">
                    {metric.change >= 0 ? '+' : ''}{metric.change.toFixed(1)}%
                  </Badge>
                )}
              </div>
              <div className="text-xs text-muted-foreground">{metric.label}</div>
              <div className={`text-lg font-bold ${metric.color}`}>{metric.value}</div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Analysis Summary */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Info className="h-5 w-5" />
            Analysis Summary
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className={`p-4 rounded-lg ${confidenceInfo.bg} border`}>
            <div className="flex items-center gap-2 mb-2">
              <Badge className={confidenceInfo.color}>{confidenceInfo.level} Confidence</Badge>
              <span className="text-sm text-muted-foreground">
                Analysis Quality Score: {analysisMetrics.qualityScore}/100
              </span>
            </div>
            <Progress value={data.confidence} className="mb-2" />
            <p className="text-sm">
              {data.analysis || `Analysis of ${symbol} shows ${quarterOverQuarter >= 0 ? 'positive' : 'negative'} quarter-over-quarter EPS growth of ${quarterOverQuarter.toFixed(1)}%. The year-over-year growth of ${yearOverYear.toFixed(1)}% indicates ${yearOverYear >= 15 ? 'strong' : yearOverYear >= 5 ? 'moderate' : 'weak'} earnings momentum.`}
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Detailed Analysis */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Performance vs Expectations */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Target className="h-5 w-5" />
              Performance vs Expectations
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm">Actual EPS</span>
                <span className="font-medium">${data.currentEPS.toFixed(2)}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Analyst Consensus</span>
                <span className="font-medium">${analysisMetrics.analystConsensus.toFixed(2)}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Company Guidance</span>
                <span className="font-medium">${analysisMetrics.epsGuidance.toFixed(2)}</span>
              </div>
              <Separator />
              <div className="flex justify-between items-center">
                <span className="text-sm">Beat/Miss</span>
                <div className="flex items-center gap-2">
                  {data.currentEPS > analysisMetrics.analystConsensus ? (
                    <CheckCircle className="h-4 w-4 text-green-600" />
                  ) : (
                    <AlertCircle className="h-4 w-4 text-red-600" />
                  )}
                  <span className={`font-medium ${
                    data.currentEPS > analysisMetrics.analystConsensus ? 'text-green-600' : 'text-red-600'
                  }`}>
                    {data.currentEPS > analysisMetrics.analystConsensus ? 'Beat' : 'Miss'} by ${analysisMetrics.surprisePercentage}%
                  </span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Historical Context */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Calendar className="h-5 w-5" />
              Historical Context
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm">Historical Beat Rate</span>
                <span className="font-medium">{analysisMetrics.beatRate}%</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Estimate Revisions</span>
                <Badge variant={analysisMetrics.revision === 'upward' ? 'default' : 'destructive'}>
                  {analysisMetrics.revision} trend
                </Badge>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">Growth Consistency</span>
                <div className="flex items-center gap-2">
                  <Progress value={75} className="w-16 h-2" />
                  <span className="text-sm">75%</span>
                </div>
              </div>
              <Separator />
              <div className="text-xs text-muted-foreground">
                Based on last 12 quarters of earnings data and analyst revisions.
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Key Insights */}
      <Card>
        <CardHeader>
          <CardTitle>Key Insights</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-start gap-3 p-3 bg-blue-50 rounded-lg">
              <Info className="h-4 w-4 text-blue-600 mt-0.5" />
              <div>
                <div className="font-medium text-blue-900">Growth Momentum</div>
                <div className="text-sm text-blue-700">
                  {quarterOverQuarter >= 0 ? 'Positive' : 'Negative'} quarter-over-quarter growth indicates 
                  {quarterOverQuarter >= 0 ? ' continued business expansion' : ' potential challenges in the business'}.
                </div>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 bg-green-50 rounded-lg">
              <CheckCircle className="h-4 w-4 text-green-600 mt-0.5" />
              <div>
                <div className="font-medium text-green-900">Analyst Sentiment</div>
                <div className="text-sm text-green-700">
                  {analysisMetrics.revision === 'upward' ? 'Upward' : 'Downward'} revision trend suggests 
                  {analysisMetrics.revision === 'upward' ? ' improving' : ' deteriorating'} analyst confidence.
                </div>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 bg-yellow-50 rounded-lg">
              <AlertCircle className="h-4 w-4 text-yellow-600 mt-0.5" />
              <div>
                <div className="font-medium text-yellow-900">Risk Assessment</div>
                <div className="text-sm text-yellow-700">
                  Quality score of {analysisMetrics.qualityScore}/100 indicates 
                  {analysisMetrics.qualityScore >= 80 ? ' high-quality' : ' moderate-quality'} earnings with 
                  {analysisMetrics.qualityScore >= 80 ? ' low' : ' moderate'} accounting risk.
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}