'use client';

import { useState } from 'react';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { X, Plus, TrendingUp, TrendingDown, Minus } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Input } from '@epsx/ui';

interface ComparisonTableProps {
  primarySymbol: string;
  comparisonSymbols: string[];
  onRemoveComparison: (symbol: string) => void;
}

interface CompanyData {
  symbol: string;
  name: string;
  currentEPS: number;
  previousEPS: number;
  qoqGrowth: number;
  yoyGrowth: number;
  avgGrowth: number;
  volatility: number;
  marketCap: string;
  sector: string;
  peRatio: number;
}

// Mock comparison data
const mockCompanyData: Record<string, CompanyData> = {
  'AAPL': {
    symbol: 'AAPL',
    name: 'Apple Inc.',
    currentEPS: 2.18,
    previousEPS: 1.88,
    qoqGrowth: 15.9,
    yoyGrowth: 24.6,
    avgGrowth: 18.2,
    volatility: 12.4,
    marketCap: '2.8T',
    sector: 'Technology',
    peRatio: 28.5
  },
  'MSFT': {
    symbol: 'MSFT',
    name: 'Microsoft Corporation',
    currentEPS: 2.45,
    previousEPS: 2.22,
    qoqGrowth: 10.4,
    yoyGrowth: 22.1,
    avgGrowth: 16.8,
    volatility: 14.2,
    marketCap: '2.1T',
    sector: 'Technology',
    peRatio: 31.2
  },
  'GOOGL': {
    symbol: 'GOOGL',
    name: 'Alphabet Inc.',
    currentEPS: 1.85,
    previousEPS: 1.62,
    qoqGrowth: 14.2,
    yoyGrowth: 19.8,
    avgGrowth: 15.4,
    volatility: 18.7,
    marketCap: '1.6T',
    sector: 'Technology',
    peRatio: 25.1
  },
  'AMZN': {
    symbol: 'AMZN',
    name: 'Amazon.com Inc.',
    currentEPS: 1.32,
    previousEPS: 0.98,
    qoqGrowth: 34.7,
    yoyGrowth: 28.3,
    avgGrowth: 21.8,
    volatility: 25.3,
    marketCap: '1.4T',
    sector: 'Consumer Discretionary',
    peRatio: 45.2
  }
};

