'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Search, 
  TrendingUp, 
  Brain, 
  AlertTriangle,
  Filter,
  Zap,
  Eye,
  Star,
  Calendar
} from 'lucide-react';
import { PatternScanner } from '@/components/analytics/pattern/PatternScanner';
import { AlertSystem } from '@/components/analytics/pattern/AlertSystem';
import { PatternVisualization } from '@/components/analytics/pattern/PatternVisualization';
import { PatternHistory } from '@/components/analytics/pattern/PatternHistory';
import { useFirebaseAnalytics } from '@/hooks/useFirebaseAnalytics';

interface Pattern {
  id: string;
  symbol: string;
  type: string;
  name: string;
  confidence: number;
  strength: 'weak' | 'moderate' | 'strong';
  direction: 'bullish' | 'bearish' | 'neutral';
  timeframe: string;
  detectedAt: string;
  description: string;
  indicators: string[];
  probability: number;
  riskLevel: 'low' | 'medium' | 'high';
}

const PATTERN_TYPES = [
  { value: 'all', label: 'All Patterns' },
  { value: 'trend', label: 'Trend Patterns' },
  { value: 'reversal', label: 'Reversal Patterns' },
  { value: 'continuation', label: 'Continuation Patterns' },
  { value: 'breakout', label: 'Breakout Patterns' }
];

const TIMEFRAMES = [
  { value: 'intraday', label: 'Intraday' },
  { value: 'daily', label: 'Daily' },
  { value: 'weekly', label: 'Weekly' },
  { value: 'monthly', label: 'Monthly' }
];

const mockPatterns: Pattern[] = [
  {
    id: '1',
    symbol: 'AAPL',
    type: 'breakout',
    name: 'Ascending Triangle Breakout',
    confidence: 92,
    strength: 'strong',
    direction: 'bullish',
    timeframe: 'daily',
    detectedAt: '2024-01-15T10:30:00Z',
    description: 'Strong upward breakout from ascending triangle pattern with high volume confirmation',
    indicators: ['Volume spike', 'RSI > 60', 'Moving average crossover'],
    probability: 85,
    riskLevel: 'medium'
  },
  {
    id: '2',
    symbol: 'TSLA',
    type: 'reversal',
    name: 'Double Bottom',
    confidence: 78,
    strength: 'moderate',
    direction: 'bullish',
    timeframe: 'weekly',
    detectedAt: '2024-01-14T09:15:00Z',
    description: 'Classic double bottom formation suggesting trend reversal',
    indicators: ['Support level tested twice', 'Bullish divergence', 'Volume confirmation'],
    probability: 72,
    riskLevel: 'low'
  },
  {
    id: '3',
    symbol: 'MSFT',
    type: 'trend',
    name: 'Bull Flag',
    confidence: 89,
    strength: 'strong',
    direction: 'bullish',
    timeframe: 'daily',
    detectedAt: '2024-01-13T14:20:00Z',
    description: 'Bullish flag pattern indicating trend continuation',
    indicators: ['Flagpole formation', 'Consolidation phase', 'Volume decrease'],
    probability: 80,
    riskLevel: 'low'
  }
];

