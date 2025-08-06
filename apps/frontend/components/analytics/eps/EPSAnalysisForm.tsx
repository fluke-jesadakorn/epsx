'use client';

import { useState } from 'react';
// Button import removed - not used in current implementation
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { LoadingButton } from '@/components/ui/loading-button';
import { TrendingUp, Calendar, Target, Zap } from 'lucide-react';

interface EPSAnalysisFormProps {
  onSubmit: (symbol: string, parameters: AnalysisParameters) => void;
  isLoading: boolean;
}

interface AnalysisParameters {
  timeframe: string;
  analysisDepth: number;
  includePatterns: boolean;
  includePredictions: boolean;
  comparisonMode: string;
  confidenceThreshold: number;
}

const TIMEFRAME_OPTIONS = [
  { value: '1y', label: '1 Year' },
  { value: '2y', label: '2 Years' },
  { value: '3y', label: '3 Years' },
  { value: '5y', label: '5 Years' },
  { value: '10y', label: '10 Years' }
];

const COMPARISON_MODES = [
  { value: 'sector', label: 'Sector Average' },
  { value: 'market', label: 'Market Index' },
  { value: 'custom', label: 'Custom Companies' },
  { value: 'none', label: 'No Comparison' }
];

export function EPSAnalysisForm({ onSubmit, isLoading }: EPSAnalysisFormProps) {
  const [symbol, setSymbol] = useState('');
  const [parameters, setParameters] = useState<AnalysisParameters>({
    timeframe: '3y',
    analysisDepth: 3,
    includePatterns: true,
    includePredictions: false,
    comparisonMode: 'sector',
    confidenceThreshold: 70
  });

  const [symbolSuggestions, setSymbolSuggestions] = useState<string[]>([]);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const handleSymbolChange = (value: string) => {
    setSymbol(value.toUpperCase());
    
    // Mock symbol suggestions - replace with actual API
    if (value.length >= 2) {
      const mockSuggestions = ['AAPL', 'MSFT', 'GOOGL', 'AMZN', 'TSLA', 'META']
        .filter(s => s.startsWith(value.toUpperCase()));
      setSymbolSuggestions(mockSuggestions);
    } else {
      setSymbolSuggestions([]);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (symbol.trim()) {
      onSubmit(symbol.trim(), parameters);
    }
  };

  const updateParameter = <K extends keyof AnalysisParameters>(
    key: K, 
    value: AnalysisParameters[K]
  ) => {
    setParameters(prev => ({ ...prev, [key]: value }));
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Symbol Input */}
      <div className="space-y-2">
        <Label htmlFor="symbol" className="text-sm font-medium">
          Stock Symbol
        </Label>
        <div className="relative">
          <Input
            id="symbol"
            type="text"
            placeholder="Enter stock symbol (e.g., AAPL)"
            value={symbol}
            onChange={(e) => handleSymbolChange(e.target.value)}
            className="pr-10"
            maxLength={10}
          />
          <TrendingUp className="absolute right-3 top-3 h-4 w-4 text-muted-foreground" />
        </div>
        
        {/* Symbol Suggestions */}
        {symbolSuggestions.length > 0 && (
          <div className="flex flex-wrap gap-2 mt-2">
            {symbolSuggestions.map((suggestion) => (
              <Badge
                key={suggestion}
                variant="outline"
                className="cursor-pointer hover:bg-accent"
                onClick={() => setSymbol(suggestion)}
              >
                {suggestion}
              </Badge>
            ))}
          </div>
        )}
      </div>

      {/* Basic Parameters */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label className="text-sm font-medium flex items-center gap-2">
            <Calendar className="h-4 w-4" />
            Analysis Timeframe
          </Label>
          <Select
            value={parameters.timeframe}
            onValueChange={(value) => updateParameter('timeframe', value)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {TIMEFRAME_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label className="text-sm font-medium flex items-center gap-2">
            <Target className="h-4 w-4" />
            Comparison Mode
          </Label>
          <Select
            value={parameters.comparisonMode}
            onValueChange={(value) => updateParameter('comparisonMode', value)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {COMPARISON_MODES.map((mode) => (
                <SelectItem key={mode.value} value={mode.value}>
                  {mode.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Advanced Settings Toggle */}
      <div className="flex items-center space-x-2">
        <Switch
          id="advanced"
          checked={showAdvanced}
          onCheckedChange={setShowAdvanced}
        />
        <Label htmlFor="advanced" className="text-sm font-medium">
          Advanced Settings
        </Label>
      </div>

      {/* Advanced Parameters */}
      {showAdvanced && (
        <Card className="border-dashed">
          <CardContent className="pt-6 space-y-4">
            {/* Analysis Depth */}
            <div className="space-y-3">
              <Label className="text-sm font-medium">
                Analysis Depth: {parameters.analysisDepth}
              </Label>
              <Slider
                value={[parameters.analysisDepth]}
                onValueChange={([value]) => updateParameter('analysisDepth', value)}
                max={5}
                min={1}
                step={1}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>Basic</span>
                <span>Comprehensive</span>
              </div>
            </div>

            {/* Confidence Threshold */}
            <div className="space-y-3">
              <Label className="text-sm font-medium">
                Confidence Threshold: {parameters.confidenceThreshold}%
              </Label>
              <Slider
                value={[parameters.confidenceThreshold]}
                onValueChange={([value]) => updateParameter('confidenceThreshold', value)}
                max={95}
                min={50}
                step={5}
                className="w-full"
              />
            </div>

            {/* Feature Toggles */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="flex items-center space-x-2">
                <Switch
                  id="patterns"
                  checked={parameters.includePatterns}
                  onCheckedChange={(checked) => updateParameter('includePatterns', checked)}
                />
                <Label htmlFor="patterns" className="text-sm">
                  Include Pattern Recognition
                </Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="predictions"
                  checked={parameters.includePredictions}
                  onCheckedChange={(checked) => updateParameter('includePredictions', checked)}
                />
                <Label htmlFor="predictions" className="text-sm flex items-center gap-1">
                  <Zap className="h-3 w-3" />
                  AI Predictions (Beta)
                </Label>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Submit Button */}
      <LoadingButton
        type="submit"
        loading={isLoading}
        disabled={!symbol.trim()}
        className="w-full"
        size="lg"
      >
        {isLoading ? 'Analyzing...' : 'Start EPS Analysis'}
      </LoadingButton>

      {/* Educational Note */}
      <div className="text-xs text-muted-foreground bg-muted/50 p-3 rounded-lg">
        <strong>Educational Notice:</strong> This analysis is for educational purposes only and should not be considered as investment advice. 
        Always conduct your own research and consult with qualified financial advisors before making investment decisions.
      </div>
    </form>
  );
}