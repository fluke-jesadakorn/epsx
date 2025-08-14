'use client';

import { useState } from 'react';
import { Progress } from '@/components/ui/progress';
import { 
  TrendingUp, 
  TrendingDown, 
  Activity, 
  Target, 
  AlertTriangle, 
  CheckCircle, 
  Brain,
  Zap
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';

interface PatternRecognitionProps {
  patterns: any[];
  confidence: number;
  symbol: string;
}

interface Pattern {
  id: string;
  type: string;
  name: string;
  description: string;
  confidence: number;
  timeframe: string;
  strength: 'weak' | 'moderate' | 'strong';
  direction: 'bullish' | 'bearish' | 'neutral';
  indicators: string[];
  probability: number;
  historicalAccuracy: number;
}

const mockPatterns: Pattern[] = [
  {
    id: '1',
    type: 'growth_acceleration',
    name: 'Growth Acceleration',
    description: 'EPS growth rate is accelerating over consecutive quarters',
    confidence: 87,
    timeframe: 'Last 4 quarters',
    strength: 'strong',
    direction: 'bullish',
    indicators: ['Increasing QoQ growth', 'Revenue momentum', 'Margin expansion'],
    probability: 78,
    historicalAccuracy: 84
  },
  {
    id: '2',
    type: 'seasonal_pattern',
    name: 'Seasonal Strength',
    description: 'Strong Q4 performance pattern over multiple years',
    confidence: 73,
    timeframe: 'Multi-year trend',
    strength: 'moderate',
    direction: 'bullish',
    indicators: ['Q4 outperformance', 'Holiday season boost', 'Inventory turnover'],
    probability: 65,
    historicalAccuracy: 71
  },
  {
    id: '3',
    type: 'volatility_compression',
    name: 'Volatility Compression',
    description: 'EPS volatility decreasing, indicating business stability',
    confidence: 91,
    timeframe: 'Last 8 quarters',
    strength: 'strong',
    direction: 'bullish',
    indicators: ['Reduced variance', 'Consistent delivery', 'Business maturity'],
    probability: 82,
    historicalAccuracy: 89
  },
  {
    id: '4',
    type: 'mean_reversion',
    name: 'Mean Reversion Signal',
    description: 'EPS below historical average, potential for recovery',
    confidence: 62,
    timeframe: 'Current quarter',
    strength: 'weak',
    direction: 'neutral',
    indicators: ['Below 5-year average', 'Cyclical industry', 'Market correction'],
    probability: 58,
    historicalAccuracy: 64
  }
];

export function PatternRecognition({ patterns, confidence, symbol: _symbol }: PatternRecognitionProps) {
  const [selectedPattern, setSelectedPattern] = useState<Pattern | null>(null);
  const [activeTab, setActiveTab] = useState('detected');

  // Use mock data if no patterns provided
  const displayPatterns = patterns?.length > 0 ? patterns : mockPatterns;

  const getPatternIcon = (type: string) => {
    switch (type) {
      case 'growth_acceleration':
        return <TrendingUp className="h-4 w-4" />;
      case 'seasonal_pattern':
        return <Activity className="h-4 w-4" />;
      case 'volatility_compression':
        return <Target className="h-4 w-4" />;
      case 'mean_reversion':
        return <TrendingDown className="h-4 w-4" />;
      default:
        return <Brain className="h-4 w-4" />;
    }
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 80) return 'text-green-600';
    if (confidence >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getStrengthBadge = (strength: string) => {
    const variants = {
      weak: 'secondary',
      moderate: 'outline',
      strong: 'default'
    } as const;
    
    return <Badge variant={variants[strength as keyof typeof variants]}>{strength}</Badge>;
  };

  const getDirectionBadge = (direction: string) => {
    const colors = {
      bullish: 'bg-green-100 text-green-800',
      bearish: 'bg-red-100 text-red-800',
      neutral: 'bg-gray-100 text-gray-800'
    };
    
    return (
      <Badge className={colors[direction as keyof typeof colors]}>
        {direction}
      </Badge>
    );
  };

  return (
    <div className="space-y-4">
      {/* Overall Confidence Score */}
      <Card className="bg-gradient-to-r from-blue-50 to-indigo-50 border-blue-200">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-blue-100 rounded-lg">
                <Brain className="h-5 w-5 text-blue-600" />
              </div>
              <div>
                <h3 className="font-medium">AI Pattern Analysis</h3>
                <p className="text-sm text-muted-foreground">
                  Overall confidence in pattern detection
                </p>
              </div>
            </div>
            <div className="text-right">
              <div className="text-2xl font-bold text-blue-600">
                {confidence || 78}%
              </div>
              <Progress value={confidence || 78} className="w-20 h-2 mt-1" />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Pattern Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="detected">Detected Patterns</TabsTrigger>
          <TabsTrigger value="predictions">Predictions</TabsTrigger>
          <TabsTrigger value="alerts">Pattern Alerts</TabsTrigger>
        </TabsList>

        <TabsContent value="detected" className="space-y-3">
          {displayPatterns.map((pattern) => (
            <Card 
              key={pattern.id} 
              className={`cursor-pointer transition-all hover:shadow-md ${
                selectedPattern?.id === pattern.id ? 'ring-2 ring-primary' : ''
              }`}
              onClick={() => setSelectedPattern(pattern)}
            >
              <CardContent className="p-4">
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-3 flex-1">
                    <div className="p-2 bg-primary/10 rounded-lg">
                      {getPatternIcon(pattern.type)}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <h4 className="font-medium">{pattern.name}</h4>
                        {getStrengthBadge(pattern.strength)}
                        {getDirectionBadge(pattern.direction)}
                      </div>
                      <p className="text-sm text-muted-foreground mb-2">
                        {pattern.description}
                      </p>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <span>{pattern.timeframe}</span>
                        <span>Historical Accuracy: {pattern.historicalAccuracy}%</span>
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`text-lg font-bold ${getConfidenceColor(pattern.confidence)}`}>
                      {pattern.confidence}%
                    </div>
                    <Progress value={pattern.confidence} className="w-16 h-1 mt-1" />
                  </div>
                </div>

                {/* Pattern Details */}
                {selectedPattern?.id === pattern.id && (
                  <div className="mt-4 pt-4 border-t space-y-3">
                    <div>
                      <h5 className="font-medium text-sm mb-2">Key Indicators</h5>
                      <div className="flex flex-wrap gap-2">
                        {pattern.indicators.map((indicator, index) => (
                          <Badge key={index} variant="outline" className="text-xs">
                            {indicator}
                          </Badge>
                        ))}
                      </div>
                    </div>
                    
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <span className="text-muted-foreground">Success Probability:</span>
                        <span className="ml-2 font-medium">{pattern.probability}%</span>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Pattern Strength:</span>
                        <span className="ml-2 font-medium capitalize">{pattern.strength}</span>
                      </div>
                    </div>

                    <Button size="sm" className="w-full">
                      <AlertTriangle className="h-4 w-4 mr-2" />
                      Create Pattern Alert
                    </Button>
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </TabsContent>

        <TabsContent value="predictions" className="space-y-3">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Zap className="h-5 w-5" />
                AI Predictions
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <AlertTriangle className="h-4 w-4 text-yellow-600" />
                  <span className="font-medium text-yellow-800">Next Quarter Forecast</span>
                </div>
                <p className="text-sm text-yellow-700">
                  Based on detected patterns, EPS is predicted to increase by 12-18% next quarter 
                  (Confidence: 73%)
                </p>
              </div>

              <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle className="h-4 w-4 text-green-600" />
                  <span className="font-medium text-green-800">Long-term Outlook</span>
                </div>
                <p className="text-sm text-green-700">
                  Growth acceleration pattern suggests sustained earnings growth over the next 2-3 quarters
                </p>
              </div>

              <div className="text-xs text-muted-foreground bg-muted/50 p-3 rounded-lg">
                <strong>Disclaimer:</strong> AI predictions are experimental and for educational purposes only. 
                Past pattern performance does not guarantee future results.
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="alerts" className="space-y-3">
          <Card>
            <CardHeader>
              <CardTitle>Pattern-Based Alerts</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <div className="font-medium">Growth Acceleration Alert</div>
                    <div className="text-sm text-muted-foreground">
                      Notify when growth rate increases for 3+ consecutive quarters
                    </div>
                  </div>
                  <Button size="sm" variant="outline">Configure</Button>
                </div>

                <div className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <div className="font-medium">Pattern Break Alert</div>
                    <div className="text-sm text-muted-foreground">
                      Alert when established patterns show signs of breaking
                    </div>
                  </div>
                  <Button size="sm" variant="outline">Configure</Button>
                </div>

                <Button className="w-full">
                  <AlertTriangle className="h-4 w-4 mr-2" />
                  Create Custom Pattern Alert
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}