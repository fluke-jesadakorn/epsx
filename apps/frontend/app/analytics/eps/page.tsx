'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { EPSAnalysisForm } from '@/components/analytics/eps/EPSAnalysisForm';
import { EPSGrowthChart } from '@/components/analytics/eps/EPSGrowthChart';
import { PatternRecognition } from '@/components/analytics/eps/PatternRecognition';
import { AnalysisResults } from '@/components/analytics/eps/AnalysisResults';
import { HistoricalComparison } from '@/components/analytics/eps/HistoricalComparison';
import { EducationalContext } from '@/components/analytics/eps/EducationalContext';
import { ComparisonTable } from '@/components/analytics/eps/ComparisonTable';
import { ExportReport } from '@/components/analytics/eps/ExportReport';
import { useEPSAnalytics } from '@/hooks/useFirebaseAnalytics';
import { useAuth } from '@/auth/ctx';

interface EPSAnalysisData {
  symbol: string;
  currentEPS: number;
  previousEPS: number;
  growth: number;
  patterns: any[];
  historicalData: any[];
  confidence: number;
  analysis: string;
}

export default function EPSAnalysisPage() {
  const [selectedSymbol, setSelectedSymbol] = useState<string>('');
  const [analysisData, setAnalysisData] = useState<EPSAnalysisData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [comparisonSymbols, setComparisonSymbols] = useState<string[]>([]);
  
  const { user } = useAuth();
  const { trackEPSFormSubmit, trackEPSChartView, trackEPSExport } = useEPSAnalytics(user?.id);

  const handleAnalysisSubmit = async (symbol: string, parameters: any) => {
    setIsLoading(true);
    setSelectedSymbol(symbol);
    trackEPSFormSubmit(symbol);

    try {
      // Simulate API call - replace with actual backend integration
      const response = await fetch(`/api/v1/analytics/eps/${symbol}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(parameters)
      });
      
      if (response.ok) {
        const data = await response.json();
        setAnalysisData(data);
      }
    } catch (error) {
      console.error('EPS Analysis error:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleChartView = (chartType: string) => {
    if (selectedSymbol) {
      trackEPSChartView(selectedSymbol, chartType);
    }
  };

  const handleExport = (format: string) => {
    if (selectedSymbol) {
      trackEPSExport(selectedSymbol, format);
    }
  };

  const handleAddComparison = (symbol: string) => {
    if (!comparisonSymbols.includes(symbol)) {
      setComparisonSymbols([...comparisonSymbols, symbol]);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8 space-y-6">
      {/* Page Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">EPS Analysis</h1>
        <p className="text-muted-foreground">
          Analyze earnings per share growth patterns and trends with AI-powered insights.
        </p>
      </div>

      {/* Educational Context */}
      <EducationalContext />

      {/* Analysis Form */}
      <Card>
        <CardHeader>
          <CardTitle>EPS Analysis Configuration</CardTitle>
          <CardDescription>
            Enter a stock symbol and configure analysis parameters to generate insights.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <EPSAnalysisForm 
            onSubmit={handleAnalysisSubmit}
            isLoading={isLoading}
          />
        </CardContent>
      </Card>

      {/* Analysis Results */}
      {analysisData && (
        <>
          {/* Growth Chart and Pattern Recognition */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>EPS Growth Visualization</CardTitle>
              </CardHeader>
              <CardContent>
                <EPSGrowthChart 
                  data={analysisData.historicalData}
                  symbol={selectedSymbol}
                  onChartView={handleChartView}
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Pattern Recognition</CardTitle>
              </CardHeader>
              <CardContent>
                <PatternRecognition 
                  patterns={analysisData.patterns}
                  confidence={analysisData.confidence}
                  symbol={selectedSymbol}
                />
              </CardContent>
            </Card>
          </div>

          {/* Detailed Analysis Results */}
          <Card>
            <CardHeader>
              <CardTitle>Analysis Results</CardTitle>
            </CardHeader>
            <CardContent>
              <AnalysisResults 
                data={analysisData}
                symbol={selectedSymbol}
              />
            </CardContent>
          </Card>

          {/* Historical Comparison */}
          <Card>
            <CardHeader>
              <CardTitle>Historical Comparison</CardTitle>
            </CardHeader>
            <CardContent>
              <HistoricalComparison 
                data={analysisData.historicalData}
                symbol={selectedSymbol}
                onChartView={handleChartView}
              />
            </CardContent>
          </Card>

          {/* Multi-Company Comparison */}
          {comparisonSymbols.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Multi-Company EPS Comparison</CardTitle>
              </CardHeader>
              <CardContent>
                <ComparisonTable 
                  primarySymbol={selectedSymbol}
                  comparisonSymbols={comparisonSymbols}
                  onRemoveComparison={(symbol) => 
                    setComparisonSymbols(comparisonSymbols.filter(s => s !== symbol))
                  }
                />
              </CardContent>
            </Card>
          )}

          {/* Export Options */}
          <Card>
            <CardHeader>
              <CardTitle>Export Analysis</CardTitle>
            </CardHeader>
            <CardContent>
              <ExportReport 
                data={analysisData}
                symbol={selectedSymbol}
                onExport={handleExport}
                onAddComparison={handleAddComparison}
              />
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}