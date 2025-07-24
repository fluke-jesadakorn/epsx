'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Calendar } from '@/components/ui/calendar';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { 
  Search, 
  Calendar as CalendarIcon, 
  Filter,
  Eye,
  TrendingUp,
  TrendingDown,
  BarChart3,
  Download,
  ArrowUpDown
} from 'lucide-react';

interface HistoricalPattern {
  id: string;
  symbol: string;
  type: string;
  name: string;
  detectedAt: string;
  confidence: number;
  direction: 'bullish' | 'bearish' | 'neutral';
  outcome: 'success' | 'failed' | 'pending';
  actualMove: number;
  predictedMove: number;
  timeframe: string;
  duration: number; // days
}

const mockHistoricalPatterns: HistoricalPattern[] = [
  {
    id: '1',
    symbol: 'AAPL',
    type: 'breakout',
    name: 'Ascending Triangle',
    detectedAt: '2024-01-10T09:30:00Z',
    confidence: 85,
    direction: 'bullish',
    outcome: 'success',
    actualMove: 8.5,
    predictedMove: 6.2,
    timeframe: 'daily',
    duration: 5
  },
  {
    id: '2',
    symbol: 'TSLA',
    type: 'reversal',
    name: 'Double Bottom',
    detectedAt: '2024-01-08T14:20:00Z',
    confidence: 78,
    direction: 'bullish',
    outcome: 'success',
    actualMove: 12.3,
    predictedMove: 9.1,
    timeframe: 'weekly',
    duration: 14
  },
  {
    id: '3',
    symbol: 'MSFT',
    type: 'trend',
    name: 'Bull Flag',
    detectedAt: '2024-01-05T11:15:00Z',
    confidence: 92,
    direction: 'bullish',
    outcome: 'failed',
    actualMove: -2.1,
    predictedMove: 5.8,
    timeframe: 'daily',
    duration: 3
  },
  {
    id: '4',
    symbol: 'GOOGL',
    type: 'reversal',
    name: 'Head and Shoulders',
    detectedAt: '2024-01-03T10:45:00Z',
    confidence: 88,
    direction: 'bearish',
    outcome: 'success',
    actualMove: -7.2,
    predictedMove: -6.5,
    timeframe: 'daily',
    duration: 8
  },
  {
    id: '5',
    symbol: 'AMZN',
    type: 'continuation',
    name: 'Pennant',
    detectedAt: '2024-01-01T13:30:00Z',
    confidence: 75,
    direction: 'bullish',
    outcome: 'pending',
    actualMove: 3.2,
    predictedMove: 4.5,
    timeframe: 'intraday',
    duration: 2
  }
];

const SORT_OPTIONS = [
  { value: 'date_desc', label: 'Date (Newest)' },
  { value: 'date_asc', label: 'Date (Oldest)' },
  { value: 'confidence_desc', label: 'Confidence (High)' },
  { value: 'confidence_asc', label: 'Confidence (Low)' },
  { value: 'performance_desc', label: 'Performance (Best)' },
  { value: 'performance_asc', label: 'Performance (Worst)' }
];

const OUTCOME_FILTERS = [
  { value: 'all', label: 'All Outcomes' },
  { value: 'success', label: 'Successful' },
  { value: 'failed', label: 'Failed' },
  { value: 'pending', label: 'Pending' }
];