export function ComparisonTable({ primarySymbol, comparisonSymbols, onRemoveComparison }: ComparisonTableProps) {
  const [newSymbol, setNewSymbol] = useState('');
  
  // Get data for all companies
  const allSymbols = [primarySymbol, ...comparisonSymbols];
  const companyData = allSymbols.map(symbol => 
    mockCompanyData[symbol] || {
      symbol,
      name: `${symbol} Corp.`,
      currentEPS: Math.random() * 3 + 1,
      previousEPS: Math.random() * 2.5 + 0.8,
      qoqGrowth: Math.random() * 30 + 5,
      yoyGrowth: Math.random() * 25 + 10,
      avgGrowth: Math.random() * 20 + 8,
      volatility: Math.random() * 20 + 10,
      marketCap: `${Math.floor(Math.random() * 500 + 50)}B`,
      sector: 'Technology',
      peRatio: Math.random() * 20 + 15
    }
  );

  const handleAddSymbol = () => {
    if (newSymbol.trim() && !allSymbols.includes(newSymbol.toUpperCase())) {
      // In real implementation, this would add to parent state
      setNewSymbol('');
    }
  };

  const getGrowthIcon = (growth: number) => {
    if (growth > 15) return <TrendingUp className="h-4 w-4 text-green-600" />;
    if (growth > 5) return <TrendingUp className="h-4 w-4 text-yellow-600" />;
    if (growth > 0) return <Minus className="h-4 w-4 text-gray-600" />;
    return <TrendingDown className="h-4 w-4 text-red-600" />;
  };

  const getGrowthColor = (growth: number) => {
    if (growth > 15) return 'text-green-600';
    if (growth > 5) return 'text-yellow-600';
    if (growth > 0) return 'text-gray-600';
    return 'text-red-600';
  };

  const getBestInCategory = (metric: keyof CompanyData) => {
    if (typeof companyData[0][metric] !== 'number') return null;
    
    let bestValue = companyData[0][metric] as number;
    let bestSymbol = companyData[0].symbol;
    
    companyData.forEach(company => {
      const value = company[metric] as number;
      if (metric === 'volatility') {
        // Lower volatility is better
        if (value < bestValue) {
          bestValue = value;
          bestSymbol = company.symbol;
        }
      } else {
        // Higher values are better for growth metrics
        if (value > bestValue) {
          bestValue = value;
          bestSymbol = company.symbol;
        }
      }
    });
    
    return bestSymbol;
  };

  return (
    <div className="space-y-4">
      {/* Add New Comparison */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <Input
              placeholder="Enter symbol to compare (e.g., MSFT)"
              value={newSymbol}
              onChange={(e) => setNewSymbol(e.target.value.toUpperCase())}
              onKeyPress={(e) => e.key === 'Enter' && handleAddSymbol()}
              className="flex-1"
            />
            <Button onClick={handleAddSymbol} disabled={!newSymbol.trim()}>
              <Plus className="h-4 w-4 mr-2" />
              Add
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Comparison Table */}
      <Card>
        <CardHeader>
          <CardTitle>EPS Comparison Analysis</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Company</TableHead>
                  <TableHead>Current EPS</TableHead>
                  <TableHead>QoQ Growth</TableHead>
                  <TableHead>YoY Growth</TableHead>
                  <TableHead>Avg Growth</TableHead>
                  <TableHead>Volatility</TableHead>
                  <TableHead>P/E Ratio</TableHead>
                  <TableHead>Market Cap</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {companyData.map((company, _index) => {
                  const isPrimary = company.symbol === primarySymbol;
                  
                  return (
                    <TableRow key={company.symbol} className={isPrimary ? 'bg-blue-50' : ''}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <div>
                            <div className="font-medium flex items-center gap-2">
                              {company.symbol}
                              {isPrimary && <Badge variant="outline">Primary</Badge>}
                            </div>
                            <div className="text-sm text-muted-foreground">{company.name}</div>
                          </div>
                        </div>
                      </TableCell>
                      
                      <TableCell>
                        <div className="flex items-center gap-1">
                          ${company.currentEPS.toFixed(2)}
                          {getBestInCategory('currentEPS') === company.symbol && (
                            <Badge variant="default" className="text-xs">Best</Badge>
                          )}
                        </div>
                      </TableCell>
                      
                      <TableCell>
                        <div className={`flex items-center gap-1 ${getGrowthColor(company.qoqGrowth)}`}>
                          {getGrowthIcon(company.qoqGrowth)}
                          <span className="font-medium">{company.qoqGrowth.toFixed(1)}%</span>
                          {getBestInCategory('qoqGrowth') === company.symbol && (
                            <Badge variant="default" className="text-xs">Best</Badge>
                          )}
                        </div>
                      </TableCell>
                      
                      <TableCell>
                        <div className={`flex items-center gap-1 ${getGrowthColor(company.yoyGrowth)}`}>
                          {getGrowthIcon(company.yoyGrowth)}
                          <span className="font-medium">{company.yoyGrowth.toFixed(1)}%</span>
                          {getBestInCategory('yoyGrowth') === company.symbol && (
                            <Badge variant="default" className="text-xs">Best</Badge>
                          )}
                        </div>
                      </TableCell>
                      
                      <TableCell>
                        <div className={`flex items-center gap-1 ${getGrowthColor(company.avgGrowth)}`}>
                          {getGrowthIcon(company.avgGrowth)}
                          <span className="font-medium">{company.avgGrowth.toFixed(1)}%</span>
                          {getBestInCategory('avgGrowth') === company.symbol && (
                            <Badge variant="default" className="text-xs">Best</Badge>
                          )}
                        </div>
                      </TableCell>
                      
                      <TableCell>
                        <div className="flex items-center gap-1">
                          <span className={`font-medium ${
                            company.volatility < 15 ? 'text-green-600' : 
                            company.volatility < 20 ? 'text-yellow-600' : 'text-red-600'
                          }`}>
                            {company.volatility.toFixed(1)}%
                          </span>
                          {getBestInCategory('volatility') === company.symbol && (
                            <Badge variant="default" className="text-xs">Best</Badge>
                          )}
                        </div>
                      </TableCell>
                      
                      <TableCell>
                        <span className="font-medium">{company.peRatio.toFixed(1)}</span>
                      </TableCell>
                      
                      <TableCell>
                        <span className="font-medium">{company.marketCap}</span>
                      </TableCell>
                      
                      <TableCell>
                        {!isPrimary && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onRemoveComparison(company.symbol)}
                          >
                            <X className="h-4 w-4" />
                          </Button>
                        )}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      {/* Summary Insights */}
      <Card>
        <CardHeader>
          <CardTitle>Comparison Insights</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <h4 className="font-medium">Relative Performance</h4>
              <div className="text-sm space-y-1">
                <div>• {primarySymbol} ranks #{companyData.findIndex(c => c.symbol === primarySymbol) + 1} in YoY growth</div>
                <div>• Best EPS growth: {companyData.reduce((best, current) => 
                  current.yoyGrowth > best.yoyGrowth ? current : best
                ).symbol} ({companyData.reduce((best, current) => 
                  current.yoyGrowth > best.yoyGrowth ? current : best
                ).yoyGrowth.toFixed(1)}%)</div>
                <div>• Most consistent: {companyData.reduce((best, current) => 
                  current.volatility < best.volatility ? current : best
                ).symbol} ({companyData.reduce((best, current) => 
                  current.volatility < best.volatility ? current : best
                ).volatility.toFixed(1)}% volatility)</div>
              </div>
            </div>
            
            <div className="space-y-2">
              <h4 className="font-medium">Sector Analysis</h4>
              <div className="text-sm space-y-1">
                <div>• Sector average growth: {(companyData.reduce((sum, c) => sum + c.yoyGrowth, 0) / companyData.length).toFixed(1)}%</div>
                <div>• {primarySymbol} vs sector: {(
                  companyData.find(c => c.symbol === primarySymbol)!.yoyGrowth - 
                  (companyData.reduce((sum, c) => sum + c.yoyGrowth, 0) / companyData.length)
                ).toFixed(1)}% difference</div>
                <div>• P/E range: {Math.min(...companyData.map(c => c.peRatio)).toFixed(1)} - {Math.max(...companyData.map(c => c.peRatio)).toFixed(1)}</div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}