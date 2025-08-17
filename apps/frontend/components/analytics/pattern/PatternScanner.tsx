'use client';

import { useState } from 'react';
import { Progress } from '@/components/ui/progress';
import { Checkbox } from '@/components/ui/checkbox';
import { 
  Zap as _Zap, 
  Play, 
  Pause, 
  Settings,
  Target,
  Clock,
  Activity
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Label } from '@/components/ui';

interface PatternScannerProps {
  isScanning: boolean;
  onScan: () => void;
  timeframe: string;
  patternType: string;
}

const SCAN_TARGETS = [
  { id: 'sp500', label: 'S&P 500', count: 500 },
  { id: 'nasdaq100', label: 'NASDAQ 100', count: 100 },
  { id: 'dow30', label: 'Dow 30', count: 30 },
  { id: 'custom', label: 'Custom Watchlist', count: 25 },
  { id: 'all', label: 'All Markets', count: 8000 }
];

const PATTERN_CATEGORIES = [
  { id: 'reversal', label: 'Reversal Patterns', enabled: true },
  { id: 'continuation', label: 'Continuation Patterns', enabled: true },
  { id: 'breakout', label: 'Breakout Patterns', enabled: false },
  { id: 'consolidation', label: 'Consolidation Patterns', enabled: false }
];

const TECHNICAL_INDICATORS = [
  { id: 'volume', label: 'Volume Analysis', enabled: true },
  { id: 'rsi', label: 'RSI Divergence', enabled: true },
  { id: 'macd', label: 'MACD Signals', enabled: false },
  { id: 'bollinger', label: 'Bollinger Bands', enabled: false },
  { id: 'support_resistance', label: 'Support/Resistance', enabled: true }
];

export function PatternScanner({ isScanning, onScan, timeframe, patternType }: PatternScannerProps) {
  const [scanTarget, setScanTarget] = useState('sp500');
  const [minConfidence, setMinConfidence] = useState('70');
  const [scanProgress, setScanProgress] = useState(0);
  const [scanStats, setScanStats] = useState({
    processed: 0,
    total: 500,
    patternsFound: 0,
    avgConfidence: 0
  });
  const [patterns, setPatterns] = useState(PATTERN_CATEGORIES);
  const [indicators, setIndicators] = useState(TECHNICAL_INDICATORS);

  const handlePatternToggle = (id: string, enabled: boolean) => {
    setPatterns(patterns.map(p => p.id === id ? { ...p, enabled } : p));
  };

  const handleIndicatorToggle = (id: string, enabled: boolean) => {
    setIndicators(indicators.map(i => i.id === id ? { ...i, enabled } : i));
  };

  const mockScanProgress = () => {
    if (!isScanning) return;
    
    const interval = setInterval(() => {
      setScanProgress(prev => {
        if (prev >= 100) {
          clearInterval(interval);
          return 100;
        }
        const newProgress = prev + Math.random() * 10;
        setScanStats(prevStats => ({
          ...prevStats,
          processed: Math.floor((newProgress / 100) * prevStats.total),
          patternsFound: Math.floor(Math.random() * 15) + 1,
          avgConfidence: 65 + Math.random() * 25
        }));
        return Math.min(newProgress, 100);
      });
    }, 200);

    return () => clearInterval(interval);
  };

  React.useEffect(() => {
    if (isScanning) {
      setScanProgress(0);
      mockScanProgress();
    }
  }, [isScanning]);

  return (
    <div className="space-y-6">
      {/* Scanner Configuration */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Target className="h-5 w-5" />
              Scan Configuration
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>Scan Target</Label>
              <Select value={scanTarget} onValueChange={setScanTarget}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SCAN_TARGETS.map((target) => (
                    <SelectItem key={target.id} value={target.id}>
                      {target.label} ({target.count} symbols)
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label>Minimum Confidence</Label>
              <Select value={minConfidence} onValueChange={setMinConfidence}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="50">50% - Low</SelectItem>
                  <SelectItem value="70">70% - Medium</SelectItem>
                  <SelectItem value="80">80% - High</SelectItem>
                  <SelectItem value="90">90% - Very High</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-3">
              <Label>Pattern Categories</Label>
              {patterns.map((pattern) => (
                <div key={pattern.id} className="flex items-center space-x-2">
                  <Checkbox
                    id={pattern.id}
                    checked={pattern.enabled}
                    onCheckedChange={(checked) => 
                      handlePatternToggle(pattern.id, checked as boolean)
                    }
                  />
                  <Label htmlFor={pattern.id} className="text-sm font-normal">
                    {pattern.label}
                  </Label>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Settings className="h-5 w-5" />
              Technical Analysis
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-3">
              <Label>Technical Indicators</Label>
              {indicators.map((indicator) => (
                <div key={indicator.id} className="flex items-center space-x-2">
                  <Checkbox
                    id={indicator.id}
                    checked={indicator.enabled}
                    onCheckedChange={(checked) => 
                      handleIndicatorToggle(indicator.id, checked as boolean)
                    }
                  />
                  <Label htmlFor={indicator.id} className="text-sm font-normal">
                    {indicator.label}
                  </Label>
                </div>
              ))}
            </div>

            <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <h4 className="font-medium text-blue-900 mb-2">Scan Settings</h4>
              <div className="text-sm text-blue-700 space-y-1">
                <div>• Timeframe: {timeframe}</div>
                <div>• Pattern Type: {patternType}</div>
                <div>• Min Confidence: {minConfidence}%</div>
                <div>• Enabled Patterns: {patterns.filter(p => p.enabled).length}/4</div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Scanner Control */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Live Pattern Scanner
            </div>
            <Badge variant={isScanning ? 'default' : 'secondary'}>
              {isScanning ? 'Active' : 'Idle'}
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-center">
            <Button 
              onClick={onScan} 
              disabled={isScanning}
              size="lg"
              className="w-48"
            >
              {isScanning ? (
                <>
                  <Pause className="h-4 w-4 mr-2" />
                  Scanning...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4 mr-2" />
                  Start Scan
                </>
              )}
            </Button>
          </div>

          {isScanning && (
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span>Progress</span>
                  <span>{scanProgress.toFixed(0)}%</span>
                </div>
                <Progress value={scanProgress} className="w-full" />
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="text-center">
                  <div className="text-2xl font-bold">{scanStats.processed}</div>
                  <div className="text-xs text-muted-foreground">Processed</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold">{scanStats.total}</div>
                  <div className="text-xs text-muted-foreground">Total</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-600">{scanStats.patternsFound}</div>
                  <div className="text-xs text-muted-foreground">Found</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold">{scanStats.avgConfidence.toFixed(0)}%</div>
                  <div className="text-xs text-muted-foreground">Avg Conf.</div>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Recent Scans */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Recent Scans
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {[
              { time: '10 minutes ago', target: 'S&P 500', patterns: 12, duration: '2m 34s' },
              { time: '1 hour ago', target: 'NASDAQ 100', patterns: 8, duration: '1m 12s' },
              { time: '3 hours ago', target: 'Custom List', patterns: 5, duration: '45s' }
            ].map((scan, index) => (
              <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                <div>
                  <div className="font-medium">{scan.target}</div>
                  <div className="text-sm text-muted-foreground">{scan.time}</div>
                </div>
                <div className="text-right">
                  <div className="font-medium">{scan.patterns} patterns</div>
                  <div className="text-sm text-muted-foreground">{scan.duration}</div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}