export default function PatternRecognitionPage() {
  const [patterns, setPatterns] = useState<Pattern[]>(mockPatterns);
  const [searchSymbol, setSearchSymbol] = useState('');
  const [selectedType, setSelectedType] = useState('all');
  const [selectedTimeframe, setSelectedTimeframe] = useState('daily');
  const [isScanning, setIsScanning] = useState(false);
  const { trackPatternRecognition } = useFirebaseAnalytics();

  useEffect(() => {
    trackPatternRecognition('dashboard_view', 'all');
  }, [trackPatternRecognition]);

  const handleScan = async () => {
    setIsScanning(true);
    trackPatternRecognition('scan_initiated', selectedTimeframe);
    
    // Simulate scanning process
    setTimeout(() => {
      setIsScanning(false);
      trackPatternRecognition('scan_completed', selectedTimeframe);
    }, 3000);
  };

  const filteredPatterns = patterns.filter(pattern => {
    const matchesSymbol = !searchSymbol || pattern.symbol.toLowerCase().includes(searchSymbol.toLowerCase());
    const matchesType = selectedType === 'all' || pattern.type === selectedType;
    const matchesTimeframe = pattern.timeframe === selectedTimeframe;
    return matchesSymbol && matchesType && matchesTimeframe;
  });

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 80) return 'text-green-600';
    if (confidence >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getDirectionBadge = (direction: string) => {
    const colors = {
      bullish: 'bg-green-100 text-green-800',
      bearish: 'bg-red-100 text-red-800',
      neutral: 'bg-gray-100 text-gray-800'
    };
    return colors[direction as keyof typeof colors] || colors.neutral;
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Brain className="h-8 w-8" />
            Pattern Recognition Dashboard
          </h1>
          <p className="text-muted-foreground mt-2">
            AI-powered pattern detection and analysis
          </p>
        </div>
        <Button onClick={handleScan} disabled={isScanning} size="lg">
          <Zap className="h-4 w-4 mr-2" />
          {isScanning ? 'Scanning...' : 'Scan Markets'}
        </Button>
      </div>

      {/* Controls */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex-1 min-w-48">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search by symbol..."
                  value={searchSymbol}
                  onChange={(e) => setSearchSymbol(e.target.value)}
                  className="pl-9"
                />
              </div>
            </div>
            
            <Select value={selectedType} onValueChange={setSelectedType}>
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {PATTERN_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={selectedTimeframe} onValueChange={setSelectedTimeframe}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {TIMEFRAMES.map((timeframe) => (
                  <SelectItem key={timeframe.value} value={timeframe.value}>
                    {timeframe.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Button variant="outline">
              <Filter className="h-4 w-4 mr-2" />
              Filters
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Active Patterns</p>
                <p className="text-2xl font-bold">{filteredPatterns.length}</p>
              </div>
              <TrendingUp className="h-8 w-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">High Confidence</p>
                <p className="text-2xl font-bold text-green-600">
                  {filteredPatterns.filter(p => p.confidence >= 80).length}
                </p>
              </div>
              <Star className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Bullish Signals</p>
                <p className="text-2xl font-bold text-green-600">
                  {filteredPatterns.filter(p => p.direction === 'bullish').length}
                </p>
              </div>
              <TrendingUp className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Active Alerts</p>
                <p className="text-2xl font-bold text-orange-600">3</p>
              </div>
              <AlertTriangle className="h-8 w-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <Tabs defaultValue="patterns" className="space-y-4">
        <TabsList>
          <TabsTrigger value="patterns">Pattern List</TabsTrigger>
          <TabsTrigger value="scanner">Live Scanner</TabsTrigger>
          <TabsTrigger value="alerts">Alert System</TabsTrigger>
          <TabsTrigger value="visualization">Visualization</TabsTrigger>
          <TabsTrigger value="history">History</TabsTrigger>
        </TabsList>

        <TabsContent value="patterns" className="space-y-4">
          <div className="grid gap-4">
            {filteredPatterns.map((pattern) => (
              <Card key={pattern.id} className="hover:shadow-md transition-shadow">
                <CardContent className="p-6">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-lg font-semibold">{pattern.symbol}</h3>
                        <Badge className={getDirectionBadge(pattern.direction)}>
                          {pattern.direction}
                        </Badge>
                        <Badge variant="outline">{pattern.type}</Badge>
                      </div>
                      
                      <h4 className="font-medium text-blue-600 mb-2">{pattern.name}</h4>
                      <p className="text-sm text-muted-foreground mb-3">{pattern.description}</p>
                      
                      <div className="flex flex-wrap gap-2 mb-3">
                        {pattern.indicators.map((indicator, index) => (
                          <Badge key={index} variant="secondary" className="text-xs">
                            {indicator}
                          </Badge>
                        ))}
                      </div>
                      
                      <div className="flex items-center gap-4 text-sm">
                        <span className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {new Date(pattern.detectedAt).toLocaleDateString()}
                        </span>
                        <span>Timeframe: {pattern.timeframe}</span>
                        <span>Risk: {pattern.riskLevel}</span>
                      </div>
                    </div>
                    
                    <div className="text-right">
                      <div className="mb-2">
                        <span className="text-sm text-muted-foreground">Confidence</span>
                        <div className={`text-2xl font-bold ${getConfidenceColor(pattern.confidence)}`}>
                          {pattern.confidence}%
                        </div>
                      </div>
                      
                      <div className="mb-3">
                        <span className="text-sm text-muted-foreground">Probability</span>
                        <div className="text-lg font-semibold">{pattern.probability}%</div>
                      </div>
                      
                      <Button size="sm" variant="outline">
                        <Eye className="h-3 w-3 mr-1" />
                        View Details
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="scanner">
          <PatternScanner 
            isScanning={isScanning}
            onScan={handleScan}
            timeframe={selectedTimeframe}
            patternType={selectedType}
          />
        </TabsContent>

        <TabsContent value="alerts">
          <AlertSystem patterns={patterns} />
        </TabsContent>

        <TabsContent value="visualization">
          <PatternVisualization patterns={filteredPatterns} />
        </TabsContent>

        <TabsContent value="history">
          <PatternHistory />
        </TabsContent>
      </Tabs>
    </div>
  );
}