export function PatternHistory() {
  const [patterns, setPatterns] = useState<HistoricalPattern[]>(mockHistoricalPatterns);
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState('date_desc');
  const [outcomeFilter, setOutcomeFilter] = useState('all');
  const [selectedDate, setSelectedDate] = useState<Date>();

  const filteredAndSortedPatterns = patterns
    .filter(pattern => {
      const matchesSearch = !searchTerm || 
        pattern.symbol.toLowerCase().includes(searchTerm.toLowerCase()) ||
        pattern.name.toLowerCase().includes(searchTerm.toLowerCase());
      
      const matchesOutcome = outcomeFilter === 'all' || pattern.outcome === outcomeFilter;
      
      const matchesDate = !selectedDate || 
        new Date(pattern.detectedAt).toDateString() === selectedDate.toDateString();
      
      return matchesSearch && matchesOutcome && matchesDate;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'date_desc':
          return new Date(b.detectedAt).getTime() - new Date(a.detectedAt).getTime();
        case 'date_asc':
          return new Date(a.detectedAt).getTime() - new Date(b.detectedAt).getTime();
        case 'confidence_desc':
          return b.confidence - a.confidence;
        case 'confidence_asc':
          return a.confidence - b.confidence;
        case 'performance_desc':
          return Math.abs(b.actualMove) - Math.abs(a.actualMove);
        case 'performance_asc':
          return Math.abs(a.actualMove) - Math.abs(b.actualMove);
        default:
          return 0;
      }
    });

  const getOutcomeBadge = (outcome: string) => {
    const variants = {
      success: 'bg-green-100 text-green-800',
      failed: 'bg-red-100 text-red-800',
      pending: 'bg-yellow-100 text-yellow-800'
    };
    return variants[outcome as keyof typeof variants] || variants.pending;
  };

  const getDirectionIcon = (direction: string, move: number) => {
    if (direction === 'bullish' && move > 0) return <TrendingUp className="h-4 w-4 text-green-600" />;
    if (direction === 'bearish' && move < 0) return <TrendingDown className="h-4 w-4 text-red-600" />;
    if (move > 0) return <TrendingUp className="h-4 w-4 text-green-600" />;
    if (move < 0) return <TrendingDown className="h-4 w-4 text-red-600" />;
    return <BarChart3 className="h-4 w-4 text-gray-600" />;
  };

  const calculateStats = () => {
    const totalPatterns = filteredAndSortedPatterns.length;
    const successful = filteredAndSortedPatterns.filter(p => p.outcome === 'success').length;
    const failed = filteredAndSortedPatterns.filter(p => p.outcome === 'failed').length;
    const pending = filteredAndSortedPatterns.filter(p => p.outcome === 'pending').length;
    
    const successRate = totalPatterns > 0 ? (successful / (successful + failed)) * 100 : 0;
    const avgConfidence = totalPatterns > 0 ? 
      filteredAndSortedPatterns.reduce((sum, p) => sum + p.confidence, 0) / totalPatterns : 0;

    return {
      total: totalPatterns,
      successful,
      failed,
      pending,
      successRate,
      avgConfidence
    };
  };

  const stats = calculateStats();

  return (
    <div className="space-y-6">
      {/* Filters and Controls */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex-1 min-w-48">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search patterns or symbols..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-9"
                />
              </div>
            </div>
            
            <Select value={sortBy} onValueChange={setSortBy}>
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SORT_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={outcomeFilter} onValueChange={setOutcomeFilter}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {OUTCOME_FILTERS.map((filter) => (
                  <SelectItem key={filter.value} value={filter.value}>
                    {filter.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Popover>
              <PopoverTrigger asChild>
                <Button variant="outline">
                  <CalendarIcon className="h-4 w-4 mr-2" />
                  {selectedDate ? selectedDate.toLocaleDateString() : 'Date'}
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-auto p-0" align="start">
                <Calendar
                  mode="single"
                  selected={selectedDate}
                  onSelect={setSelectedDate}
                  initialFocus
                />
              </PopoverContent>
            </Popover>

            {(searchTerm || outcomeFilter !== 'all' || selectedDate) && (
              <Button 
                variant="outline" 
                onClick={() => {
                  setSearchTerm('');
                  setOutcomeFilter('all');
                  setSelectedDate(undefined);
                }}
              >
                Clear Filters
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Summary Statistics */}
      <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
        <Card>
          <CardContent className="p-3 text-center">
            <div className="text-lg font-bold">{stats.total}</div>
            <div className="text-xs text-muted-foreground">Total</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-3 text-center">
            <div className="text-lg font-bold text-green-600">{stats.successful}</div>
            <div className="text-xs text-muted-foreground">Successful</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-3 text-center">
            <div className="text-lg font-bold text-red-600">{stats.failed}</div>
            <div className="text-xs text-muted-foreground">Failed</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-3 text-center">
            <div className="text-lg font-bold text-yellow-600">{stats.pending}</div>
            <div className="text-xs text-muted-foreground">Pending</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-3 text-center">
            <div className="text-lg font-bold">{stats.successRate.toFixed(1)}%</div>
            <div className="text-xs text-muted-foreground">Success Rate</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-3 text-center">
            <div className="text-lg font-bold">{stats.avgConfidence.toFixed(0)}%</div>
            <div className="text-xs text-muted-foreground">Avg Confidence</div>
          </CardContent>
        </Card>
      </div>

      {/* Pattern History Table */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Pattern History</CardTitle>
            <div className="flex gap-2">
              <Button variant="outline" size="sm">
                <Download className="h-4 w-4 mr-2" />
                Export
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Symbol</TableHead>
                  <TableHead>Pattern</TableHead>
                  <TableHead>Detected</TableHead>
                  <TableHead>Confidence</TableHead>
                  <TableHead>Direction</TableHead>
                  <TableHead>Predicted</TableHead>
                  <TableHead>Actual</TableHead>
                  <TableHead>Outcome</TableHead>
                  <TableHead>Duration</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredAndSortedPatterns.map((pattern) => (
                  <TableRow key={pattern.id}>
                    <TableCell>
                      <div className="font-medium">{pattern.symbol}</div>
                      <div className="text-xs text-muted-foreground">{pattern.timeframe}</div>
                    </TableCell>
                    
                    <TableCell>
                      <div className="font-medium">{pattern.name}</div>
                      <Badge variant="outline" className="text-xs">{pattern.type}</Badge>
                    </TableCell>
                    
                    <TableCell>
                      <div className="text-sm">
                        {new Date(pattern.detectedAt).toLocaleDateString()}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {new Date(pattern.detectedAt).toLocaleTimeString([], { 
                          hour: '2-digit', 
                          minute: '2-digit' 
                        })}
                      </div>
                    </TableCell>
                    
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <span className={`font-medium ${
                          pattern.confidence >= 80 ? 'text-green-600' :
                          pattern.confidence >= 60 ? 'text-yellow-600' : 'text-red-600'
                        }`}>
                          {pattern.confidence}%
                        </span>
                      </div>
                    </TableCell>
                    
                    <TableCell>
                      <Badge 
                        variant="outline"
                        className={
                          pattern.direction === 'bullish' ? 'border-green-600 text-green-600' :
                          pattern.direction === 'bearish' ? 'border-red-600 text-red-600' :
                          'border-gray-600 text-gray-600'
                        }
                      >
                        {pattern.direction}
                      </Badge>
                    </TableCell>
                    
                    <TableCell>
                      <div className="flex items-center gap-1">
                        {getDirectionIcon(pattern.direction, pattern.predictedMove)}
                        <span className="font-medium">
                          {pattern.predictedMove > 0 ? '+' : ''}{pattern.predictedMove.toFixed(1)}%
                        </span>
                      </div>
                    </TableCell>
                    
                    <TableCell>
                      <div className="flex items-center gap-1">
                        {getDirectionIcon(pattern.direction, pattern.actualMove)}
                        <span className={`font-medium ${
                          pattern.actualMove > 0 ? 'text-green-600' : 'text-red-600'
                        }`}>
                          {pattern.actualMove > 0 ? '+' : ''}{pattern.actualMove.toFixed(1)}%
                        </span>
                      </div>
                    </TableCell>
                    
                    <TableCell>
                      <Badge className={getOutcomeBadge(pattern.outcome)}>
                        {pattern.outcome}
                      </Badge>
                    </TableCell>
                    
                    <TableCell>
                      <span className="text-sm">{pattern.duration} day{pattern.duration !== 1 ? 's' : ''}</span>
                    </TableCell>
                    
                    <TableCell>
                      <Button variant="ghost" size="sm">
                        <Eye className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
          
          {filteredAndSortedPatterns.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">
              No patterns found matching your criteria
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}